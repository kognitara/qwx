use crossterm::event::KeyCode;
use crossterm::style::{Attribute, Color};
use crossterm::terminal::WindowSize;
use qwx::terminal::component::App;
use qwx::terminal::core::{QwxEvent, QwxTerminal};
use qwx::terminal::echo::Echo;
use qwx::terminal::style::{QwxBorders, QwxStyle};
use std::io::{Result, Write, stdout};
pub mod terminal;

// 1. Notre structure globale d'application
struct Workspace {
    pub should_quit: bool, // Demain, on mettra ici :
                           // pub finder: FinderState,
}

// 2. On implémente le comportement de notre application
impl App for Workspace {
    fn on_event(&mut self, event: &QwxEvent) {
        if let Some(key) = event.key {
            // Si on appuie sur F12 ou Esc, on retourne 'true' pour quitter la boucle
            if key.code == KeyCode::F(12) || key.code == KeyCode::Esc {
                self.should_quit = true;
            }
        }
    }

    fn render<W: Write>(&self, w: &mut W, echo: &Echo, window: &WindowSize) -> Result<()> {
        echo.rect(
            w,
            0,
            0,
            window.columns,
            window.rows,
            QwxBorders::ROUNDED,
            QwxStyle {
                fg: Some(Color::Blue),
                bg: Some(Color::Black),
                attr: Some(Attribute::Bold),
            },
        )?;
        Ok(())
    }
}

fn main() {
    let mut w = stdout();
    let window = crossterm::terminal::window_size().expect("");

    // On instancie notre application
    let mut app = Workspace { should_quit: false };

    // On lance la machine
    let mut t = QwxTerminal::new(window.rows, window.columns);
    t.open(&mut w);

    while app.should_quit.eq(&false) {
        // Le run remplace le interact !
        let _ = t.run(&mut w, &mut app);
    }
    t.close(&mut w);
}
