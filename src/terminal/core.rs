use crate::terminal::echo::Echo;
use crate::terminal::style::QwxStyle;
use crossterm::cursor::Hide;
use crossterm::cursor::Show;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use crossterm::event::MouseEvent;
use crossterm::event::read;
use crossterm::execute;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use crossterm::terminal::WindowSize;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use std::io::Result;
use std::io::Write;

#[derive(Debug)]
pub struct QwxEvent {
    key: Option<KeyEvent>,
    mouse: Option<MouseEvent>,
    paste: Option<String>,
}

impl Drop for QwxEvent {
    fn drop(&mut self) {
        self.key = None;
        self.mouse = None;
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
        callback: fn(
            &mut W,
            &WindowSize,
            Option<KeyCode>,
            Option<KeyModifiers>,
            QwxEvent,
            Echo,
        ) -> Result<()>,
    ) -> &mut Self {
        loop {
            if callback(
                w,
                window,
                None,
                None,
                QwxEvent {
                    mouse: None,
                    key: None,
                    paste: None,
                },
                Echo,
            )
            .is_err()
            {
                break;
            };
            match read().expect("") {
                Event::Mouse(_e) => {}
                Event::Paste(x) => {
                    if callback(
                        w,
                        window,
                        None,
                        None,
                        QwxEvent {
                            mouse: None,
                            key: None,
                            paste: Some(x),
                        },
                        Echo,
                    )
                    .is_err()
                    {
                        break;
                    }
                }

                Event::Key(e) => {
                    if e.kind != KeyEventKind::Press {
                        continue;
                    }

                    if e.code == exit_code {
                        break;
                    } else {
                        if callback(
                            w,
                            window,
                            Some(e.code),
                            Some(e.modifiers),
                            QwxEvent {
                                mouse: None,
                                key: Some(e),
                                paste: None,
                            },
                            Echo,
                        )
                        .is_err()
                        {
                            break;
                        }
                    }
                }
                Event::FocusLost => {}
                Event::FocusGained => {}
                Event::Resize(width, height) => {
                    self.resize(width, height);
                }
            }
        }
        self
    }
}
