use crossterm::event::KeyCode;
use qwx::terminal::core::QwxTerminal;
use std::io::stdout;

pub mod terminal;
// Tu importeras ton Finder ici quand il implémentera le trait Component
// use crate::finder::Finder;

fn main() {
    let mut w = stdout();
    let window = crossterm::terminal::window_size().expect("");

    let mut t = QwxTerminal::new(window.rows, window.columns);

    t.open(&mut w)
        .interact(&mut w, &window, KeyCode::F(12))
        .close(&mut w);
}
