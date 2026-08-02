use crossterm::event::KeyCode;
use crossterm::style::{Attribute, Color};
use crossterm::terminal::{WindowSize, window_size};
use qwx::finder::search::FinderSearch;
use qwx::fs::QwxFileSystem;
use qwx::terminal::component::{App, Editor, Finder};
use qwx::terminal::core::{QwxEvent, QwxTerminal};
use qwx::terminal::echo::Echo;
use qwx::terminal::style::{QwxBorders, QwxStyle};
use std::io::{Result, Write, stdout};
use std::path::Path;
pub mod editor;
pub mod finder;
pub mod terminal;

#[derive(Clone)]
pub struct Zuu {
    pub should_quit: bool, // Demain, on mettra ici :
    pub fs: QwxFileSystem,
    pub search_state: FinderSearch,
    pub finder_selected_index: usize,
    pub is_finder_open: bool,
    pub terminal: QwxTerminal,
    // L'état de l'Éditeur
    pub current_file: Option<String>,
    pub editor_cursor_x: usize,
    pub editor_cursor_y: usize,
}

impl Finder for Zuu {
    fn find(&mut self, query: &str) {
        todo!()
    }

    fn finder_results(&self) -> Vec<String> {
        todo!()
    }

    fn finder_layout(&self) -> qwx::finder::layout::FinderLayout {
        todo!()
    }

    fn set_finder_layout(&mut self, layout: qwx::finder::layout::FinderLayout) {
        todo!()
    }

    fn finder_search_kind(&self) -> qwx::finder::search::FinderSearchKind {
        todo!()
    }

    fn finder_search_order(&self) -> qwx::finder::search::FinderSearchOrder {
        todo!()
    }

    fn finder_filter_kind(&self) -> qwx::finder::search::FilterKind {
        todo!()
    }

    fn set_finder_search_kind(&mut self, kind: qwx::finder::search::FinderSearchKind) {
        todo!()
    }

    fn set_finder_search_order(&mut self, order: qwx::finder::search::FinderSearchOrder) {
        todo!()
    }

    fn set_finder_filter_kind(&mut self, kind: qwx::finder::search::FilterKind) {
        todo!()
    }

    fn finder_capture(&mut self, input: char) {
        todo!()
    }

    fn finder_next_result(&mut self) {
        todo!()
    }

    fn finder_previous_result(&mut self) {
        todo!()
    }

    fn finder_get_selected(&self) -> Option<String> {
        todo!()
    }
}

impl Editor for Zuu {
    fn editor_insert_char(&mut self, c: char) {
        todo!()
    }

    fn editor_backspace(&mut self) {
        todo!()
    }

    fn editor_delete_char(&mut self) {
        todo!()
    }

    fn editor_scroll(&mut self, delta_y: isize) {
        todo!()
    }

    fn editor_get_viewport(&self) -> (usize, usize) {
        todo!()
    }

    fn editor_move_cursor(&mut self, dx: isize, dy: isize) {
        todo!()
    }

    fn editor_select_line(&mut self) {
        todo!()
    }

    fn editor_insert_line(&mut self, line: &str) {
        todo!()
    }

    fn editor_delete_line(&mut self, line_number: usize) {
        todo!()
    }

    fn editor_get_lines(&self) -> Vec<String> {
        todo!()
    }

    fn editor_open(&self, file: &std::path::Path) {
        todo!()
    }

    fn editor_save(&self, file: &std::path::Path) {
        todo!()
    }

    fn editor_close(&self, file: &std::path::Path) {
        todo!()
    }

    fn editor_undo(&mut self) {
        todo!()
    }

    fn editor_redo(&mut self) {
        todo!()
    }

    fn editor_cut(&mut self) {
        todo!()
    }

    fn editor_copy(&mut self) {
        todo!()
    }

    fn editor_paste(&mut self) {
        todo!()
    }

    fn editor_find(&mut self, query: &str) {
        todo!()
    }

    fn editor_replace(&mut self, query: &str, replacement: &str) {
        todo!()
    }

    fn editor_replace_all(&mut self, query: &str, replacement: &str) {
        todo!()
    }

    fn editor_get_cursor_position(&self) -> (usize, usize) {
        todo!()
    }

    fn editor_set_cursor_position(&mut self, line: usize, column: usize) {
        todo!()
    }

    fn editor_get_selection(&self) -> Option<(usize, usize, usize, usize)> {
        todo!()
    }

    fn editor_set_selection(
        &mut self,
        start_line: usize,
        start_column: usize,
        end_line: usize,
        end_column: usize,
    ) {
        todo!()
    }
}
// 2. On implémente le comportement de notre application
impl App for Zuu {
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

impl Zuu {
    pub fn new<P: AsRef<Path>>(p: P, window: &WindowSize) -> Self {
        Self {
            should_quit: false,
            fs: QwxFileSystem::new(&p),
            search_state: FinderSearch::default(),
            finder_selected_index: 0,
            is_finder_open: false,
            current_file: None,
            editor_cursor_x: 0,
            editor_cursor_y: 0,
            terminal: QwxTerminal::new(window.columns, window.rows),
        }
    }
    pub fn run(&mut self) {
        let mut w = stdout();
        let mut t = self.terminal.clone();
        t.open(&mut w);
        while !self.should_quit {
            let _ = t.run(&mut w, self);
        }
        t.close(&mut w);
    }
}
fn main() {
    Zuu::new(Path::new("."), &window_size().expect("msg")).run();
}
