use crate::terminal::component::App;
use crate::terminal::echo::Echo;
use crate::terminal::style::QwxStyle;
use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::event::Event;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::read;
use crossterm::execute;
use crossterm::queue;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use std::io::Result;
use std::io::Write;

#[derive(Debug)]
pub struct QwxEvent {
    pub key: Option<KeyEvent>,
    pub paste: Option<String>,
}

impl Drop for QwxEvent {
    fn drop(&mut self) {
        self.key = None;
        self.paste = None;
    }
}

pub struct QwxTerminal {
    width: u16,
    height: u16,
    event: Option<Event>,
    style: QwxStyle,
}
impl Drop for QwxTerminal {
    fn drop(&mut self) {
        self.width = 0;
        self.height = 0;
        self.event = None;
        self.style = QwxStyle {
            fg: None,
            bg: None,
            attr: None,
        };
    }
}
impl QwxTerminal {
    #[must_use]
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            event: None,
            style: QwxStyle {
                fg: None,
                bg: None,
                attr: None,
            },
        }
    }
    pub fn run<A: App, W: Write>(&mut self, w: &mut W, app: &mut A) -> Result<()> {
        let echo = Echo;
        // 1. On récupère la taille actuelle de la fenêtre
        let window = crossterm::terminal::window_size().expect("Failed to get window size");

        // 2. On dessine l'application
        app.render(w, &echo, &window)?;
        w.flush()?;

        // 3. On écoute les événements
        match read().expect("Failed to read event") {
            Event::Mouse(_e) => {}
            Event::Paste(x) => {
                let evt = QwxEvent {
                    key: None,
                    paste: Some(x),
                };
                app.on_event(&evt);
            }
            Event::Key(e) => {
                if e.kind == KeyEventKind::Press {
                    let evt = QwxEvent {
                        key: Some(e),
                        paste: None,
                    };
                    app.on_event(&evt);
                }
            }
            Event::FocusLost => {}
            Event::FocusGained => {}
            Event::Resize(width, height) => {
                queue!(w,Clear(ClearType::All))?;
                self.resize(width, height);
            }
        }
        Ok(())
    }
    pub fn open<W: Write>(&mut self, w: &mut W) {
        enable_raw_mode().expect("failed to enable raw mode");
        execute!(w, EnterAlternateScreen, Hide).expect("failed to enter to ealternate screen");
    }

    pub fn close<W: Write>(&mut self, w: &mut W) {
        execute!(w, LeaveAlternateScreen, Show).expect("failed to quit");
        disable_raw_mode().expect("failed to disable raw mode");
    }
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }
}
