use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::Result;
use std::io::{Write, stdout};

#[derive(Copy, Clone, PartialEq)]
enum PaneFocus {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
}
#[derive(Copy, Clone)]
struct PaneState {
    workspace: u8,
    view: u8,
}
pub struct App {
    width: u16,
    height: u16,
    running: bool,
    focus: PaneFocus,      // Le panneau actuellement actif
    panes: [PaneState; 4], // Le tableau qui stocke l'état des 4 panneaux
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
            self.handle()?;
        }

        // Nettoyage en quittant
        execute!(stdout, LeaveAlternateScreen, cursor::Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }
    fn handle(&mut self) -> Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => self.running = false,

                        // --- DÉPLACEMENT DU FOCUS ---
                        // Utilise les flèches (ou ajoute 'h','j','k','l' si tu préfères la navigation Vim)
                        KeyCode::Right => {
                            self.focus = match self.focus {
                                PaneFocus::TopLeft => PaneFocus::TopRight,
                                PaneFocus::BottomLeft => PaneFocus::BottomRight,
                                _ => self.focus,
                            };
                        }
                        KeyCode::Left => {
                            self.focus = match self.focus {
                                PaneFocus::TopRight => PaneFocus::TopLeft,
                                PaneFocus::BottomRight => PaneFocus::BottomLeft,
                                _ => self.focus,
                            };
                        }
                        KeyCode::Down => {
                            self.focus = match self.focus {
                                PaneFocus::TopLeft => PaneFocus::BottomLeft,
                                PaneFocus::TopRight => PaneFocus::BottomRight,
                                _ => self.focus,
                            };
                        }
                        KeyCode::Up => {
                            self.focus = match self.focus {
                                PaneFocus::BottomLeft => PaneFocus::TopLeft,
                                PaneFocus::BottomRight => PaneFocus::TopRight,
                                _ => self.focus,
                            };
                        }
                        // --- CHANGEMENT DE WORKSPACE ---
                        // Si on tape un chiffre entre 1 et 9, on change le workspace du panneau actif
                        KeyCode::Char(c) if c.is_ascii_digit() && c != '0' => {
                            let val = c.to_digit(10).unwrap() as u8;
                            let idx = self.focus as usize;
                            self.panes[idx].workspace = val;
                        }
                        // --- CHANGEMENTDE VIEW (Exemple avec Maj + Chiffre) ---
                        // On pourrait utiliser un modificateur pour la view
                        // ... à définir selon tes préférences de raccourcis !
                        _ => {}
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
                queue!(w, cursor::MoveTo(x, mid_y), Print("─"))?;
            }
        }

        // 2. Dessiner la ligne verticale
        for y in 0..self.height {
            if y != mid_y {
                queue!(w, cursor::MoveTo(mid_x, y), Print("│"))?;
            }
        }

        // 3. L'intersection au centre
        queue!(w, cursor::MoveTo(mid_x, mid_y), Print("┼"))?;

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
        queue!(w, ResetColor)?;
        // On envoie tout au terminal d'un coup
        w.flush()?;
        Ok(())
    }
}
