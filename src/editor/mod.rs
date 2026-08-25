use crate::editor::theme::{
    UI_BORDER_ACTIVE, UI_BORDER_INACTIVE, UI_DMENU_BG, UI_DMENU_FG, UI_TEXT_MUTED,
    get_color_for_capture,
};
use crate::finder::{Finder, FinderLayout, list_files};
use crossterm::cursor::{
    Hide, MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp, SetCursorStyle, Show,
};
use crossterm::event::{Event, KeyCode, KeyModifiers, read};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{
    self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size,
};
use crossterm::{execute, queue};
use ropey::Rope;
use std::fs::{File, create_dir_all};
use std::io::{self, BufRead, BufReader, Error, Write, stdout};
use std::path::Path;
use std::path::PathBuf;
use tree_sitter::{InputEdit, Language, Point, QueryCursor};
use tree_sitter::{Parser, Tree};
use tree_sitter::{Query, StreamingIterator};
use tree_sitter_highlight::HighlightConfiguration;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub mod theme;
/// To init correct panel state in panel
pub const INIT_PANE_STATE: PaneState = PaneState {
    workspace: 1,
    view: 1,
    cursor: 0,
};
pub fn get_superscript(num: u8) -> &'static str {
    match num {
        1 => "¹",
        2 => "²",
        3 => "³",
        4 => "⁴",
        5 => "⁵",
        6 => "⁶",
        7 => "⁷",
        8 => "⁸",
        9 => "⁹",
        _ => "⁰",
    }
}
pub trait QwxUi<W: Write> {
    fn draw(&mut self, w: &mut W) -> Result<(), Error>;
}

impl<W: Write> QwxUi<W> for Qwx {
    fn draw(&mut self, w: &mut W) -> Result<(), Error> {
        execute!(w, Hide)?;
        // ==========================================
        // CALCUL DES MARGES POUR CENTRER LA GRILLE
        // ==========================================
        // On fixe une largeur maximale agréable à lire (ex: 180 caractères).
        // Si le terminal est plus petit, on prend toute la place (self.width).
        let max_width = 180.min(self.width);

        // On calcule l'espace vide restant pour trouver le point de départ en X
        let left_x = (self.width.saturating_sub(max_width)) / 2;

        // Nouvelles coordonnées basées sur la marge
        let right_x = left_x + max_width.saturating_sub(1);
        let mid_x = left_x + (max_width / 2);

        let top_y = 0;
        let bottom_y = self.height.saturating_sub(1);
        let mid_y = self.height / 2;

        // ==========================================
        // 1. DESSINER LE CADRE EXTÉRIEUR ET LA CROIX
        // ==========================================

        // Ligne horizontale dynamique adaptée à max_width
        let horiz_line = "─".repeat(max_width.saturating_sub(2) as usize);

        queue!(
            w,
            MoveTo(left_x, top_y),
            SetForegroundColor(UI_BORDER_INACTIVE),
            Print(format!("┌{}┐", horiz_line)),
            MoveTo(left_x, bottom_y),
            Print(format!("└{}┘", horiz_line))
        )?;

        // Lignes verticales extérieures (Gauche et Droite)
        for y in (top_y + 1)..bottom_y {
            if y != mid_y {
                queue!(
                    w,
                    MoveTo(left_x, y),
                    SetForegroundColor(UI_BORDER_INACTIVE),
                    Print("│"),
                    MoveTo(right_x, y),
                    Print("│")
                )?;
            }
        }

        // Ligne de séparation horizontale centrale
        for x in (left_x + 1)..right_x {
            if x != mid_x {
                queue!(
                    w,
                    MoveTo(x, mid_y),
                    SetForegroundColor(UI_BORDER_INACTIVE),
                    Print("─")
                )?;
            }
        }

        // Ligne de séparation verticale centrale
        for y in (top_y + 1)..bottom_y {
            if y != mid_y {
                queue!(
                    w,
                    MoveTo(mid_x, y),
                    SetForegroundColor(UI_BORDER_INACTIVE),
                    Print("│")
                )?;
            }
        }

        // Intersections
        queue!(
            w,
            SetForegroundColor(UI_BORDER_INACTIVE),
            MoveTo(left_x, mid_y),
            Print("├"),
            MoveTo(right_x, mid_y),
            Print("┤"),
            MoveTo(mid_x, top_y),
            Print("┬"),
            MoveTo(mid_x, bottom_y),
            Print("┴"),
            MoveTo(mid_x, mid_y),
            Print("┼")
        )?;

        // ==========================================
        // 2. DÉFINIR LES ZONES DES PANNEAUX
        // ==========================================
        // Les dimensions s'adaptent désormais parfaitement aux marges
        let panes_bounds = [
            (
                PaneFocus::TopLeft,
                left_x + 1,
                top_y + 1,
                (mid_x - left_x).saturating_sub(1),
                (mid_y - top_y).saturating_sub(1),
            ),
            (
                PaneFocus::TopRight,
                mid_x + 1,
                top_y + 1,
                (right_x - mid_x).saturating_sub(1),
                (mid_y - top_y).saturating_sub(1),
            ),
            (
                PaneFocus::BottomLeft,
                left_x + 1,
                mid_y + 1,
                (mid_x - left_x).saturating_sub(1),
                (bottom_y - mid_y).saturating_sub(1),
            ),
            (
                PaneFocus::BottomRight,
                mid_x + 1,
                mid_y + 1,
                (right_x - mid_x).saturating_sub(1),
                (bottom_y - mid_y).saturating_sub(1),
            ),
        ];

        // 3. Dessiner le contenu de chaque panneau
        for (i, &(pane_focus, start_x, start_y, p_width, p_height)) in
            panes_bounds.iter().enumerate()
        {
            let pane = self.panes[i];
            let is_active = self.focus == pane_focus;

            if let Some(view) = self.views.get(i)
                && let Some(node) = self.nodes.iter().find(|n| n.id == view.active_node_id)
                && node.is_file
            {
                let selection =
                    if is_active && (self.mode == Mode::Editor || self.mode == Mode::Normal) {
                        self.editor.selection
                    } else {
                        None
                    };
                let _ = self.preview(
                    node,
                    start_x,
                    start_y,
                    p_width,
                    p_height,
                    pane.cursor as usize,
                    selection,
                );
            }

            let percentage_str = if let Some(view) = self.views.get(i)
                && let Some(node) = self.nodes.iter().find(|n| n.id == view.active_node_id)
                && node.is_file
            {
                let len = node.content.len();
                if len <= 1 {
                    100
                } else {
                    ((pane.cursor as usize * 100) / (len - 1)).min(100)
                }
            } else {
                0
            };
            let expo = get_superscript(pane.view);
            let info_display = format!("{}% {}{}", percentage_str, pane.workspace, expo);
            let indicator_x = start_x + p_width.saturating_sub(info_display.len() as u16);
            let indicator_y = start_y + p_height.saturating_sub(1);
            queue!(w, MoveTo(indicator_x, indicator_y))?;
            if is_active {
                // Le panneau actif utilise notre violet cosmique
                queue!(
                    w,
                    SetForegroundColor(UI_BORDER_ACTIVE),
                    Print(format!("{}% ", percentage_str)),
                    Print(pane.workspace),
                    Print(expo)
                )?;
            } else {
                // Les panneaux inactifs reculent visuellement avec le texte muté
                queue!(
                    w,
                    SetForegroundColor(UI_TEXT_MUTED),
                    Print(format!("{}% ", percentage_str)),
                    Print(pane.workspace),
                    Print(expo)
                )?;
            }
        }

        // ==========================================
        // DESSINER DMENU / FINDER
        // ==========================================
        if self.is_finder_open() {
            self.draw_finder(w)?;
        } else if self.mode == Mode::Dmenu {
            // Modification ici pour s'aligner sur les nouvelles colonnes restreintes
            let (start_x, start_y, pane_width) = match self.focus {
                PaneFocus::TopLeft => (left_x, top_y, (mid_x - left_x)),
                PaneFocus::TopRight => (mid_x + 1, top_y, (right_x - mid_x)),
                PaneFocus::BottomLeft => (left_x, mid_y + 1, (mid_x - left_x)),
                PaneFocus::BottomRight => (mid_x + 1, mid_y + 1, (right_x - mid_x)),
            };

            let prompt = format!(" {} ", self.dmenu_input);
            let padded_prompt = format!("{:<width$}", prompt, width = pane_width as usize);

            queue!(
                w,
                MoveTo(start_x, start_y),
                SetBackgroundColor(UI_DMENU_BG),
                SetForegroundColor(UI_DMENU_BG),
                Print(padded_prompt),
                ResetColor
            )?;
        } else if self.mode == Mode::Search {
            // Affichage de la barre de recherche dans le panneau actif
            let (start_x, start_y, pane_width) = match self.focus {
                PaneFocus::TopLeft => (left_x, top_y, (mid_x - left_x)),
                PaneFocus::TopRight => (mid_x + 1, top_y, (right_x - mid_x)),
                PaneFocus::BottomLeft => (left_x, mid_y + 1, (mid_x - left_x)),
                PaneFocus::BottomRight => (mid_x + 1, mid_y + 1, (right_x - mid_x)),
            };

            let prompt = format!(" /{} ", self.search_input); // Le fameux slash de recherche
            let padded_prompt = format!("{:<width$}", prompt, width = pane_width as usize);
            queue!(
                w,
                MoveTo(start_x, start_y),
                SetBackgroundColor(UI_DMENU_BG),
                SetForegroundColor(UI_DMENU_FG),
                Print(padded_prompt),
                ResetColor
            )?;
        }

        // ==========================================
        // POSITIONNEMENT ET STYLE DU CURSEUR
        // ==========================================
        // ✨ On active le curseur pour le mode Normal ET le mode Editor
        if self.mode == Mode::Editor || self.mode == Mode::Normal {
            queue!(w, Show)?;

            // ✨ Changement de style selon le mode
            if self.mode == Mode::Editor {
                // Bloc plein quand on écrit
                queue!(w, SetCursorStyle::SteadyBlock)?;
            } else {
                // Tiret du bas quand on se déplace en Normal
                queue!(w, SetCursorStyle::SteadyUnderScore)?;
            }

            let active_bounds = panes_bounds
                .iter()
                .find(|(focus, _, _, _, _)| *focus == self.focus);

            if let Some(&(_, start_x, start_y, p_width, p_height)) = active_bounds {
                let active_pane = self.panes[self.focus as usize];
                let scroll_y = active_pane.cursor as usize;
                let line_idx = self.editor.cursor_line;
                let col_idx = self.editor.cursor_col;

                // Si le curseur est dans la partie visible de la fenêtre
                if line_idx >= scroll_y && line_idx < scroll_y + (p_height as usize) {
                    let screen_y = start_y + (line_idx - scroll_y) as u16;
                    let screen_x = start_x + (col_idx as u16).min(p_width.saturating_sub(1));

                    queue!(w, MoveTo(screen_x, screen_y))?;
                } else {
                    // Si le curseur logique sort de l'écran, on le cache pour éviter les artefacts
                    queue!(w, Hide)?;
                }
            }
        } else {
            // Dans les autres modes (Finder, Dmenu, Search), on le cache par défaut
            // (ou on pourra le gérer plus tard pour les barres de recherche !)
            queue!(w, Hide)?;
        }

        queue!(w, ResetColor)?;
        w.flush()?;
        Ok(())
    }
}
pub trait QwxFinder {
    fn previous_finder_layout(&mut self);
    fn next_finder_layout(&mut self);
}

impl QwxFinder for Qwx {
    fn previous_finder_layout(&mut self) {
        self.finder_layout = self.finder_layout.previous();
    }

    fn next_finder_layout(&mut self) {
        self.finder_layout = self.finder_layout.next();
    }
}

pub trait QwxPanel {
    fn load_active_pane_file(&mut self);
    fn active_pane_mut(&mut self) -> &mut PaneState;
}

impl QwxPanel for Qwx {
    fn load_active_pane_file(&mut self) {
        let active_idx = self.focus as usize;
        if let Some(view) = self.views.get(active_idx)
            && let Some(node) = self.nodes.get(view.active_node_id)
            && node.is_file
        {
            let full_path = self.current_dir.join(&node.name);
            if let Some(path_str) = full_path.to_str()
                && let Ok(editor) = Ji::open(path_str)
            {
                self.editor = editor;
                // ✨ On aligne le curseur de l'éditeur sur le défilement visuel du nouveau panneau !
                self.editor.cursor_line = self.panes[active_idx].cursor as usize;
                self.editor.cursor_col = 0;
            }
        }
    }

    fn active_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[self.focus as usize]
    }
}
pub fn qwx_read_lines(path: impl AsRef<Path>) -> Result<Vec<String>, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => lines.push(line),
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                continue;
            }
            Err(e) => return Err(e), // On continue de propager les autres erreurs (ex: droits d'accès)
        }
    }

    Ok(lines)
}

fn qwx_load_node(id: usize, path: &Path) -> Result<Node, Error> {
    let content = qwx_read_lines(path)?;
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();
    let is_file = path.is_file();

    let mut colored_lines = Vec::new();

    if is_file {
        // On ouvre silencieusement le fichier avec Ji pour générer l'arbre syntaxique
        if let Ok(temp_ji) = Ji::open(path) {
            let spans = temp_ji.get_colored_spans();

            // On ne peuple le cache que si Tree-sitter a vraiment renvoyé des couleurs
            if !spans.is_empty() {
                colored_lines.push(vec![]);
                for (text, color) in spans {
                    let mut is_first = true;
                    for part in text.split('\n') {
                        if !is_first {
                            colored_lines.push(vec![]);
                        }
                        if !part.is_empty() {
                            colored_lines
                                .last_mut()
                                .unwrap()
                                .push((part.to_string(), color));
                        }
                        is_first = false;
                    }
                }
            }
        }
    }

    // Fallback : Si le fichier est vide, ou qu'il n'y a pas de Tree-sitter pour lui, texte en blanc
    if colored_lines.is_empty() {
        for line in &content {
            colored_lines.push(vec![(line.clone(), Color::White)]);
        }
    }

    Ok(Node {
        id,
        name,
        content,
        colored_lines,
        is_file,
    })
}

pub struct Qwx {
    finder_layout: FinderLayout,
    finder: Finder,
    finder_recherch: String,
    nodes: Vec<Node>, // La mémoire brute
    views: Vec<View>, // Les fenêtres de défilement
    width: u16,
    height: u16,
    running: bool,
    current_dir: Box<Path>,
    focus: PaneFocus,
    panes: [PaneState; 4],
    mode: Mode,
    dmenu_input: String,
    editor: Ji,
    search_input: String,
}

pub struct QwxContext;
pub struct QwxEventResult;
// Niveau 8 : La Vue (La fenêtre de défilement)
pub struct View {
    pub active_node_id: usize,
}
pub enum QwxDirection {
    Left,
    Right,
    Down,
    Up,
    Vertical,
    Horizontal,
}

pub enum QwxScrollDirection {
    Left(u16),
    Right(u16),
    Down(u16),
    Up(u16),
    Vertical(u16),
    Horizontal(u16),
}
#[derive(Copy, Clone, PartialEq)]
pub enum PaneFocus {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
}

#[derive(Default, Clone)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub content: Vec<String>,
    pub colored_lines: Vec<Vec<(String, Color)>>,
    pub is_file: bool,
}

#[derive(Copy, Clone)]
pub struct PaneState {
    workspace: u8,
    view: u8,
    cursor: u16,
}

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Dmenu,
    Finder,
    Editor,
    Search,
}

pub trait QwxCursor<W: Write> {
    fn scroll(&mut self, w: &mut W, direction: QwxScrollDirection) -> Result<(), Error>;
    fn show(&mut self, w: &mut W) -> Result<(), Error>;
    fn hide(&mut self, w: &mut W) -> Result<(), Error>;
}
pub trait QwxRenderer<W: Write> {
    fn clear(&mut self, w: &mut W, mode: ClearType) -> Result<(), Error>;
    fn clear_screen(&mut self, w: &mut W) -> Result<(), Error>;
    fn draw_text(
        &mut self,
        w: &mut W,
        x: u16,
        y: u16,
        text: &str,
        color: Color,
    ) -> Result<(), Error>;
    fn flush(&mut self, w: &mut W) -> Result<(), Error>;
}

pub trait QwxBuffer {
    fn insert_char(&mut self, line: usize, col: usize, c: char);
    fn delete_char(&mut self, line: usize, col: usize);
    fn get_line(&self, line: usize) -> Option<&str>;
    fn len_lines(&self) -> usize;
}

impl<W: Write> QwxCursor<W> for Qwx {
    fn scroll(&mut self, w: &mut W, direction: QwxScrollDirection) -> Result<(), Error> {
        match direction {
            QwxScrollDirection::Vertical(x) => execute!(w, MoveRight(x)),
            QwxScrollDirection::Horizontal(x) => execute!(w, MoveDown(x)),
            QwxScrollDirection::Left(x) => execute!(w, MoveLeft(x)),
            QwxScrollDirection::Right(x) => execute!(w, MoveRight(x)),
            QwxScrollDirection::Down(x) => execute!(w, MoveDown(x)),
            QwxScrollDirection::Up(x) => execute!(w, MoveUp(x)),
        }
    }

    fn show(&mut self, w: &mut W) -> Result<(), Error> {
        execute!(w, Show)
    }

    fn hide(&mut self, w: &mut W) -> Result<(), Error> {
        execute!(w, Hide)
    }
}

impl<W: Write> QwxRenderer<W> for Qwx {
    fn clear_screen(&mut self, w: &mut W) -> Result<(), Error> {
        queue!(w, Clear(ClearType::All))
    }

    fn draw_text(
        &mut self,
        w: &mut W,
        x: u16,
        y: u16,
        text: &str,
        color: Color,
    ) -> Result<(), Error> {
        queue!(
            w,
            SetBackgroundColor(Color::Black),
            SetForegroundColor(color),
            MoveTo(x, y),
            Print(text),
            SetBackgroundColor(Color::Reset),
            SetForegroundColor(Color::Reset)
        )
    }

    fn flush(&mut self, w: &mut W) -> Result<(), Error> {
        w.flush()
    }

    fn clear(&mut self, w: &mut W, mode: ClearType) -> Result<(), Error> {
        queue!(w, Clear(mode))
    }
}

impl Qwx {
    fn sync_node_content(&mut self) {
        let active_idx = self.focus as usize;

        if let Some(view) = self.views.get(active_idx) {
            let node_id = view.active_node_id;

            if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                let mut new_content = Vec::new();

                // On parcourt toutes les lignes générées par ropey
                for line in self.editor.rope.lines() {
                    // On enlève les retours à la ligne de la fin, car ta fonction preview
                    // les rajoute manuellement avant de les passer à syntect

                    let clean_line = line
                        .to_string()
                        .trim_end_matches(&['\n', '\r'][..])
                        .to_string();
                    new_content.push(clean_line);
                }

                node.content = new_content;
                let mut new_colored = Vec::new();
                let spans = self.editor.get_colored_spans();

                if !spans.is_empty() {
                    new_colored.push(vec![]);
                    for (text, color) in spans {
                        let mut is_first = true;
                        for part in text.split('\n') {
                            if !is_first {
                                new_colored.push(vec![]);
                            }
                            if !part.is_empty() {
                                new_colored
                                    .last_mut()
                                    .unwrap()
                                    .push((part.to_string(), color));
                            }
                            is_first = false;
                        }
                    }
                } else {
                    // Secours en blanc au cas où l'arbre syntaxique saute pendant la frappe
                    for line in &node.content {
                        new_colored.push(vec![(line.clone(), Color::White)]);
                    }
                }
                node.colored_lines = new_colored;
            }
        }
        self.follow();
    }
    fn active_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[self.focus as usize]
    }
    pub fn follow(&mut self) {
        let cursor_line = self.editor.cursor_line;
        let mid_y = self.height / 2;
        let bottom_y = self.height.saturating_sub(1);
        let p_height = match self.focus {
            PaneFocus::TopLeft | PaneFocus::TopRight => mid_y.saturating_sub(1),
            PaneFocus::BottomLeft | PaneFocus::BottomRight => (bottom_y - mid_y).saturating_sub(1),
        } as usize;

        let pane = self.active_pane_mut();
        let scroll_y = pane.cursor as usize;

        let margin = 3.min(p_height / 3);

        // Si le curseur s'approche trop du bord HAUT de l'écran
        if cursor_line < scroll_y + margin {
            pane.cursor = cursor_line.saturating_sub(margin) as u16;
        }
        // Si le curseur s'approche trop du bord BAS de l'écran
        else if cursor_line + margin >= scroll_y + p_height {
            pane.cursor = (cursor_line + margin + 1).saturating_sub(p_height) as u16;
        }
    }
    fn handle_normal(&mut self) {
        match read().expect("failed to get terminal input") {
            Event::Key(key) => match (key.modifiers, key.code) {
                // --- CURSEUR ---
                (KeyModifiers::NONE, KeyCode::Char('j')) => {
                    // On vérifie qu'on ne dépasse pas la fin du fichier avec le curseur logique
                    if self.editor.cursor_line + 1 < self.editor.rope.len_lines() {
                        self.editor.cursor_line += 1;

                        let max_col = self
                            .editor
                            .rope
                            .line(self.editor.cursor_line)
                            .len_chars()
                            .saturating_sub(1);

                        // Sécurité pour ne pas déborder sur une ligne vide
                        self.editor.cursor_col = self.editor.cursor_col.min(max_col);
                    }
                    // C'est follow qui se charge de faire défiler le panneau si nécessaire !
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('k')) => {
                    if self.editor.cursor_line > 0 {
                        self.editor.cursor_line -= 1;

                        let max_col = self
                            .editor
                            .rope
                            .line(self.editor.cursor_line)
                            .len_chars()
                            .saturating_sub(1);

                        self.editor.cursor_col = self.editor.cursor_col.min(max_col);
                    }
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('h')) => {
                    if self.editor.cursor_col > 0 {
                        self.editor.cursor_col -= 1;
                    } else if self.editor.cursor_line > 0 {
                        self.editor.cursor_line -= 1;
                        self.editor.cursor_col = self
                            .editor
                            .rope
                            .line(self.editor.cursor_line)
                            .len_chars()
                            .saturating_sub(1);
                    }
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('l')) => {
                    let max_col = self
                        .editor
                        .rope
                        .line(self.editor.cursor_line)
                        .len_chars()
                        .saturating_sub(1);
                    if self.editor.cursor_col < max_col {
                        self.editor.cursor_col += 1;
                    } else if self.editor.cursor_line + 1 < self.editor.rope.len_lines() {
                        self.editor.cursor_line += 1;
                        self.editor.cursor_col = 0;
                    }
                    self.follow();
                }
                // --- SCROLL RAPIDE ---
                (KeyModifiers::NONE, KeyCode::PageDown) => {
                    let active_idx = self.focus as usize;
                    let node_len = if let Some(view) = self.views.get(active_idx) {
                        self.nodes
                            .get(view.active_node_id)
                            .map(|n| n.content.len())
                            .unwrap_or(0)
                    } else {
                        0
                    };
                    let step = 15;
                    let active_pane = self.active_pane_mut();
                    if (active_pane.cursor as usize) + step < node_len {
                        active_pane.cursor += step as u16;
                    } else {
                        active_pane.cursor = node_len.saturating_sub(1) as u16;
                    }
                    self.editor.cursor_line = self.active_pane_mut().cursor as usize;
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::PageUp) => {
                    let step = 15;
                    let active_pane = self.active_pane_mut();
                    active_pane.cursor = active_pane.cursor.saturating_sub(step);
                    self.editor.cursor_line = self.active_pane_mut().cursor as usize;
                    self.follow();
                }

                // --- ÉDITION RAPIDE & PRESSE-PAPIER ---
                (KeyModifiers::NONE, KeyCode::Char('x')) => {
                    self.editor.select_line();
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('d')) => {
                    if self.editor.selection.is_some() {
                        self.editor.delete_selection();
                        self.sync_node_content();
                        self.follow();
                    }
                }
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    if self.editor.selection.is_some() {
                        self.editor.selection = None;
                    }
                }

                // --- PANNEAUX (Ctrl + hjkl) ---
                (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                    self.focus = match self.focus {
                        PaneFocus::TopLeft => PaneFocus::TopRight,
                        PaneFocus::BottomLeft => PaneFocus::BottomRight,
                        _ => self.focus,
                    };
                    self.load_active_pane_file();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
                    self.focus = match self.focus {
                        PaneFocus::TopRight => PaneFocus::TopLeft,
                        PaneFocus::BottomRight => PaneFocus::BottomLeft,
                        _ => self.focus,
                    };
                    self.load_active_pane_file();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('j')) => {
                    self.focus = match self.focus {
                        PaneFocus::TopLeft => PaneFocus::BottomLeft,
                        PaneFocus::TopRight => PaneFocus::BottomRight,
                        _ => self.focus,
                    };
                    self.load_active_pane_file();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                    self.focus = match self.focus {
                        PaneFocus::BottomLeft => PaneFocus::TopLeft,
                        PaneFocus::BottomRight => PaneFocus::TopRight,
                        _ => self.focus,
                    };
                    self.load_active_pane_file();
                }

                // --- TRANSITIONS DE MODES ---
                (KeyModifiers::NONE, KeyCode::Char('o')) => {
                    let max_col = self
                        .editor
                        .rope
                        .line(self.editor.cursor_line)
                        .len_chars()
                        .saturating_sub(1);
                    self.editor.cursor_col = max_col;
                    self.editor.insert_char('\n');
                    self.sync_node_content();
                    self.follow();
                    self.mode = Mode::Editor;
                }
                (KeyModifiers::NONE, KeyCode::Char('e')) => {
                    self.mode = Mode::Editor;
                }
                (KeyModifiers::ALT, KeyCode::Char('f')) => {
                    self.mode = Mode::Finder;
                }
                (KeyModifiers::ALT, KeyCode::Char('d')) => {
                    self.mode = Mode::Dmenu;
                    self.dmenu_input.clear();
                }
                (KeyModifiers::ALT, KeyCode::Char('/')) => {
                    self.mode = Mode::Search;
                    self.search_input.clear();
                }
                (KeyModifiers::NONE, KeyCode::Char('q')) => {
                    self.running = false;
                }
                // --- Rotation Horaire (Ctrl + r) ---
                (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                    let old_panes = self.panes;
                    self.panes[1] = old_panes[0];
                    self.panes[3] = old_panes[1];
                    self.panes[2] = old_panes[3];
                    self.panes[0] = old_panes[2];

                    if self.views.len() < 4 {
                        self.views.resize_with(4, || View { active_node_id: 0 });
                    }

                    let v0 = self.views[0].active_node_id;
                    let v1 = self.views[1].active_node_id;
                    let v2 = self.views[2].active_node_id;
                    let v3 = self.views[3].active_node_id;

                    self.views[1].active_node_id = v0;
                    self.views[3].active_node_id = v1;
                    self.views[2].active_node_id = v3;
                    self.views[0].active_node_id = v2;

                    self.load_active_pane_file();
                }

                // --- Rotation Anti-Horaire (Alt + r) ---
                (KeyModifiers::ALT, KeyCode::Char('r')) => {
                    let old_panes = self.panes;
                    self.panes[2] = old_panes[0];
                    self.panes[3] = old_panes[2];
                    self.panes[1] = old_panes[3];
                    self.panes[0] = old_panes[1];

                    if self.views.len() < 4 {
                        self.views.resize_with(4, || View { active_node_id: 0 });
                    }

                    let v0 = self.views[0].active_node_id;
                    let v1 = self.views[1].active_node_id;
                    let v2 = self.views[2].active_node_id;
                    let v3 = self.views[3].active_node_id;

                    self.views[2].active_node_id = v0;
                    self.views[3].active_node_id = v2;
                    self.views[1].active_node_id = v3;
                    self.views[0].active_node_id = v1;

                    self.load_active_pane_file();
                }
                _ => {}
            },
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
            }
            _ => {}
        }
    }

    fn handle_menu(&mut self) {
        match read().expect("msg") {
            Event::Key(key) => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    self.mode = Mode::Normal;
                    self.dmenu_input.clear();
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    if let Some(cmd) = self.dmenu_input.strip_prefix('!') {
                        let cmd_clean = cmd.trim();
                        if let Some(dir_name) = cmd_clean.strip_prefix("mkdir ") {
                            let target_path = self.current_dir.join(dir_name.trim());
                            let _ = create_dir_all(&target_path);
                        } else if let Some(file_name) = cmd_clean.strip_prefix("touch ") {
                            let target_path = self.current_dir.join(file_name.trim());
                            let _ = File::create(&target_path);
                        } else {
                            let _ = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(cmd_clean)
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .status();

                            for node in self.nodes.iter_mut() {
                                if node.is_file {
                                    let full_path = self.current_dir.join(&node.name);
                                    if let Ok(fresh_node) = qwx_load_node(node.id, &full_path) {
                                        node.content = fresh_node.content;
                                        node.colored_lines = fresh_node.colored_lines;
                                    }
                                }
                            }
                        }
                    }
                    self.mode = Mode::Normal;
                    self.dmenu_input.clear();
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    self.dmenu_input.pop();
                }
                (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                    self.dmenu_input.push(c);
                }
                _ => {}
            },
            Event::Paste(x) => {
                self.dmenu_input.push_str(x.as_str());
            }
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
            }
            _ => {}
        }
    }

    fn handle_editor(&mut self) {
        match read().expect("msg") {
            Event::Key(key) => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    if self.editor.selection.is_some() {
                        self.editor.selection = None;
                    } else {
                        self.mode = Mode::Normal;
                    }
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    self.editor.insert_char('\n');
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Delete) => {
                    self.editor.delete();
                    self.sync_node_content();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                    if self.editor.save().is_err() {
                        eprintln!("Erreur lors de la sauvegarde");
                    }
                }
                (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                    let current_line = self.editor.cursor_line;
                    if current_line < self.editor.rope.len_lines() {
                        let chars_to_delete = self.editor.rope.line(current_line).len_chars();
                        self.editor.cursor_col = 0;
                        for _ in 0..chars_to_delete {
                            self.editor.delete();
                        }
                        if current_line >= self.editor.rope.len_lines() && current_line > 0 {
                            self.editor.cursor_line -= 1;
                        }
                        self.sync_node_content();
                    }
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    self.editor.backspace();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::ALT, KeyCode::Char('x')) => {
                    self.editor.select_line();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::ALT, KeyCode::Char('d')) => {
                    if self.editor.selection.is_some() {
                        self.editor.delete_selection();
                    }
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Tab) => {
                    for _ in 0..4 {
                        self.editor.insert_char(' ');
                    }
                    self.sync_node_content();
                }
                (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                    self.editor.insert_char(c);
                    self.sync_node_content();
                    self.follow();
                }
                _ => {}
            },
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
            }
            _ => {}
        }
    }

    fn handle_finder(&mut self) {
        match read().expect("msg") {
            Event::Key(key) => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    self.mode = Mode::Normal;
                    self.finder_recherch.clear();
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    self.finder_recherch.pop();
                    self.finder.filter(self.finder_recherch.clone());
                }
                (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                    self.finder_recherch.push(c);
                    self.finder.filter(self.finder_recherch.clone());
                }
                (KeyModifiers::CONTROL, KeyCode::Char('j')) => {
                    self.finder.next_file();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                    self.finder.prev_file();
                }
                (KeyModifiers::ALT, KeyCode::Char('j')) => {
                    self.finder.next_dir();
                }
                (KeyModifiers::ALT, KeyCode::Char('k')) => {
                    self.finder.prev_dir();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
                    if let Some(parent) = self.current_dir.parent() {
                        self.current_dir = parent.into();
                        self.finder = Finder::new(&self.current_dir, self.finder_layout.clone());
                        self.finder_recherch.clear();
                    }
                }
                (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                    let dirs = self.finder.get_directories();
                    if !dirs.is_empty() && self.finder.selected_dir < dirs.len() {
                        let dirname = &dirs[self.finder.selected_dir];
                        let new_path = self.current_dir.join(dirname);
                        self.current_dir = new_path.clone().into();
                        self.finder = Finder::new(&new_path, self.finder_layout.clone());
                        self.finder_recherch.clear();
                    }
                }
                (m, KeyCode::Char('j'))
                    if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) =>
                {
                    self.finder.next_sub_dir();
                }
                (m, KeyCode::Char('k'))
                    if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) =>
                {
                    self.finder.prev_sub_dir();
                }
                (m, KeyCode::Char('l'))
                    if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) =>
                {
                    let sub_dirs = self.finder.get_sub_directories();
                    if !sub_dirs.is_empty() && self.finder.selected_sub_dir < sub_dirs.len() {
                        let dirname = &sub_dirs[self.finder.selected_sub_dir];
                        let new_path = self.current_dir.join(dirname);
                        self.current_dir = new_path.clone().into();
                        self.finder = Finder::new(&new_path, self.finder_layout.clone());
                        self.finder_recherch.clear();
                    }
                }
                (KeyModifiers::META, KeyCode::Char('h')) => {
                    self.previous_finder_layout();
                }
                (KeyModifiers::META, KeyCode::Char('l')) => {
                    self.next_finder_layout();
                }
                (KeyModifiers::NONE, KeyCode::F(5)) => {
                    self.finder = Finder::new(Path::new("."), self.finder_layout.clone());
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    let files = self.finder.get_files();
                    if !files.is_empty() && self.finder.selected_file < files.len() {
                        let filename = &files[self.finder.selected_file];
                        let full_path = self.current_dir.join(filename);

                        let node_id = if let Some(existing_node) =
                            self.nodes.iter().find(|n| n.name == *filename)
                        {
                            existing_node.id
                        } else {
                            let new_id = self.nodes.len();
                            if let Ok(node) = qwx_load_node(new_id, &full_path) {
                                self.nodes.push(node);
                                new_id
                            } else {
                                self.finder_recherch.clear();
                                return;
                            }
                        };
                        let active_idx = self.focus as usize;

                        if self.views.len() <= active_idx {
                            self.views
                                .resize_with(active_idx + 1, || View { active_node_id: 0 });
                        }

                        if let Some(view) = self.views.get_mut(active_idx) {
                            view.active_node_id = node_id;
                        }

                        self.panes[active_idx].cursor = 0;
                        if let Some(path_str) = full_path.to_str()
                            && let Ok(editor) = Ji::open(path_str)
                        {
                            self.editor = editor;
                        }
                    }
                    self.mode = Mode::Normal;
                    self.finder_recherch.clear();
                }
                _ => {}
            },
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
                self.finder.resize(cols, rows);
            }
            _ => {}
        }
    }

    fn handle_search(&mut self) {
        match read().expect("msg") {
            Event::Key(key) => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    self.mode = Mode::Normal;
                    self.search_input.clear();
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    if let Ok(re) = regex::Regex::new(&self.search_input) {
                        let text = self.editor.rope.to_string();
                        let cursor_char = self.editor.rope.line_to_char(self.editor.cursor_line)
                            + self.editor.cursor_col;
                        let cursor_byte = self.editor.rope.char_to_byte(cursor_char);

                        let found = re.find_at(&text, cursor_byte).or_else(|| re.find(&text));

                        if let Some(m) = found {
                            let match_char = self.editor.rope.byte_to_char(m.start());
                            self.editor.cursor_line = self.editor.rope.char_to_line(match_char);
                            self.editor.cursor_col =
                                match_char - self.editor.rope.line_to_char(self.editor.cursor_line);
                            self.active_pane_mut().cursor = self.editor.cursor_line as u16;
                        }
                    }
                    self.mode = Mode::Normal;
                    self.search_input.clear();
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    self.search_input.pop();
                }
                (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                    self.search_input.push(c);
                }
                _ => {}
            },
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
            }
            _ => {}
        }
    }
    fn handle_events(&mut self) {
        match self.mode {
            Mode::Normal => self.handle_normal(),
            Mode::Finder => self.handle_finder(),
            Mode::Dmenu => self.handle_menu(),
            Mode::Editor => self.handle_editor(),
            Mode::Search => self.handle_search(),
        }
    }
    pub fn is_finder_open(&mut self) -> bool {
        self.mode == Mode::Finder
    }
    pub fn run(&mut self) -> Result<(), Error> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        queue!(stdout, Clear(ClearType::All))?;
        while self.running {
            self.draw(&mut stdout)?;
            self.handle_events();
        }
        // Nettoyage en quittant
        execute!(stdout, LeaveAlternateScreen, Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn preview(
        &self,
        node: &Node,
        start_x: u16,
        start_y: u16,
        p_width: u16,
        p_height: u16,
        scroll_y: usize,
        selection: Option<(usize, usize)>, // N'oublie pas le nouveau paramètre !
    ) -> Result<(), Error> {
        let mut w = stdout();
        let mut drawn_lines = 0;

        for (line_idx, line_spans) in node
            .colored_lines
            .iter()
            .skip(scroll_y)
            .take(p_height as usize)
            .enumerate()
        {
            queue!(w, MoveTo(start_x, start_y + line_idx as u16))?;

            let current_absolute_line = scroll_y + line_idx;
            let is_selected = match selection {
                Some((start, end)) => {
                    current_absolute_line >= start && current_absolute_line <= end
                }
                None => false,
            };

            if is_selected {
                queue!(
                    w,
                    SetBackgroundColor(Color::Rgb {
                        r: 55,
                        g: 65,
                        b: 85
                    })
                )?;
            }

            let mut current_width = 0;
            for (text, color) in line_spans {
                let clean_text = text.replace('\t', "    ").replace('\r', "");
                let text_width = clean_text.width();
                let remaining_width = p_width.saturating_sub(current_width) as usize;

                if remaining_width == 0 {
                    break;
                }

                let display_text = if text_width > remaining_width {
                    let mut acc_width = 0;
                    let mut truncated = String::new();
                    for c in clean_text.chars() {
                        let c_width = c.width().unwrap_or(0);
                        if acc_width + c_width > remaining_width {
                            break;
                        }
                        truncated.push(c);
                        acc_width += c_width;
                    }
                    truncated
                } else {
                    clean_text
                };

                queue!(w, SetForegroundColor(*color), Print(&display_text))?;
                current_width += display_text.width() as u16;
            }

            if current_width < p_width {
                let padding = " ".repeat((p_width - current_width) as usize);
                queue!(w, Print(padding))?;
            }

            queue!(w, ResetColor)?;
            drawn_lines += 1;
        }

        // Nettoyer les lignes restantes en bas du panneau (si fin de fichier)
        for empty_y in drawn_lines..(p_height as usize) {
            let padding = " ".repeat(p_width as usize);
            queue!(
                w,
                MoveTo(start_x, start_y + empty_y as u16),
                ResetColor,
                Print(padding)
            )?;
        }
        Ok(())
    }
    
    pub fn new(path: &Path, open_mode: Mode) -> Result<Self, Error> {
        let (width, height) = size()?;
        let mut nodes: Vec<Node> = Vec::new();
        let mut views: Vec<View> = Vec::new();
        for (i, filename) in list_files(path).iter().enumerate() {
            if let Ok(node) = qwx_load_node(i, PathBuf::from(filename).as_path()) {
                nodes.push(node);
                views.push(View { active_node_id: i });
            }
        }
        Ok(Self {
            width,
            height,
            running: true,
            focus: PaneFocus::TopLeft,
            panes: [
                INIT_PANE_STATE,
                INIT_PANE_STATE,
                INIT_PANE_STATE,
                INIT_PANE_STATE,
            ],
            mode: open_mode,
            dmenu_input: String::new(),
            nodes: nodes.clone(),
            views,
            finder_layout: FinderLayout::Grid,
            finder_recherch: String::new(),
            finder: Finder::new(path, FinderLayout::Grid),
            current_dir: path.into(),
            editor: Ji::default(),
            search_input: String::new(),
        })
    }
    pub fn draw_finder<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        // On recalcule la zone centrale
        let max_width = 180.min(self.width);
        let left_x = (self.width.saturating_sub(max_width)) / 2;

        // On envoie left_x, 0 (pour top_y), max_width et la hauteur totale au Finder
        self.finder.draw(
            w,
            self.finder_recherch.clone(),
            left_x,
            0,
            max_width,
            self.height,
        )
    }
}
pub fn create_config(
    scope: &str,
    lang: Language,
    query: &'static str,
    theme_keys: &[&'static str],
) -> Option<LangConfig> {
    let mut ts_config = HighlightConfiguration::new(lang, scope, query, "", "").ok()?;
    ts_config.configure(theme_keys);
    Some(LangConfig {
        ts_config,
        query_string: query,
    })
}

/// Associe une extension de fichier à sa configuration Tree-sitter correspondante.
fn detect_langage(extension: &str, theme_keys: &[&'static str]) -> Option<LangConfig> {
    match extension {
        "ada" | "adb" => create_config(
            "ada",
            Language::from(tree_sitter_ada::LANGUAGE),
            "",
            theme_keys,
        ),
        "ps1" | "psm1" | "psd1" => create_config(
            "powershell",
            Language::from(tree_sitter_powershell::LANGUAGE),
            tree_sitter_powershell::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "scss" | "sass" => create_config(
            "scss",
            Language::from(tree_sitter_sas::LANGUAGE),
            tree_sitter_sas::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "Kconfig" => create_config(
            "kconfig",
            Language::from(tree_sitter_kconfig::LANGUAGE),
            tree_sitter_kconfig::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "vhdl" => create_config(
            "vhdl",
            Language::from(tree_sitter_vhdl::LANGUAGE),
            tree_sitter_vhdl::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "jinja2" => create_config(
            "jinja2",
            Language::from(tree_sitter_jinja2::LANGUAGE),
            tree_sitter_jinja2::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "nginx" => create_config(
            "nginx",
            Language::from(tree_sitter_nginx::LANGUAGE),
            "",
            theme_keys,
        ),
        "zsh" => create_config(
            "zsh",
            Language::from(tree_sitter_zsh::LANGUAGE),
            tree_sitter_zsh::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "md" => create_config(
            "md",
            Language::from(tree_sitter_md::LANGUAGE),
            "",
            theme_keys,
        ),
        "agda" => create_config(
            "agda",
            Language::from(tree_sitter_agda::LANGUAGE),
            tree_sitter_agda::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "asm" | "s" => create_config(
            "asm",
            Language::from(tree_sitter_asm::LANGUAGE),
            tree_sitter_asm::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "sh" | "bash" => create_config(
            "bash",
            Language::from(tree_sitter_bash::LANGUAGE),
            tree_sitter_bash::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "bat" | "cmd" => create_config(
            "batch",
            Language::from(tree_sitter_batch::LANGUAGE),
            tree_sitter_batch::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "c" | "h" => create_config(
            "c",
            Language::from(tree_sitter_c::LANGUAGE),
            tree_sitter_c::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "cs" => create_config(
            "c_sharp",
            Language::from(tree_sitter_c_sharp::LANGUAGE),
            tree_sitter_c_sharp::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "cmake" => create_config(
            "cmake",
            Language::from(tree_sitter_cmake::LANGUAGE),
            tree_sitter_cmake::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "cpp" | "cc" | "cxx" | "hpp" => create_config(
            "cpp",
            Language::from(tree_sitter_cpp::LANGUAGE),
            tree_sitter_cpp::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "css" => create_config(
            "css",
            Language::from(tree_sitter_css::LANGUAGE),
            tree_sitter_css::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "d" => create_config("d", Language::from(tree_sitter_d::LANGUAGE), "", theme_keys),
        "dart" => create_config(
            "dart",
            Language::from(tree_sitter_dart::LANGUAGE),
            tree_sitter_dart::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "diff" | "patch" => create_config(
            "diff",
            Language::from(tree_sitter_diff::LANGUAGE),
            "",
            theme_keys,
        ),
        "ex" | "exs" => create_config(
            "elixir",
            Language::from(tree_sitter_elixir::LANGUAGE),
            tree_sitter_elixir::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "elm" => create_config(
            "elm",
            Language::from(tree_sitter_elm::LANGUAGE),
            tree_sitter_elm::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "erl" | "hrl" => create_config(
            "erlang",
            Language::from(tree_sitter_erlang::LANGUAGE),
            tree_sitter_erlang::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "fish" => create_config(
            "fish",
            tree_sitter_fish::language(),
            tree_sitter_fish::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "fs" | "fsi" | "fsx" => create_config(
            "fsharp",
            Language::from(tree_sitter_fsharp::LANGUAGE_FSHARP),
            tree_sitter_fsharp::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "glsl" | "vert" | "frag" => create_config(
            "glsl",
            Language::from(tree_sitter_glsl::LANGUAGE_GLSL),
            "",
            theme_keys,
        ),
        "go" => create_config(
            "go",
            Language::from(tree_sitter_go::LANGUAGE),
            tree_sitter_go::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "gql" | "graphql" => create_config(
            "graphql",
            Language::from(tree_sitter_graphql::LANGUAGE),
            "",
            theme_keys,
        ),
        "hs" => create_config(
            "haskell",
            Language::from(tree_sitter_haskell::LANGUAGE),
            tree_sitter_haskell::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "hcl" | "tf" => create_config(
            "hcl",
            Language::from(tree_sitter_hcl::LANGUAGE),
            "",
            theme_keys,
        ),
        "hlsl" => create_config(
            "hlsl",
            Language::from(tree_sitter_hlsl::LANGUAGE_HLSL),
            "",
            theme_keys,
        ),
        "html" | "htm" => create_config(
            "html",
            Language::from(tree_sitter_html::LANGUAGE),
            tree_sitter_html::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "ini" => create_config(
            "ini",
            Language::from(tree_sitter_ini::LANGUAGE),
            tree_sitter_ini::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "java" => create_config(
            "java",
            Language::from(tree_sitter_java::LANGUAGE),
            tree_sitter_java::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "js" | "mjs" | "cjs" => create_config(
            "javascript",
            Language::from(tree_sitter_javascript::LANGUAGE),
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "json" => create_config(
            "json",
            Language::from(tree_sitter_json::LANGUAGE),
            tree_sitter_json::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "lua" => create_config(
            "lua",
            Language::from(tree_sitter_lua::LANGUAGE),
            tree_sitter_lua::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "make" | "makefile" | "Makefile" => create_config(
            "make",
            Language::from(tree_sitter_make::LANGUAGE),
            tree_sitter_make::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "nix" => create_config(
            "nix",
            Language::from(tree_sitter_nix::LANGUAGE),
            tree_sitter_nix::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "m" => create_config(
            "objc",
            Language::from(tree_sitter_objc::LANGUAGE),
            tree_sitter_objc::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "odin" => create_config(
            "odin",
            Language::from(tree_sitter_odin::LANGUAGE),
            tree_sitter_odin::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "pl" | "pm" => create_config(
            "perl",
            Language::from(tree_sitter_perl::LANGUAGE),
            "",
            theme_keys,
        ),
        "php" => create_config(
            "php",
            Language::from(tree_sitter_php::LANGUAGE_PHP),
            tree_sitter_php::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "py" | "pyw" => create_config(
            "python",
            Language::from(tree_sitter_python::LANGUAGE),
            tree_sitter_python::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "r" => create_config(
            "r",
            Language::from(tree_sitter_r::LANGUAGE),
            tree_sitter_r::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "rb" => create_config(
            "ruby",
            Language::from(tree_sitter_ruby::LANGUAGE),
            tree_sitter_ruby::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "rs" => create_config(
            "rust",
            Language::from(tree_sitter_rust::LANGUAGE),
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "scala" | "sc" => create_config(
            "scala",
            Language::from(tree_sitter_scala::LANGUAGE),
            tree_sitter_scala::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "swift" => create_config(
            "swift",
            Language::from(tree_sitter_swift::LANGUAGE),
            tree_sitter_swift::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "ts" | "mts" | "cts" => create_config(
            "typescript",
            Language::from(tree_sitter_typescript::LANGUAGE_TYPESCRIPT),
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "xml" | "xsd" => create_config(
            "xml",
            Language::from(tree_sitter_xml::LANGUAGE_XML),
            tree_sitter_xml::XML_HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "yaml" | "yml" => create_config(
            "yaml",
            Language::from(tree_sitter_yaml::LANGUAGE),
            tree_sitter_yaml::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "zig" => create_config(
            "zig",
            Language::from(tree_sitter_zig::LANGUAGE),
            tree_sitter_zig::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        _ => None, // Extension inconnue
    }
}

/// Représente la configuration de coloration pour un langage spécifique
pub struct LangConfig {
    pub ts_config: HighlightConfiguration,
    pub query_string: &'static str,
}
#[derive(Default)]
pub struct Ji {
    pub rope: Rope,
    pub file_path: Option<PathBuf>,
    pub query: Option<Query>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub parser: Parser,
    pub syntax_tree: Option<Tree>,
    pub lang_config: Option<LangConfig>,
    pub selection: Option<(usize, usize)>,
}

impl Ji {
    pub fn select_line(&mut self) {
        if let Some((start, end)) = self.selection {
            if end + 1 < self.rope.len_lines() {
                self.selection = Some((start, end + 1));
                self.cursor_line = end + 1; // Le curseur descend visuellement
            }
        } else {
            self.selection = Some((self.cursor_line, self.cursor_line));
        }
    }

    /// Mimétisme de Helix 'd' : Supprime toutes les lignes sélectionnées
    pub fn delete_selection(&mut self) {
        if let Some((start, end)) = self.selection {
            let start_char = self.rope.line_to_char(start);
            let end_char = if end + 1 < self.rope.len_lines() {
                self.rope.line_to_char(end + 1)
            } else {
                self.rope.len_chars()
            };

            let start_byte = self.rope.char_to_byte(start_char);
            let end_byte = self.rope.char_to_byte(end_char);

            // Mise à jour chirurgicale de Tree-sitter pour ne pas casser la coloration
            if let Some(ref mut tree) = self.syntax_tree {
                let edit = InputEdit {
                    start_byte,
                    old_end_byte: end_byte,
                    new_end_byte: start_byte,
                    start_position: Point::new(start, 0),
                    old_end_position: Point::new(end + 1, 0),
                    new_end_position: Point::new(start, 0),
                };
                tree.edit(&edit);
            }

            self.rope.remove(start_char..end_char);
            self.cursor_line = start;
            self.cursor_col = 0;
            self.selection = None;
            self.update_syntax_tree();
        }
    }
    /// Supprime le caractère situé sous le curseur (Touche Suppr)
    pub fn delete(&mut self) {
        // 1. Calculer l'index absolu du curseur
        let cursor_char_idx = self.rope.line_to_char(self.cursor_line) + self.cursor_col;

        // Si on est à la toute fin du fichier, il n'y a rien à supprimer
        if cursor_char_idx >= self.rope.len_chars() {
            return;
        }

        // 2. Identifier le caractère ciblé (exactement sous le curseur)
        let target_char = self.rope.char(cursor_char_idx);
        let char_len_bytes = target_char.len_utf8();
        let byte_idx = self.rope.char_to_byte(cursor_char_idx);

        // 3. Déterminer les positions graphiques pour Tree-sitter
        let start_point = Point::new(self.cursor_line, self.cursor_col);

        let mut old_end_point = start_point;
        if target_char == '\n' {
            old_end_point.row += 1;
            old_end_point.column = 0;
        } else {
            old_end_point.column += char_len_bytes;
        }

        // 4. Notifier l'arbre syntaxique de la suppression
        if let Some(ref mut tree) = self.syntax_tree {
            let edit = InputEdit {
                start_byte: byte_idx,
                old_end_byte: byte_idx + char_len_bytes,
                new_end_byte: byte_idx,
                start_position: start_point,
                old_end_position: old_end_point,
                new_end_position: start_point, // Le curseur ne bouge pas
            };
            tree.edit(&edit);
        }

        // 5. Supprimer le caractère dans la structure Rope
        self.rope.remove(cursor_char_idx..(cursor_char_idx + 1));

        // 6. Mettre à jour l'arbre syntaxique
        self.update_syntax_tree();
    }
    /// Sauvegarde le contenu de l'éditeur dans le fichier d'origine
    pub fn save(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.file_path {
            // Création ou écrasement du fichier
            let file = File::create(path)?;

            // Utilisation d'un BufWriter pour une écriture disque performante
            let writer = std::io::BufWriter::new(file);

            // Ropey possède une méthode hyper optimisée pour s'écrire dans un flux
            self.rope.write_to(writer)?;
        }
        Ok(())
    }

    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)?;
        let rope = Rope::from_reader(BufReader::new(file))?;

        // 1. Votre catalogue de tokens utilisé par votre gestionnaire de thème
        let theme_keys = vec![
            "keyword",
            "keyword.function",
            "keyword.return",
            "keyword.operator",
            "function",
            "function.macro",
            "function.method",
            "method",
            "string",
            "string_literal",
            "character",
            "number",
            "integer",
            "float",
            "boolean",
            "comment",
            "line_comment",
            "block_comment",
            "type",
            "primitive_type",
            "type.builtin",
            "operator",
            "punctuation.bracket",
            "punctuation.delimiter",
            "variable",
            "variable.parameter",
            "variable.builtin",
            "property",
            "attribute",
            "label",
            "constant",
            "constant.builtin",
            "constant.character.escape",
            "namespace",
            "keyword.directive",
            "punctuation.special",
        ];

        let filename = path_ref.file_name().expect("");
        let ext = path_ref.extension().unwrap_or(filename);

        // ✨ 2. On initialise l'éditeur avec le texte et le chemin dans TOUS LES CAS
        let mut ji = Self {
            rope,
            file_path: Some(path_ref.to_path_buf()),
            cursor_line: 0,
            cursor_col: 0,
            parser: Parser::new(),
            syntax_tree: None,
            lang_config: None,
            query: None,
            selection: None,
        };

        // ✨ 3. On tente d'appliquer la surcouche Tree-sitter si le langage est connu
        if let Some(config) = detect_langage(ext.to_str().expect(""), &theme_keys) {
            ji.query = Query::new(&config.ts_config.language, config.query_string).ok();
            let _ = ji.parser.set_language(&config.ts_config.language);
            ji.lang_config = Some(config);

            // On génère l'arbre syntaxique
            ji.update_syntax_tree();
        }

        // 4. On retourne l'éditeur (avec ou sans coloration, mais toujours avec le bon texte !)
        Ok(ji)
    }
    /// Insère un caractère à la position actuelle du curseur (ligne, col)
    pub fn insert_char(&mut self, ch: char) {
        // 1. Calculer l'index absolu en caractères et en octets (bytes)
        let char_idx = self.rope.line_to_char(self.cursor_line) + self.cursor_col;
        let byte_idx = self.rope.char_to_byte(char_idx);

        // 2. Définir les coordonnées graphiques de départ
        let start_point = Point::new(self.cursor_line, self.cursor_col);

        // 3. Calculer les nouvelles coordonnées graphiques après l'insertion
        let mut new_end_point = start_point;
        if ch == '\n' {
            new_end_point.row += 1;
            new_end_point.column = 0;
        } else {
            new_end_point.column += ch.len_utf8();
        }

        // 4. Notifier l'arbre syntaxique du changement (si un arbre existe)
        if let Some(ref mut tree) = self.syntax_tree {
            let edit = InputEdit {
                start_byte: byte_idx,
                old_end_byte: byte_idx,
                new_end_byte: byte_idx + ch.len_utf8(),
                start_position: start_point,
                old_end_position: start_point,
                new_end_position: new_end_point,
            };
            tree.edit(&edit); // Ajuste les index de l'arbre de manière chirurgicale
        }

        // 5. Insérer réellement le caractère dans la Rope
        self.rope.insert_char(char_idx, ch);

        // 6. Mettre à jour la position du curseur
        if ch == '\n' {
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            self.cursor_col += 1;
        }

        // 7. Relancer le parsing incrémental ultra-rapide
        self.update_syntax_tree();
    }

    /// Supprime le caractère situé juste avant le curseur (Retour arrière)
    pub fn backspace(&mut self) {
        // Si on est tout au début du fichier, on ne peut rien supprimer
        if self.cursor_line == 0 && self.cursor_col == 0 {
            return;
        }

        // 1. Déterminer la position du caractère à supprimer (juste avant le curseur)
        let cursor_char_idx = self.rope.line_to_char(self.cursor_line) + self.cursor_col;
        let target_char_idx = cursor_char_idx - 1;

        let target_char = self.rope.char(target_char_idx);
        let char_len_bytes = target_char.len_utf8();
        let byte_idx = self.rope.char_to_byte(target_char_idx);

        // 2. Déterminer les anciennes et nouvelles positions du curseur graphique
        let old_end_point = Point::new(self.cursor_line, self.cursor_col);
        let mut start_point = old_end_point;

        if target_char == '\n' {
            // Si on supprime un retour à la ligne, le curseur remonte à la ligne précédente
            start_point.row -= 1;
            // On se place à la fin de cette ligne précédente (avant la fusion des lignes)
            start_point.column = self.rope.line(start_point.row).len_chars() - 1;
        } else {
            start_point.column -= 1;
        }

        // 3. Notifier l'arbre syntaxique de la suppression
        if let Some(ref mut tree) = self.syntax_tree {
            let edit = InputEdit {
                start_byte: byte_idx,
                old_end_byte: byte_idx + char_len_bytes,
                new_end_byte: byte_idx,
                start_position: start_point,
                old_end_position: old_end_point,
                new_end_position: start_point,
            };
            tree.edit(&edit);
        }

        // 4. Supprimer le caractère dans la Rope
        self.rope.remove(target_char_idx..cursor_char_idx);

        // 5. Déplacer le curseur physique vers sa nouvelle position
        self.cursor_line = start_point.row;
        self.cursor_col = start_point.column;

        // 6. Mettre à jour l'arbre syntaxique
        self.update_syntax_tree();
    }

    pub fn update_syntax_tree(&mut self) {
        // Si aucun langage n'est configuré, impossible de générer un arbre.
        if self.lang_config.is_none() {
            return;
        }

        // On crée une référence locale à la Rope pour la closure
        let rope = &self.rope;

        // Appel de parse_with_options (ou parse) en utilisant le parseur interne de Ji
        // tree-sitter demande des morceaux d'octets au fur et à mesure de ses besoins.
        let tree = self.parser.parse_with_options(
            &mut |byte_offset, _position| {
                if byte_offset < rope.len_bytes() {
                    // Ropey trouve instantanément le bloc de texte ("chunk") contenant cet octet
                    let (chunk, chunk_byte_idx, _, _) = rope.chunk_at_byte(byte_offset);
                    // On renvoie la tranche exacte d'octets demandée par le parseur
                    &chunk.as_bytes()[byte_offset - chunk_byte_idx..]
                } else {
                    // Fin du texte atteinte, on renvoie une tranche vide
                    &[] as &[u8]
                }
            },
            self.syntax_tree.as_ref(), // Fournit l'ancien arbre pour permettre le calcul incrémental
            None,                      // Pas d'options de parsing spécifiques nécessaires
        );

        // On sauvegarde le nouvel arbre mis à jour
        self.syntax_tree = tree;
    }
    /// Retourne une liste de segments textuels (String) associés à leur couleur Crossterm,
    /// couvrant l'intégralité du document de manière continue.
    pub fn get_colored_spans(&self) -> Vec<(String, crossterm::style::Color)> {
        let mut spans = Vec::new();
        let total_bytes = self.rope.len_bytes();
        if total_bytes == 0 {
            return spans;
        }

        // 1. Récupérer toutes les captures brutes
        let mut raw_highlights = Vec::new();
        if let (Some(tree), Some(query)) = (&self.syntax_tree, &self.query) {
            let mut cursor = QueryCursor::new();
            let text_bytes = self.rope.to_string().into_bytes();
            let mut matches = cursor.matches(query, tree.root_node(), text_bytes.as_slice());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let start = capture.node.start_byte();
                    let end = capture.node.end_byte();
                    let name = &query.capture_names()[capture.index as usize];
                    raw_highlights.push((start, end, name.to_string()));
                }
            }
        }

        // 2. Trier les captures : par début croissant, puis par fin décroissante (les plus larges d'abord)
        raw_highlights.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));

        // 3. Linéariser les captures pour éviter les chevauchements
        let mut current_byte = 0;
        let text_string = self.rope.to_string();
        let text_bytes = text_string.as_bytes();

        for (start, end, name) in raw_highlights {
            // Ignorer les captures obsolètes ou déjà dépassées (imbriquées)
            if start < current_byte {
                continue;
            }

            // S'il y a un trou entre la position actuelle et le début de la capture,
            // on ajoute du texte avec la couleur par défaut.
            if start > current_byte {
                if let Ok(text_slice) = std::str::from_utf8(&text_bytes[current_byte..start]) {
                    spans.push((text_slice.to_string(), theme::FG_DEFAULT));
                }
                current_byte = start;
            }

            // Ajouter la zone colorée
            if let Ok(text_slice) = std::str::from_utf8(&text_bytes[start..end]) {
                let color = get_color_for_capture(&name);
                spans.push((text_slice.to_string(), color));
                current_byte = end;
            }
        }

        // Ajouter le reste du fichier s'il reste du texte non coloré à la fin
        if current_byte < total_bytes
            && let Ok(text_slice) = std::str::from_utf8(&text_bytes[current_byte..total_bytes])
        {
            spans.push((text_slice.to_string(), theme::FG_DEFAULT));
        }
        spans
    }
}
