use crate::finder::{Finder, FinderLayout, list_files};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::Write;
use std::time::Duration;
use std::{
    fs::File,
    io::{BufRead, BufReader, Result, stdout},
    path::{Path, PathBuf},
};
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
// Niveau 10 : Le Nœud (La donnée brute en mémoire)
#[derive(Default, Clone)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub ext: String,
    pub content: Vec<String>, // Les lignes de ton fichier texte
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
    Dmenu, // Le mode où l'on tape du texte dans la barre verte
    Finder,
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
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
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

impl App {
    fn active_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[self.focus as usize]
    }

    pub fn new(path: &Path) -> Result<Self> {
        let (width, height) = terminal::size()?;
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        let mut nodes: Vec<Node> = Vec::new();
        let mut views: Vec<View> = Vec::new();
        for (i, filename) in list_files(path).iter().enumerate() {
            nodes.push(Node {
                id: i,
                content: read_lines(PathBuf::from(filename).as_path())?,
                name: PathBuf::from(filename)
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
                ext: PathBuf::from(filename)
                    .extension()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_string(),
                is_file: PathBuf::from(filename).is_file(),
            });
            views.push(View { active_node_id: i });
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
            syntax_set,
            theme_set,
        })
    }

    fn draw_finder<W: Write>(&mut self, w: &mut W) -> Result<()> {
        self.finder.draw(w, self.finder_recherch.to_string())
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
            self.handle_events()?;
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
        let syntax = self
            .syntax_set
            .find_syntax_by_extension(&node.ext)
            .unwrap_or(self.syntax_set.find_syntax_plain_text());
        let mut highlighter =
            syntect::easy::HighlightLines::new(syntax, &self.theme_set.themes["base16-ocean.dark"]);
        let mut w = stdout();

        let mut drawn_lines = 0; // <-- NOUVEAU : On traque le nombre de lignes dessinées

        for (line_idx, line) in node
            .content
            .iter()
            .skip(scroll_y)
            .take(p_height as usize)
            .enumerate()
        {
            queue!(w, cursor::MoveTo(start_x, start_y + line_idx as u16))?;

            // syntect a besoin du \n pour bien identifier les fins d'instructions ou commentaires
            let line_with_nl = format!("{line}\n");

            // On demande à syntect de découper la ligne en segments (style, texte)
            let ranges: Vec<(syntect::highlighting::Style, &str)> = highlighter
                .highlight_line(&line_with_nl, &self.syntax_set)
                .unwrap();

            let mut current_width = 0;

            for (style, text) in ranges {
                let clean_text = text
                    .trim_end_matches(&['\n', '\r'][..])
                    .replace('\t', "    ");

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

                let crossterm_color = Color::Rgb {
                    r: style.foreground.r,
                    g: style.foreground.g,
                    b: style.foreground.b,
                };

                queue!(w, SetForegroundColor(crossterm_color), Print(&display_text))?;
                current_width += display_text.width() as u16;
            }

            // ✨ CORRECTION 1 : Nettoyer la fin de la ligne avec des espaces si elle est trop courte
            if current_width < p_width {
                let padding = " ".repeat((p_width - current_width) as usize);
                queue!(w, ResetColor, Print(padding))?;
            }

            drawn_lines += 1;
        }

        // ✨ CORRECTION 2 : Nettoyer les lignes restantes en bas du panneau avec des lignes vides
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
    fn handle_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    match self.mode {
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

                                // --- VALIDER LA RECHERCHE ---
                                (KeyModifiers::NONE, KeyCode::Enter) => {
                                    // TODO : Implémenter le scan de dossier avec walkdir/jwalk ici
                                    // en utilisant le contenu de self.dmenu_input

                                    self.mode = Mode::Normal;
                                    self.dmenu_input.clear();
                                }

                                // --- EFFACER UN CARACTÈRE ---
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
                            (KeyModifiers::ALT, KeyCode::Down) => {
                                self.finder.next_dir();
                            }
                            (KeyModifiers::ALT, KeyCode::Up) => {
                                self.finder.prev_dir();
                            }
                            (KeyModifiers::NONE, KeyCode::F(5)) => {
                                self.finder = Finder::new(Path::new("."), FinderLayout::Grid);
                            }
                            (KeyModifiers::NONE, KeyCode::Esc) => {
                                self.mode = Mode::Normal;
                                self.finder_recherch.clear();
                            }
                            (KeyModifiers::ALT, KeyCode::Right) => {
                                // On récupère le premier dossier de la liste filtrée
                                if let Some(dirname) = self.finder.get_directories().first() {
                                    let new_path = self.current_dir.join(dirname);

                                    // On met à jour le chemin actuel de l'application
                                    self.current_dir = new_path.clone().into();

                                    // On recrée le Finder pour qu'il scanne ce nouveau dossier
                                    self.finder =
                                        Finder::new(&new_path, self.finder_layout.clone());

                                    // On nettoie la barre de recherche
                                    self.finder_recherch.clear();
                                }
                            }
                            // --- REMONTER AU DOSSIER PARENT ---
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
                            (KeyModifiers::NONE, KeyCode::Enter) => {
                                if let Some(filename) = self.finder.get_files().first() {
                                    let full_path = self.current_dir.join(filename);

                                    let node_id = if let Some(existing_node) =
                                        self.nodes.iter().find(|n| n.name == *filename)
                                    {
                                        existing_node.id
                                    } else {
                                        let new_id = self.nodes.len();
                                        // On utilise ta fonction pour lire le contenu
                                        if let Ok(content) = read_lines(&full_path) {
                                            self.nodes.push(Node {
                                                id: new_id,
                                                name: filename.clone(),
                                                ext: full_path
                                                    .extension()
                                                    .unwrap_or_default()
                                                    .to_str()
                                                    .unwrap_or_default()
                                                    .to_string(),
                                                content,
                                                is_file: true,
                                            });
                                            new_id
                                        } else {
                                            self.finder_recherch.clear();
                                            // En cas d'erreur de lecture (fichier protégé, etc.), on stoppe l'action
                                            return Ok(());
                                        }
                                    };
                                    let active_idx = self.focus as usize;

                                    // Sécurité pour s'assurer que la vue existe bien
                                    if self.views.len() <= active_idx {
                                        self.views.resize_with(active_idx + 1, || View {
                                            active_node_id: 0,
                                        });
                                    }

                                    // On assigne le nouvel ID de fichier à afficher
                                    if let Some(view) = self.views.get_mut(active_idx) {
                                        view.active_node_id = node_id;
                                    }

                                    // ✨ LA CORRECTION EST ICI : Réinitialiser le scroll pour le nouveau fichier
                                    self.panes[active_idx].cursor = 0;
                                }

                                // 4. On nettoie la barre de recherche et on quitte le finder
                                self.finder_recherch.clear();
                                self.mode = Mode::Normal;
                            }
                            (KeyModifiers::NONE, KeyCode::Backspace) => {
                                self.finder_recherch.pop();
                                self.finder
                                    .filter(&self.current_dir, self.finder_recherch.clone());
                            }
                            (_, KeyCode::Char(c)) => {
                                self.finder_recherch.push(c);
                                self.finder
                                    .filter(&self.current_dir, self.finder_recherch.clone());
                            }
                            _ => {}
                        },
                    }
                }
                Event::Resize(columns, rows) => {
                    self.width = columns;
                    self.height = rows;
                    self.finder.resize(columns, rows);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Gère l'affichage de l'interface
    fn draw<W: Write>(&mut self, w: &mut W) -> Result<()> {
        let mid_x = self.width / 2;
        let mid_y = self.height / 2;
        let right_x = self.width.saturating_sub(1);
        let bottom_y = self.height.saturating_sub(1);

        // ==========================================
        // 1. DESSINER LE CADRE EXTÉRIEUR ET LA CROIX
        // ==========================================

        // Coins et lignes horizontales du cadre (Haut et Bas)
        queue!(
            w,
            cursor::MoveTo(0, 0),
            SetForegroundColor(Color::DarkGrey),
            Print(format!(
                "┌{}┐",
                "─".repeat((self.width.saturating_sub(2)) as usize)
            )),
            cursor::MoveTo(0, bottom_y),
            Print(format!(
                "└{}┘",
                "─".repeat((self.width.saturating_sub(2)) as usize)
            ))
        )?;

        // Lignes verticales extérieures (Gauche et Droite) et séparateur horizontal interne
        for y in 1..bottom_y {
            if y != mid_y {
                queue!(
                    w,
                    cursor::MoveTo(0, y),
                    SetForegroundColor(Color::DarkGrey),
                    Print("│"),
                    cursor::MoveTo(right_x, y),
                    Print("│")
                )?;
            }
        }

        // Ligne de séparation horizontale centrale (entre le haut et le bas)
        for x in 1..right_x {
            if x != mid_x {
                queue!(
                    w,
                    cursor::MoveTo(x, mid_y),
                    SetForegroundColor(Color::DarkGrey),
                    Print("─")
                )?;
            }
        }

        // Ligne de séparation verticale centrale (entre la gauche et la droite)
        for y in 1..bottom_y {
            queue!(
                w,
                cursor::MoveTo(mid_x, y),
                SetForegroundColor(Color::DarkGrey),
                Print("│")
            )?;
        }
        // Intersections des bordures extérieures avec la croix centrale
        queue!(
            w,
            cursor::MoveTo(0, mid_y),
            Print("├"),
            cursor::MoveTo(right_x, mid_y),
            Print("┤"),
            cursor::MoveTo(mid_x, 0),
            Print("┬"),
            cursor::MoveTo(mid_x, bottom_y),
            Print("┴"),
            cursor::MoveTo(mid_x, mid_y),
            Print("┼")
        )?;
        // 3. Dessiner le contenu de chaque panneau
        // 2. Définir les zones (x_départ, y_départ, largeur, hauteur) en protégeant les bordures
        let panes_bounds = [
            (
                PaneFocus::TopLeft,
                1,
                1,
                mid_x.saturating_sub(1),
                mid_y.saturating_sub(1),
            ),
            (
                PaneFocus::TopRight,
                mid_x + 1,
                1,
                self.width.saturating_sub(mid_x + 2),
                mid_y.saturating_sub(1),
            ),
            (
                PaneFocus::BottomLeft,
                1,
                mid_y + 1,
                mid_x.saturating_sub(1),
                self.height.saturating_sub(mid_y + 2),
            ),
            (
                PaneFocus::BottomRight,
                mid_x + 1,
                mid_y + 1,
                self.width.saturating_sub(mid_x + 2),
                self.height.saturating_sub(mid_y + 2),
            ),
        ];

        for (i, &(pane_focus, start_x, start_y, p_width, p_height)) in
            panes_bounds.iter().enumerate()
        {
            let pane = self.panes[i];
            let is_active = self.focus == pane_focus;

            if let Some(view) = self.views.get(i)
                && let Some(node) = self.nodes.iter().find(|n| n.id == view.active_node_id)
                && node.is_file
            {
                // On passe le curseur propre de ce panneau spécifique
                let _ = self.preview(
                    node,
                    start_x,
                    start_y,
                    p_width,
                    p_height,
                    pane.cursor as usize,
                );
            }

            // Calcul du pourcentage basé sur le curseur de CE panneau
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
                queue!(
                    w,
                    SetForegroundColor(Color::Green),
                    Print(format!("{}% ", percentage_str)),
                    SetForegroundColor(Color::Green),
                    Print(pane.workspace),
                    SetForegroundColor(Color::Cyan),
                    Print(expo)
                )?;
            } else {
                queue!(
                    w,
                    SetForegroundColor(Color::DarkGrey),
                    Print(format!("{}% ", percentage_str)),
                    Print(pane.workspace),
                    Print(expo)
                )?;
            }
        }
        if self.is_finder_open() {
            self.draw_finder(w)?;
        } else if self.mode == Mode::Dmenu {
            let (start_x, start_y, pane_width) = match self.focus {
                PaneFocus::TopLeft => (0, 0, mid_x),
                PaneFocus::TopRight => (mid_x + 1, 0, self.width.saturating_sub(mid_x + 1)),
                PaneFocus::BottomLeft => (0, mid_y + 1, mid_x),
                PaneFocus::BottomRight => {
                    (mid_x + 1, mid_y + 1, self.width.saturating_sub(mid_x + 1))
                }
            };

            let prompt = format!(" {} ", self.dmenu_input);
            let padded_prompt = format!("{:<width$}", prompt, width = pane_width as usize);

            // Palette d'inspiration océan sombre pour l'interface de commande
            queue!(
                w,
                cursor::MoveTo(start_x, start_y),
                SetBackgroundColor(Color::Green),
                SetForegroundColor(Color::Black),
                Print(padded_prompt),
                ResetColor
            )?;
        }
        queue!(w, ResetColor)?;
        w.flush()?;
        Ok(())
    }
}
