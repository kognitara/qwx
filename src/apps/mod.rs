use crate::{
    editor::{
        Ji,
        theme::{UI_BORDER_ACTIVE, UI_BORDER_INACTIVE, UI_DMENU_BG, UI_DMENU_FG, UI_TEXT_MUTED},
    },
    finder::{Finder, FinderLayout, list_files},
};
use crossterm::{
    cursor::{self, Hide, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::Write;
use std::{
    fs::File,
    fs::create_dir_all,
    io::{BufRead, BufReader, Result, stdout},
    path::{Path, PathBuf},
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Crée un répertoire et tous ses parents s'ils n'existent pas
pub fn create_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    create_dir_all(path)
}

/// Crée un fichier vide. Si les dossiers parents n'existent pas,
/// ils seront créés automatiquement pour éviter un crash.
pub fn create_empty_file<P: AsRef<Path>>(path: P) -> Result<()> {
    let path = path.as_ref();

    // On vérifie s'il y a des dossiers parents dans le chemin saisi
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        // On crée toute la hiérarchie parente nécessaire
        create_dir_all(parent)?;
    }
    // On crée le fichier (et on le referme immédiatement)
    File::create(path)?;
    Ok(())
}
#[derive(Default, Clone)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub content: Vec<String>,
    pub colored_lines: Vec<Vec<(String, Color)>>,
    pub is_file: bool,
}

// Niveau 8 : La Vue (La fenêtre de défilement)
pub struct View {
    pub active_node_id: usize,
}

#[derive(Copy, Clone, PartialEq)]
enum PaneFocus {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
}
#[derive(PartialEq)]
enum Mode {
    Normal,
    Dmenu,
    Finder,
    Editor,
}
#[derive(Copy, Clone)]
struct PaneState {
    workspace: u8,
    view: u8,
    cursor: u16,
}

pub struct App {
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
}

fn get_superscript(num: u8) -> &'static str {
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

fn read_lines(path: impl AsRef<Path>) -> Result<Vec<String>> {
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

fn load_node(id: usize, path: &Path) -> Result<Node> {
    let content = read_lines(path)?;
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

impl App {
    fn active_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[self.focus as usize]
    }

    pub fn new(path: &Path) -> Result<Self> {
        let (width, height) = terminal::size()?;
        let mut nodes: Vec<Node> = Vec::new();
        let mut views: Vec<View> = Vec::new();
        for (i, filename) in list_files(path).iter().enumerate() {
            if let Ok(node) = load_node(i, PathBuf::from(filename).as_path()) {
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
                PaneState {
                    workspace: 1,
                    view: 1,
                    cursor: 0,
                },
                PaneState {
                    workspace: 2,
                    view: 1,
                    cursor: 0,
                },
                PaneState {
                    workspace: 3,
                    view: 1,
                    cursor: 0,
                },
                PaneState {
                    workspace: 4,
                    view: 1,
                    cursor: 0,
                },
            ],
            mode: Mode::Normal,
            dmenu_input: String::new(),
            nodes: nodes.clone(),
            views,
            finder_layout: FinderLayout::Grid,
            finder_recherch: String::new(),
            finder: Finder::new(path, FinderLayout::Grid),
            current_dir: path.into(),
            editor: Ji::default(),
        })
    }
    /// Synchronise le Rope de l'éditeur vers le Vec<String> du Nœud actif
    /// pour que l'affichage et syntect soient mis à jour en temps réel.
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
    }
    fn draw_finder<W: Write>(&mut self, w: &mut W) -> Result<()> {
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
    pub fn is_finder_open(&mut self) -> bool {
        self.mode == Mode::Finder
    }
    /// Boucle principale de l'application
    pub fn run(&mut self) -> Result<()> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        queue!(stdout, Clear(ClearType::All))?;
        while self.running {
            self.draw(&mut stdout)?;
            self.handle_events(&mut stdout)?;
        }
        // Nettoyage en quittant
        execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
    pub fn next_finder_layout(&mut self) {
        self.finder_layout = self.finder_layout.next();
    }
    pub fn previous_finder_layout(&mut self) {
        self.finder_layout = self.finder_layout.previous();
    }
    pub fn preview(
        &self,
        node: &Node,
        start_x: u16,
        start_y: u16,
        p_width: u16,
        p_height: u16,
        scroll_y: usize,
    ) -> Result<()> {
        let mut w = stdout();
        let mut drawn_lines = 0;

        // RENDU DES LIGNES DIRECTEMENT DEPUIS LE CACHE DU NODE (0 calcul, 100% vitesse)
        for (line_idx, line_spans) in node
            .colored_lines
            .iter()
            .skip(scroll_y)
            .take(p_height as usize)
            .enumerate()
        {
            queue!(w, cursor::MoveTo(start_x, start_y + line_idx as u16))?;

            let mut current_width = 0;

            for (text, color) in line_spans {
                // Nettoyage des tabulations et retours chariots orphelins
                let clean_text = text.replace('\t', "    ").replace('\r', "");
                let text_width = clean_text.width();
                let remaining_width = p_width.saturating_sub(current_width) as usize;

                if remaining_width == 0 {
                    break; // Plus de place sur cette ligne
                }

                // Logique pour tronquer proprement le texte
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

                // On déréférence la couleur avec *color car on itère sur les références du cache
                queue!(w, SetForegroundColor(*color), Print(&display_text))?;
                current_width += display_text.width() as u16;
            }

            // Remplissage avec des espaces pour vider le reste de la ligne
            if current_width < p_width {
                let padding = " ".repeat((p_width - current_width) as usize);
                queue!(w, ResetColor, Print(padding))?;
            }

            drawn_lines += 1;
        }

        // Nettoyer les lignes restantes en bas du panneau (si fin de fichier)
        for empty_y in drawn_lines..(p_height as usize) {
            let padding = " ".repeat(p_width as usize);
            queue!(
                w,
                cursor::MoveTo(start_x, start_y + empty_y as u16),
                ResetColor,
                Print(padding)
            )?;
        }
        Ok(())
    }
    fn handle_events<W: Write>(&mut self, w: &mut W) -> Result<()> {
        match event::read()? {
            Event::Key(key) => {
                match self.mode {
                    Mode::Editor => {
                        match key.code {
                            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                let current_line = self.editor.cursor_line;
                                let total_lines = self.editor.rope.len_lines();

                                if current_line < total_lines {
                                    // 1. On récupère le nombre exact de caractères sur la ligne (incluant le \n)
                                    let chars_to_delete =
                                        self.editor.rope.line(current_line).len_chars();

                                    // 2. On place le curseur au tout début de la ligne
                                    self.editor.cursor_col = 0;

                                    // 3. On supprime proprement via l'API de `Ji` pour garder Tree-sitter à jour !
                                    for _ in 0..chars_to_delete {
                                        self.editor.delete();
                                    }

                                    // 4. Si on vient de supprimer la toute dernière ligne du fichier, on remonte le curseur
                                    if current_line >= self.editor.rope.len_lines()
                                        && current_line > 0
                                    {
                                        self.editor.cursor_line -= 1;
                                    }

                                    // 5. On synchronise l'affichage avec le nouveau cache
                                    self.sync_node_content();
                                }
                            }
                            // Sauvegarder (Ctrl + S)
                            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                if self.editor.save().is_err() {
                                    eprintln!("Erreur lors de la sauvegarde");
                                }
                            }

                            // Quitter le mode édition (Echap)
                            KeyCode::Esc => {
                                self.mode = Mode::Normal;
                                queue!(stdout(), Hide)?; // On cache le curseur en sortant
                            }

                            // ==========================================
                            // DÉPLACEMENTS DU CURSEUR (Flèches)
                            // ==========================================
                            KeyCode::Left => {
                                if self.editor.cursor_col > 0 {
                                    self.editor.cursor_col -= 1;
                                } else if self.editor.cursor_line > 0 {
                                    // Remonte à la fin de la ligne précédente
                                    self.editor.cursor_line -= 1;
                                    self.editor.cursor_col = self
                                        .editor
                                        .rope
                                        .line(self.editor.cursor_line)
                                        .len_chars()
                                        .saturating_sub(1);
                                }
                            }

                            KeyCode::Right => {
                                let max_col = self
                                    .editor
                                    .rope
                                    .line(self.editor.cursor_line)
                                    .len_chars()
                                    .saturating_sub(1);
                                if self.editor.cursor_col < max_col {
                                    self.editor.cursor_col += 1;
                                } else if self.editor.cursor_line + 1 < self.editor.rope.len_lines()
                                {
                                    // Passe au début de la ligne suivante
                                    self.editor.cursor_line += 1;
                                    self.editor.cursor_col = 0;
                                }
                            }

                            KeyCode::Up => {
                                if self.editor.cursor_line > 0 {
                                    self.editor.cursor_line -= 1;
                                    // On s'assure que la colonne ne dépasse pas la taille de la nouvelle ligne
                                    let max_col = self
                                        .editor
                                        .rope
                                        .line(self.editor.cursor_line)
                                        .len_chars()
                                        .saturating_sub(1);
                                    self.editor.cursor_col = self.editor.cursor_col.min(max_col);
                                }
                            }

                            KeyCode::Down => {
                                if self.editor.cursor_line + 1 < self.editor.rope.len_lines() {
                                    self.editor.cursor_line += 1;
                                    let max_col = self
                                        .editor
                                        .rope
                                        .line(self.editor.cursor_line)
                                        .len_chars()
                                        .saturating_sub(1);
                                    self.editor.cursor_col = self.editor.cursor_col.min(max_col);
                                }
                            }

                            KeyCode::Home => {
                                self.editor.cursor_col = 0;
                            }

                            KeyCode::End => {
                                self.editor.cursor_col = self
                                    .editor
                                    .rope
                                    .line(self.editor.cursor_line)
                                    .len_chars()
                                    .saturating_sub(1);
                            }

                            // ==========================================
                            // ACTIONS D'ÉDITION
                            // ==========================================
                            KeyCode::Backspace => {
                                self.editor.backspace();
                                self.sync_node_content();
                            }

                            KeyCode::Delete => {
                                self.editor.delete();
                                self.sync_node_content();
                            }

                            KeyCode::Enter => {
                                self.editor.insert_char('\n');
                                self.sync_node_content();
                            }

                            // Tabulation propre (convertie en 4 espaces pour le code)
                            KeyCode::Tab => {
                                for _ in 0..4 {
                                    self.editor.insert_char(' ');
                                }
                                self.sync_node_content();
                            }

                            // Saisie des caractères (Majuscule ou Minuscule)
                            KeyCode::Char(c)
                                if key.modifiers.is_empty()
                                    || key.modifiers == KeyModifiers::SHIFT =>
                            {
                                self.editor.insert_char(c);
                                self.sync_node_content();
                            }

                            _ => {}
                        }
                        let cursor_line = self.editor.cursor_line;
                        // On recalcule la hauteur de la vue actuelle
                        let mid_y = self.height / 2;
                        let bottom_y = self.height.saturating_sub(1);
                        let p_height = match self.focus {
                            PaneFocus::TopLeft | PaneFocus::TopRight => mid_y.saturating_sub(1),
                            PaneFocus::BottomLeft | PaneFocus::BottomRight => {
                                (bottom_y - mid_y).saturating_sub(1)
                            }
                        } as usize;

                        let pane = self.active_pane_mut();
                        let scroll_y = pane.cursor as usize;

                        // Si le curseur monte plus haut que la vue, on remonte le scroll
                        if cursor_line < scroll_y {
                            pane.cursor = cursor_line as u16;
                        }
                        // Si le curseur descend plus bas que la vue, on descend le scroll
                        else if cursor_line >= scroll_y + p_height {
                            pane.cursor =
                                (cursor_line.saturating_sub(p_height.saturating_sub(1))) as u16;
                        }
                    }

                    // ==========================================
                    // MODE NORMAL : Navigation et Raccourcis
                    // ==========================================
                    Mode::Normal => {
                        match (key.modifiers, key.code) {
                            (KeyModifiers::NONE, KeyCode::Char('q'))
                            | (KeyModifiers::NONE, KeyCode::Esc) => self.running = false,

                            // --- CHANGEMENT DE FACE (Cube F1 à F6) ---
                            (KeyModifiers::NONE, KeyCode::F(n)) if (1..=6).contains(&n) => {
                                // On stocke la face active (de 0 à 5 en interne)
                            }
                            // --- DÉFILEMENT RAPIDE (PageUp / PageDown) ---
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

                                let active_pane = self.active_pane_mut();
                                let step = 15; // Nombre de lignes sautées

                                if (active_pane.cursor as usize) + step < node_len {
                                    active_pane.cursor += step as u16;
                                } else {
                                    // Sécurité pour ne pas dépasser la fin du fichier
                                    active_pane.cursor = node_len.saturating_sub(1) as u16;
                                }
                            }
                            (KeyModifiers::NONE, KeyCode::PageUp) => {
                                let active_pane = self.active_pane_mut();
                                let step = 15; // Nombre de lignes sautées

                                // saturating_sub empêche de descendre en dessous de 0
                                active_pane.cursor = active_pane.cursor.saturating_sub(step);
                            }
                            (KeyModifiers::NONE, KeyCode::Char('e')) => {
                                let active_idx = self.focus as usize;

                                if let Some(view) = self.views.get(active_idx)
                                    && let Some(node) = self.nodes.get(view.active_node_id)
                                    && node.is_file
                                {
                                    let full_path = self.current_dir.join(&node.name);

                                    if let Some(path_str) = full_path.to_str()
                                        && let Ok(editor) = Ji::open(path_str)
                                    {
                                        let scroll_y = self.panes[active_idx].cursor as usize;
                                        if scroll_y < self.editor.rope.len_lines() {
                                            self.editor.cursor_line = scroll_y;
                                            self.editor.cursor_col = 0;
                                        }
                                        self.editor = editor;
                                        self.mode = Mode::Editor;
                                    }
                                }
                            }
                            (KeyModifiers::NONE, KeyCode::Char('j')) => {
                                // 1. ON LIT (Immutable)
                                let active_idx = self.focus as usize;
                                let node_len = if let Some(view) = self.views.get(active_idx) {
                                    self.nodes
                                        .get(view.active_node_id)
                                        .map(|n| n.content.len())
                                        .unwrap_or(0)
                                } else {
                                    0
                                };

                                // 2. ON MODIFIE (Mutable) après avoir terminé la lecture
                                let active_pane = self.active_pane_mut();
                                if (active_pane.cursor as usize) < node_len.saturating_sub(1) {
                                    active_pane.cursor += 1;
                                }
                            }
                            (KeyModifiers::NONE, KeyCode::Char('k')) => {
                                let active_pane = self.active_pane_mut();
                                // On soustrait 1 pour remonter, sans jamais descendre sous zéro
                                active_pane.cursor = active_pane.cursor.saturating_sub(1);
                            }
                            (KeyModifiers::NONE, KeyCode::Right) => {
                                self.focus = match self.focus {
                                    PaneFocus::TopLeft => PaneFocus::TopRight,
                                    PaneFocus::BottomLeft => PaneFocus::BottomRight,
                                    _ => self.focus,
                                };
                            }
                            (KeyModifiers::ALT, KeyCode::Char('f')) => {
                                self.mode = Mode::Finder;
                            }
                            (KeyModifiers::NONE, KeyCode::Left) => {
                                self.focus = match self.focus {
                                    PaneFocus::TopRight => PaneFocus::TopLeft,
                                    PaneFocus::BottomRight => PaneFocus::BottomLeft,
                                    _ => self.focus,
                                };
                            }
                            (KeyModifiers::NONE, KeyCode::Down) => {
                                self.focus = match self.focus {
                                    PaneFocus::TopLeft => PaneFocus::BottomLeft,
                                    PaneFocus::TopRight => PaneFocus::BottomRight,
                                    _ => self.focus,
                                };
                            }
                            (KeyModifiers::NONE, KeyCode::Up) => {
                                self.focus = match self.focus {
                                    PaneFocus::BottomLeft => PaneFocus::TopLeft,
                                    PaneFocus::BottomRight => PaneFocus::TopRight,
                                    _ => self.focus,
                                };
                            }

                            // --- CYCLAGE WORKSPACES (Alt + Gauche / Droite) ---
                            (KeyModifiers::ALT, KeyCode::Left) => {
                                let pane = self.active_pane_mut();
                                pane.workspace = if pane.workspace > 1 {
                                    pane.workspace - 1
                                } else {
                                    9
                                };
                            }
                            (KeyModifiers::ALT, KeyCode::Right) => {
                                let pane = self.active_pane_mut();
                                pane.workspace = if pane.workspace < 9 {
                                    pane.workspace + 1
                                } else {
                                    1
                                };
                            }

                            // --- CYCLAGE VIEWS (Alt + Haut / Bas) ---
                            (KeyModifiers::ALT, KeyCode::Up) => {
                                let pane = self.active_pane_mut();
                                pane.view = if pane.view < 9 { pane.view + 1 } else { 1 };
                            }
                            (KeyModifiers::ALT, KeyCode::Down) => {
                                let pane = self.active_pane_mut();
                                pane.view = if pane.view > 1 { pane.view - 1 } else { 9 };
                            }

                            // --- LANCEMENT DU DMENU (Alt + d) ---
                            (KeyModifiers::ALT, KeyCode::Char('d')) => {
                                self.mode = Mode::Dmenu;
                                self.dmenu_input.clear();
                            }

                            _ => {}
                        }
                    }

                    // ==========================================
                    // MODE DMENU : Saisie de texte
                    // ==========================================
                    Mode::Dmenu => {
                        match (key.modifiers, key.code) {
                            // --- ANNULER ET QUITTER LE MENU ---
                            (KeyModifiers::NONE, KeyCode::Esc) => {
                                self.mode = Mode::Normal;
                                self.dmenu_input.clear();
                            }

                            // --- VALIDER LA RECHERCHE OU LANCER UNE COMMANDE ---
                            (KeyModifiers::NONE, KeyCode::Enter) => {
                                if let Some(cmd) = self.dmenu_input.strip_prefix('!') {
                                    let cmd_clean = cmd.trim();

                                    if let Some(dir_name) = cmd_clean.strip_prefix("mkdir ") {
                                        // On combine le dossier actuel avec le nom saisi
                                        let target_path = self.current_dir.join(dir_name.trim());
                                        let _ = create_directory(&target_path);
                                    } else if let Some(file_name) = cmd_clean.strip_prefix("touch ")
                                    {
                                        // Pareil pour touch
                                        let target_path = self.current_dir.join(file_name.trim());
                                        let _ = create_empty_file(&target_path);
                                    } else {
                                        // 1. Exécuter la commande de façon bloquante (ex: cargo fmt)
                                        let _ = std::process::Command::new("sh")
                                            .arg("-c")
                                            .arg(cmd_clean)
                                            .stdout(std::process::Stdio::null())
                                            .stderr(std::process::Stdio::null())
                                            .status();

                                        // 2. Mettre à jour les fichiers en mémoire SANS casser les vues
                                        for node in self.nodes.iter_mut() {
                                            if node.is_file {
                                                let full_path = self.current_dir.join(&node.name);

                                                // On recharge le fichier silencieusement
                                                if let Ok(fresh_node) =
                                                    load_node(node.id, &full_path)
                                                {
                                                    node.content = fresh_node.content;
                                                    node.colored_lines = fresh_node.colored_lines;
                                                }
                                            }
                                        }
                                        let mut w = std::io::stdout();
                                        let _ = queue!(w, Clear(ClearType::All));
                                    }
                                } else {
                                    // TODO : Scan dmenu normal avec jwalk/walkdir
                                }
                                self.mode = Mode::Normal;
                                self.dmenu_input.clear();
                            }
                            (KeyModifiers::NONE, KeyCode::Backspace) => {
                                self.dmenu_input.pop();
                            }

                            (_, KeyCode::Char(c)) => {
                                self.dmenu_input.push(c);
                            }
                            _ => {}
                        }
                    }
                    Mode::Finder => match (key.modifiers, key.code) {
                        (KeyModifiers::SHIFT, KeyCode::Down) => {
                            self.finder.next_sub_dir();
                        }
                        (KeyModifiers::SHIFT, KeyCode::Up) => {
                            self.finder.prev_sub_dir();
                        }
                        (KeyModifiers::SHIFT, KeyCode::Right) => {
                            let sub_dirs = self.finder.get_sub_directories();

                            // On vérifie que la liste n'est pas vide et que l'index est valide
                            if !sub_dirs.is_empty() && self.finder.selected_sub_dir < sub_dirs.len()
                            {
                                let dirname = &sub_dirs[self.finder.selected_sub_dir];

                                // On construit le chemin en fusionnant le dossier actuel avec le sous-dossier ciblé
                                let new_path = self.current_dir.join(dirname);

                                // On met à jour le chemin actuel de l'application
                                self.current_dir = new_path.clone().into();

                                // On recrée le Finder pour qu'il scanne ce nouveau dossier
                                self.finder = Finder::new(&new_path, self.finder_layout.clone());

                                // On nettoie la barre de recherche
                                self.finder_recherch.clear();
                            }
                        }

                        (KeyModifiers::NONE, KeyCode::F(5)) => {
                            self.finder = Finder::new(Path::new("."), FinderLayout::Grid);
                        }
                        (KeyModifiers::NONE, KeyCode::Esc) => {
                            self.mode = Mode::Normal;
                            self.finder_recherch.clear();
                        }
                        (KeyModifiers::ALT, KeyCode::Right) => {
                            let dirs = self.finder.get_directories();

                            // On s'assure que la liste contient des dossiers et que le curseur est valide
                            if !dirs.is_empty() && self.finder.selected_dir < dirs.len() {
                                let dirname = &dirs[self.finder.selected_dir];
                                let new_path = self.current_dir.join(dirname);

                                // On met à jour le chemin actuel de l'application
                                self.current_dir = new_path.clone().into();

                                // On recrée le Finder pour qu'il scanne ce nouveau dossier
                                self.finder = Finder::new(&new_path, self.finder_layout.clone());

                                // On nettoie la barre de recherche
                                self.finder_recherch.clear();
                            }
                        }
                        (KeyModifiers::ALT, KeyCode::Left) => {
                            // On utilise .parent() pour remonter d'un niveau en toute sécurité
                            if let Some(parent) = self.current_dir.parent() {
                                self.current_dir = parent.into();
                                self.finder =
                                    Finder::new(&self.current_dir, self.finder_layout.clone());
                                self.finder_recherch.clear();
                            }
                        }
                        (KeyModifiers::META, KeyCode::Left) => {
                            self.previous_finder_layout();
                        }
                        (KeyModifiers::META, KeyCode::Right) => {
                            self.next_finder_layout();
                        }
                        (KeyModifiers::ALT, KeyCode::Char('j')) => {
                            self.finder.next_file();
                        }
                        (KeyModifiers::ALT, KeyCode::Char('k')) => {
                            self.finder.prev_file();
                        }
                        (KeyModifiers::ALT, KeyCode::Down) => {
                            self.finder.next_dir();
                        }
                        (KeyModifiers::ALT, KeyCode::Up) => {
                            self.finder.prev_dir();
                        }
                        (KeyModifiers::NONE, KeyCode::Enter) => {
                            let files = self.finder.get_files();
                            if !files.is_empty() && self.finder.selected_file < files.len() {
                                let filename = &files[self.finder.selected_file];

                                // 1. On construit le chemin complet du fichier sélectionné
                                let full_path = self.current_dir.join(filename);

                                // 2. On charge le noeud (le cache visuel)
                                let node_id = if let Some(existing_node) =
                                    self.nodes.iter().find(|n| n.name == *filename)
                                {
                                    existing_node.id
                                } else {
                                    let new_id = self.nodes.len();
                                    if let Ok(node) = load_node(new_id, &full_path) {
                                        self.nodes.push(node);
                                        new_id
                                    } else {
                                        self.finder_recherch.clear();
                                        return Ok(());
                                    }
                                };
                                let active_idx = self.focus as usize;

                                // Sécurité pour s'assurer que la vue existe bien
                                if self.views.len() <= active_idx {
                                    self.views
                                        .resize_with(active_idx + 1, || View { active_node_id: 0 });
                                }

                                // On assigne le nouvel ID de fichier à afficher
                                if let Some(view) = self.views.get_mut(active_idx) {
                                    view.active_node_id = node_id;
                                }

                                // Réinitialiser le scroll pour le nouveau fichier
                                self.panes[active_idx].cursor = 0;

                                // ==========================================
                                // ✨ NOUVEAUTÉ : BASCULE DIRECTE EN ÉDITION
                                // ==========================================
                                if let Some(path_str) = full_path.to_str() {
                                    if let Ok(editor) = Ji::open(path_str) {
                                        // On charge le fichier dans le moteur d'édition
                                        self.editor = editor;

                                        // On place l'application en mode Éditeur !
                                        self.mode = Mode::Editor;
                                    } else {
                                        // Fallback de sécurité si le fichier n'est pas lisible
                                        self.mode = Mode::Normal;
                                    }
                                } else {
                                    self.mode = Mode::Normal;
                                }
                            } else {
                                // Si on appuie sur Enter avec une liste vide
                                self.mode = Mode::Normal;
                            }

                            // 4. On nettoie la barre de recherche dans tous les cas
                            self.finder_recherch.clear();
                        }
                        (KeyModifiers::NONE, KeyCode::Backspace) => {
                            self.finder_recherch.pop();
                            self.draw(w)?;
                            w.flush()?;
                            self.finder.filter(self.finder_recherch.clone());
                        }
                        (_, KeyCode::Char(c)) => {
                            self.finder_recherch.push(c);
                            self.draw(w)?;
                            w.flush()?;
                            self.finder.filter(self.finder_recherch.clone());
                        }
                        _ => {}
                    },
                }
            }
            Event::Resize(columns, rows) => {
                self.width = columns;
                self.height = rows;
                self.finder.resize(columns, rows);
                queue!(stdout(), Clear(ClearType::All))?;
            }
            _ => {}
        }
        Ok(())
    }

    /// Gère l'affichage de l'interface
    fn draw<W: Write>(&mut self, w: &mut W) -> Result<()> {
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
            cursor::MoveTo(left_x, top_y),
            SetForegroundColor(UI_BORDER_INACTIVE),
            Print(format!("┌{}┐", horiz_line)),
            cursor::MoveTo(left_x, bottom_y),
            Print(format!("└{}┘", horiz_line))
        )?;

        // Lignes verticales extérieures (Gauche et Droite)
        for y in (top_y + 1)..bottom_y {
            if y != mid_y {
                queue!(
                    w,
                    cursor::MoveTo(left_x, y),
                    SetForegroundColor(UI_BORDER_INACTIVE),
                    Print("│"),
                    cursor::MoveTo(right_x, y),
                    Print("│")
                )?;
            }
        }

        // Ligne de séparation horizontale centrale
        for x in (left_x + 1)..right_x {
            if x != mid_x {
                queue!(
                    w,
                    cursor::MoveTo(x, mid_y),
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
                    cursor::MoveTo(mid_x, y),
                    SetForegroundColor(UI_BORDER_INACTIVE),
                    Print("│")
                )?;
            }
        }

        // Intersections
        queue!(
            w,
            SetForegroundColor(UI_BORDER_INACTIVE),
            cursor::MoveTo(left_x, mid_y),
            Print("├"),
            cursor::MoveTo(right_x, mid_y),
            Print("┤"),
            cursor::MoveTo(mid_x, top_y),
            Print("┬"),
            cursor::MoveTo(mid_x, bottom_y),
            Print("┴"),
            cursor::MoveTo(mid_x, mid_y),
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
                let _ = self.preview(
                    node,
                    start_x,
                    start_y,
                    p_width,
                    p_height,
                    pane.cursor as usize,
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
            queue!(w, cursor::MoveTo(indicator_x, indicator_y))?;
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
                cursor::MoveTo(start_x, start_y),
                SetBackgroundColor(UI_DMENU_BG),
                SetForegroundColor(UI_DMENU_FG),
                Print(padded_prompt),
                ResetColor
            )?;
        }

        // ==========================================
        // POSITIONNEMENT DU CURSEUR EN MODE ÉDITEUR
        // ==========================================
        if self.mode == Mode::Editor {
            queue!(w, Show)?;
            let active_bounds = panes_bounds
                .iter()
                .find(|(focus, _, _, _, _)| *focus == self.focus);

            if let Some(&(_, start_x, start_y, p_width, p_height)) = active_bounds {
                let active_pane = self.panes[self.focus as usize];
                let scroll_y = active_pane.cursor as usize;
                let line_idx = self.editor.cursor_line;
                let col_idx = self.editor.cursor_col;

                if line_idx >= scroll_y && line_idx < scroll_y + (p_height as usize) {
                    let screen_y = start_y + (line_idx - scroll_y) as u16;
                    let screen_x = start_x + (col_idx as u16).min(p_width.saturating_sub(1));

                    queue!(w, cursor::MoveTo(screen_x, screen_y), cursor::Show)?;
                } else {
                    queue!(w, Hide)?;
                }
            }
        } else {
            queue!(w, Hide)?;
        }
        queue!(w, ResetColor)?;
        w.flush()?;
        Ok(())
    }
}
