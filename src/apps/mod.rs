use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{Write, stdout};
use std::{io::Result, time::Duration};

// Niveau 10 : Le Nœud (La donnée brute en mémoire)
pub struct Node {
    pub id: usize,
    pub content: Vec<String>, // Les lignes de ton fichier texte
}

// Niveau 9 : Le Calque (La surcouche visuelle)
pub struct Layer {
    pub name: String, // ex: "Base_Text", "Linter_Errors"
    pub is_visible: bool,
}

// Niveau 8 : La Vue (La fenêtre de défilement)
pub struct View {
    pub active_node_id: usize,
    pub layers: Vec<Layer>,
    pub cursor_x: u16,
    pub cursor_y: u16,
}

#[allow(dead_code)]
struct Environment {
    name: String,
    // Chaque environnement a ses 6 faces d'outils, contenant chacune 4 quadrants
    faces: [[PaneState; 4]; 6],
}
#[derive(Copy, Clone, PartialEq)]
#[allow(dead_code)]
enum Face {
    Front = 0,
    Back = 1,
    Left = 2,
    Right = 3,
    Top = 4,
    Bottom = 5,
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
}
#[derive(Copy, Clone)]
struct PaneState {
    workspace: u8,
    view: u8,
}

pub struct App {
    environments: Vec<Environment>,
    nodes: Vec<Node>, // La mémoire brute
    views: Vec<View>, // Les fenêtres de défilement
    width: u16,
    height: u16,
    running: bool,
    focus: PaneFocus,
    panes: [PaneState; 4],
    mode: Mode,
    dmenu_input: String,
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
impl App {
    /// Retourne une référence mutable (pour modifier) le panneau actif
    fn active_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[self.focus as usize]
    }

    pub fn new() -> Result<Self> {
        let (width, height) = terminal::size()?;

        // 1. Création d'un Nœud de test avec quelques lignes de code
        let initial_node = Node {
            id: 1,
            content: vec![
                String::from("fn main() {"),
                String::from("    println!(\"Bienvenue dans Qwx\");"),
                String::from("}"),
            ],
        };

        // 2. Création de la Vue associée
        let initial_view = View {
            active_node_id: 1,
            layers: Vec::new(),
            cursor_x: 0,
            cursor_y: 0,
        };

        Ok(Self {
            width,
            height,
            running: true,
            focus: PaneFocus::TopLeft,
            panes: [
                PaneState {
                    workspace: 1,
                    view: 1,
                },
                PaneState {
                    workspace: 2,
                    view: 1,
                },
                PaneState {
                    workspace: 3,
                    view: 1,
                },
                PaneState {
                    workspace: 4,
                    view: 1,
                },
            ],
            mode: Mode::Normal,
            dmenu_input: String::new(),
            environments: Vec::new(),
            nodes: vec![initial_node], // On injecte la donnée
            views: vec![initial_view], // On injecte la vue
        })
    }

    /// Boucle principale de l'application
    pub fn run(&mut self) -> Result<()> {
        let mut stdout = stdout();

        // Passage en mode brut et écran alternatif (pour ne pas polluer le terminal de base)
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;

        while self.running {
            self.draw(&mut stdout)?;
            self.handle_events()?;
        }

        // Nettoyage en quittant
        execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
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
                                // --- QUITTER ---
                                (KeyModifiers::NONE, KeyCode::Char('q'))
                                | (KeyModifiers::NONE, KeyCode::Esc) => self.running = false,

                                // --- CHANGEMENT DE FACE (Cube F1 à F6) ---
                                (KeyModifiers::NONE, KeyCode::F(n)) if (1..=6).contains(&n) => {
                                    // On stocke la face active (de 0 à 5 en interne)
                                }

                                // --- DÉPLACEMENT DU FOCUS (Flèches simples) ---
                                (KeyModifiers::NONE, KeyCode::Right) => {
                                    self.focus = match self.focus {
                                        PaneFocus::TopLeft => PaneFocus::TopRight,
                                        PaneFocus::BottomLeft => PaneFocus::BottomRight,
                                        _ => self.focus,
                                    };
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

                                // --- TAPER DU TEXTE ---
                                // On ignore les modificateurs pour attraper les lettres simplement
                                (_, KeyCode::Char(c)) => {
                                    self.dmenu_input.push(c);
                                }

                                _ => {}
                            }
                        }
                    }
                }
                Event::Resize(columns, rows) => {
                    self.width = columns;
                    self.height = rows;
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// Gère l'affichage de l'interface
    fn draw<W: Write>(&self, w: &mut W) -> Result<()> {
        queue!(w, Clear(ClearType::All))?;
        let mid_x = self.width / 2;
        let mid_y = self.height / 2;

        // 1. Dessiner la croix centrale (Gris sombre pour ne pas agresser l'œil)
        for x in 0..self.width {
            if x != mid_x {
                queue!(
                    w,
                    cursor::MoveTo(x, mid_y),
                    SetForegroundColor(Color::DarkGrey),
                    Print("─")
                )?;
            }
        }
        for y in 0..self.height {
            if y != mid_y {
                queue!(
                    w,
                    cursor::MoveTo(mid_x, y),
                    SetForegroundColor(Color::DarkGrey),
                    Print("│")
                )?;
            }
        }
        queue!(
            w,
            cursor::MoveTo(mid_x, mid_y),
            SetForegroundColor(Color::DarkGrey),
            Print("┼")
        )?;

        // 2. Définir les zones (x_départ, y_départ, largeur, hauteur) pour chaque panneau
        let panes_bounds = [
            (PaneFocus::TopLeft, 0, 0, mid_x, mid_y),
            (
                PaneFocus::TopRight,
                mid_x + 1,
                0,
                self.width.saturating_sub(mid_x + 1),
                mid_y,
            ),
            (
                PaneFocus::BottomLeft,
                0,
                mid_y + 1,
                mid_x,
                self.height.saturating_sub(mid_y + 1),
            ),
            (
                PaneFocus::BottomRight,
                mid_x + 1,
                mid_y + 1,
                self.width.saturating_sub(mid_x + 1),
                self.height.saturating_sub(mid_y + 1),
            ),
        ];

        // 3. Dessiner le contenu de chaque panneau
        for (i, &(pane_focus, start_x, start_y, p_width, p_height)) in
            panes_bounds.iter().enumerate()
        {
            let pane = self.panes[i];
            let is_active = self.focus == pane_focus;

            // Le panneau actif est en Cyan, les autres sont grisés
            let text_color = if is_active {
                Color::Cyan
            } else {
                Color::DarkGrey
            };

            // Récupération de la donnée : on cherche la vue et le nœud associé
            // (Pour ce prototype, on prend la première vue et le premier nœud dispo)
            if let Some(view) = self.views.first()
                && let Some(node) = self.nodes.iter().find(|n| n.id == view.active_node_id)
            {
                // Itération sur le texte, limitée à la hauteur disponible du panneau
                for (line_idx, line) in node.content.iter().take(p_height as usize).enumerate() {
                    // On tronque la ligne pour éviter qu'elle ne déborde visuellement du quadrant
                    let display_line = if line.len() > p_width as usize {
                        &line[0..p_width as usize]
                    } else {
                        line
                    };

                    queue!(
                        w,
                        cursor::MoveTo(start_x, start_y + line_idx as u16),
                        SetForegroundColor(text_color),
                        Print(display_line)
                    )?;
                }
            }

            // 4. Placer les indicateurs Workspaces/Views en bas à droite de chaque panneau
            let expo = get_superscript(pane.view);
            let indicator_x = start_x + p_width.saturating_sub(3);
            let indicator_y = start_y + p_height.saturating_sub(1);

            queue!(w, cursor::MoveTo(indicator_x, indicator_y))?;

            if is_active {
                queue!(
                    w,
                    SetForegroundColor(Color::Green),
                    Print(pane.workspace),
                    SetForegroundColor(Color::Cyan),
                    Print(expo)
                )?;
            } else {
                queue!(
                    w,
                    SetForegroundColor(Color::DarkGrey),
                    Print(pane.workspace),
                    Print(expo)
                )?;
            }
        }

        // 5. Rendu du mode Dmenu (Barre de commande)
        if self.mode == Mode::Dmenu {
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
                SetAttribute(crossterm::style::Attribute::Bold),
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
