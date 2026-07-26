use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{Write, stdout};
use std::{io::Result, time::Duration};

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
#[allow(dead_code)]
pub struct App {
    environments: Vec<Environment>,
    width: u16,
    height: u16,
    running: bool,
    focus: PaneFocus,      // Le panneau actuellement actif
    panes: [PaneState; 4], // Le tableau qui stocke l'état des 4 panneaux
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
        Ok(Self {
            width,
            height,
            running: true,
            focus: PaneFocus::TopLeft, // Focus par défaut en haut à gauche
            panes: [
                PaneState {
                    workspace: 1,
                    view: 1,
                }, // TopLeft
                PaneState {
                    workspace: 2,
                    view: 1,
                }, // TopRight
                PaneState {
                    workspace: 3,
                    view: 1,
                }, // BottomLeft
                PaneState {
                    workspace: 4,
                    view: 1,
                }, // BottomRight
            ],
            mode: Mode::Normal,
            dmenu_input: String::new(),
            environments: Vec::new(),
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
        let positions = [
            (
                mid_x.saturating_sub(4),
                mid_y.saturating_sub(2),
                PaneFocus::TopLeft,
            ),
            (
                mid_x.saturating_add(3),
                mid_y.saturating_sub(2),
                PaneFocus::TopRight,
            ),
            (
                mid_x.saturating_sub(4),
                mid_y.saturating_add(2),
                PaneFocus::BottomLeft,
            ),
            (
                mid_x.saturating_add(3),
                mid_y.saturating_add(2),
                PaneFocus::BottomRight,
            ),
        ];
        // 1. Dessiner la ligne horizontale
        for x in 0..self.width {
            if x != mid_x {
                queue!(
                    w,
                    cursor::MoveTo(x, mid_y),
                    SetForegroundColor(Color::Grey),
                    Print("─")
                )?;
            }
        }

        // 2. Dessiner la ligne verticale
        for y in 0..self.height {
            if y != mid_y {
                queue!(
                    w,
                    cursor::MoveTo(mid_x, y),
                    SetForegroundColor(Color::Grey),
                    Print("│")
                )?;
            }
        }

        // 3. L'intersection au centre
        queue!(
            w,
            cursor::MoveTo(mid_x, mid_y),
            SetForegroundColor(Color::Grey),
            Print("┼")
        )?;

        // 4. Placer les indicateurs Workspaces/Views (les exposants)

        for (i, &(x, y, pane_type)) in positions.iter().enumerate() {
            let pane = self.panes[i];
            let expo = get_superscript(pane.view);
            let is_active = self.focus == pane_type;

            queue!(w, cursor::MoveTo(x, y))?;

            if is_active {
                // Panneau actif : Vert Clair et Cyan
                queue!(w, SetForegroundColor(Color::Green), Print(pane.workspace))?;
                queue!(w, SetForegroundColor(Color::Cyan), Print(expo))?;
            } else {
                // Panneaux inactifs : Gris sombre pour ne pas distraire
                queue!(
                    w,
                    SetForegroundColor(Color::DarkGrey),
                    Print(pane.workspace),
                    Print(expo)
                )?;
            }
        }
        if self.mode == Mode::Dmenu {
            // 1. Déterminer la position et la largeur selon le focus
            let (start_x, start_y, pane_width) = match self.focus {
                PaneFocus::TopLeft => (0, 0, mid_x),
                PaneFocus::TopRight => (mid_x, 0, self.width - mid_x),
                PaneFocus::BottomLeft => (0, mid_y, mid_x),
                PaneFocus::BottomRight => (mid_x, mid_y, self.width - mid_x),
            };

            // 2. Préparer la chaîne à afficher (avec des espaces pour remplir la largeur)
            let prompt = format!(" > {} ", self.dmenu_input);
            let padded_prompt = format!("{:<width$}", prompt, width = pane_width as usize);

            // 3. Dessiner la barre (Fond Vert, Texte Noir)
            queue!(
                w,
                cursor::MoveTo(start_x, start_y),
                SetBackgroundColor(Color::Green),
                SetForegroundColor(Color::Black),
                Print(padded_prompt),
                ResetColor // On n'oublie pas de réinitialiser !
            )?;
        }
        queue!(w, ResetColor)?;
        // On envoie tout au terminal d'un coup
        w.flush()?;
        Ok(())
    }
}
