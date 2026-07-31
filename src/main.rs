use crossterm::event::{KeyCode, KeyModifiers};
use crossterm::style::{Attribute, Color};
use crossterm::terminal::WindowSize;
use qwx::terminal::core::QwxEvent;
use qwx::terminal::core::QwxTerminal;
use qwx::terminal::echo::Echo;
use qwx::terminal::style::QwxStyle;
use std::io::{Result, Write, stdout};
pub mod terminal;

fn draw<W: Write>(
    w: &mut W,
    window: &WindowSize,
    _k: Option<KeyCode>,
    _m: Option<KeyModifiers>,
    _e: QwxEvent,
    echo: Echo,
) -> Result<()> {
    echo.rect(
        w,
        0,
        0,
        window.columns,
        window.rows,
        qwx::terminal::style::QwxBorders::ROUNDED,
        QwxStyle {
            fg: Some(Color::Blue),
            bg: Some(Color::Black),
            attr: Some(Attribute::Bold),
        },
    )?;
    w.flush()?;
    Ok(())
}

fn main() {
    let mut w = stdout();
    let window = crossterm::terminal::window_size().expect("");
    QwxTerminal::new(window.rows, window.columns)
        .open(&mut w)
        .interact(&mut w, &window, KeyCode::F(12), draw)
        .close(&mut w);
}
