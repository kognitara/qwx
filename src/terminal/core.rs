use crate::terminal::component::Component;
use crate::terminal::echo::Echo;
use crate::terminal::style::QwxStyle;
use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::MouseEvent;
use crossterm::event::read;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::WindowSize;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use std::io::Write;

#[derive(Debug)]
pub struct QwxEvent {
    key: Option<KeyEvent>,
    mouse: Option<MouseEvent>,
    paste: Option<String>,
    focus: Option<bool>,
    lost_focus: Option<bool>,
    resize: Option<(u16, u16)>,
}

impl Drop for QwxEvent {
    fn drop(&mut self) {
        self.key = None;
        self.mouse = None;
        self.paste = None;
        self.focus = None;
        self.lost_focus = None;
        self.resize = None;
    }
}

pub struct QwxTerminal {
    width: u16,
    height: u16,
    event: Option<Event>,
    style: QwxStyle,
    components: Vec<Box<dyn Component>>,
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
        self.components.clear();
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
            components: Vec::new(),
        }
    }
    pub fn add(&mut self, mut component: Box<dyn Component>) -> &mut Self {
        component.on_mount(); // On déclenche le hook au montage
        self.components.push(component);
        self
    }
    pub fn open<W: Write>(&mut self, w: &mut W) -> &mut Self {
        enable_raw_mode().expect("failed to enable raw mode");
        execute!(w, EnterAlternateScreen, Hide).expect("failed to enter to ealternate screen");
        self
    }

    pub fn close<W: Write>(&mut self, w: &mut W) {
        // Ajout de "Show"
        execute!(w, LeaveAlternateScreen, Show).expect("failed to quit");
        disable_raw_mode().expect("failed to disable raw mode");
    }
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }

    pub fn interact<W: Write>(
        &mut self,
        w: &mut W,
        window: &WindowSize,
        exit_code: KeyCode,
    ) -> &mut Self {
        let echo = Echo;

        loop {
            let e = match read().expect("") {
                Event::Key(e) => QwxEvent {
                    key: Some(e),
                    mouse: None,
                    paste: None,
                    focus: None,
                    lost_focus: None,
                    resize: None,
                },
                Event::Mouse(e) => QwxEvent {
                    key: None,
                    mouse: Some(e),
                    paste: None,
                    focus: None,
                    lost_focus: None,
                    resize: None,
                },
                Event::Paste(x) => QwxEvent {
                    key: None,
                    mouse: None,
                    paste: Some(x),
                    focus: None,
                    lost_focus: None,
                    resize: None,
                },
                Event::FocusLost => QwxEvent {
                    key: None,
                    mouse: None,
                    paste: None,
                    focus: None,
                    lost_focus: Some(true),
                    resize: None,
                },
                Event::FocusGained => QwxEvent {
                    key: None,
                    mouse: None,
                    paste: None,
                    focus: Some(true),
                    lost_focus: None,
                    resize: None,
                },
                Event::Resize(cols, rows) => QwxEvent {
                    key: None,
                    mouse: None,
                    paste: None,
                    focus: None,
                    lost_focus: None,
                    resize: Some((cols, rows)),
                },
            };
            if e.key.is_some() && e.key.expect("").code.eq(&exit_code) {
                return self;
            } else {
                for comp in &self.components {
                    // Le `w as &mut dyn Write` convertit la référence générique
                    // en référence dynamique (vtable) à la volée !
                    let _ = comp.render(
                        w as &mut dyn Write,
                        &echo,
                        0,
                        0,
                        window.columns,
                        window.rows,
                    );
                }
            }
        }
    }
}
