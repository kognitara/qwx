use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[doc = "Module for customize `Qwx` editor"]
pub mod editor;
#[doc = "`Qwx` editor finder"]
pub mod finder;
#[doc = "`Qwx` module for draw ui"]
pub mod painter;
pub mod player;
#[doc = "`Qwx` module for search hub"]
pub mod search;
#[doc = "`Qwx` module for web search"]
pub mod web;

pub type KeyMap = HashMap<(KeyModifiers, KeyCode), Action>;
pub fn parse_key(s: &str) -> (KeyModifiers, KeyCode) {
    match s {
        "esc" => (KeyModifiers::NONE, KeyCode::Esc),
        "enter" => (KeyModifiers::NONE, KeyCode::Enter),
        "backtab" => (KeyModifiers::NONE, KeyCode::BackTab),
        "tab" => (KeyModifiers::NONE, KeyCode::Tab),
        "backspace" => (KeyModifiers::NONE, KeyCode::Backspace),
        "left" => (KeyModifiers::NONE, KeyCode::Left),
        "right" => (KeyModifiers::NONE, KeyCode::Right),
        "up" => (KeyModifiers::NONE, KeyCode::Up),
        "down" => (KeyModifiers::NONE, KeyCode::Down),
        "home" => (KeyModifiers::NONE, KeyCode::Home),
        "end" => (KeyModifiers::NONE, KeyCode::End),
        "delete" => (KeyModifiers::NONE, KeyCode::Delete),
        "insert" => (KeyModifiers::NONE, KeyCode::Insert),
        "pageup" => (KeyModifiers::NONE, KeyCode::PageUp),
        "pagedown" => (KeyModifiers::NONE, KeyCode::PageDown),
        "space" => (KeyModifiers::NONE, KeyCode::Char(' ')),

        // Gestion récursive propre des modificateurs (supporte "C-space" !)
        k if k.starts_with("C-") => {
            let (_, code) = parse_key(&k[2..]);
            (KeyModifiers::CONTROL, code)
        }
        k if k.starts_with("A-") => {
            let (_, code) = parse_key(&k[2..]);
            (KeyModifiers::ALT, code)
        }

        // Parse proprement les touches F-1 à F-12
        k if k.starts_with("F-") => {
            if let Ok(num) = k[2..].parse::<u8>() {
                (KeyModifiers::NONE, KeyCode::F(num))
            } else {
                panic!("Touche F- invalide : {}", k);
            }
        }

        // Caractères simples (a, b, 1, 2, -, +)
        k if k.chars().count() == 1 => {
            (KeyModifiers::NONE, KeyCode::Char(k.chars().next().unwrap()))
        }

        _ => panic!("Touche invalide dans la configuration : {}", s),
    }
}
pub fn build_keymap(config: &QwxConfig) -> KeyMap {
    let mut map = HashMap::new();
    map.insert(parse_key(&config.keys.move_down), Action::MoveDown);
    map.insert(parse_key(&config.keys.move_up), Action::MoveUp);
    map.insert(parse_key(&config.keys.move_left), Action::MoveLeft);
    map.insert(parse_key(&config.keys.move_right), Action::MoveRight);
    map.insert(parse_key(&config.keys.pageup), Action::PageUp);
    map.insert(parse_key(&config.keys.pagedown), Action::PageDown);
    map.insert(parse_key(&config.keys.select_line), Action::SelectLine);
    map.insert(parse_key(&config.keys.delete_line), Action::DeleteLine);
    map.insert(
        parse_key(&config.keys.delete_selection),
        Action::DeleteSelection,
    );
    map.insert(parse_key(&config.keys.undo), Action::Undo);
    map.insert(parse_key(&config.keys.redo), Action::Redo);
    map.insert(parse_key(&config.keys.yank), Action::Yank);
    map.insert(parse_key(&config.keys.paste), Action::Paste);
    map.insert(parse_key(&config.keys.toggle_facet), Action::ToggleFacet);
    map.insert(
        parse_key(&config.keys.show_front_facet),
        Action::ShowFrontFacet,
    );
    map.insert(
        parse_key(&config.keys.show_back_facet),
        Action::ShowBackFacet,
    );
    map.insert(parse_key(&config.keys.go_top), Action::GoTop);
    map.insert(parse_key(&config.keys.go_end), Action::GoEnd);
    map.insert(
        parse_key(&config.keys.go_bottom_panel),
        Action::GoBottomPanel,
    );
    map.insert(parse_key(&config.keys.go_top_panel), Action::GotTopPanel);
    map.insert(parse_key(&config.keys.go_left_panel), Action::GoLeftPanel);
    map.insert(parse_key(&config.keys.go_right_panel), Action::GoRightPanel);
    map.insert(parse_key(&config.keys.open_finder), Action::OpenFinder);
    map.insert(parse_key(&config.keys.edit), Action::Edit);
    map.insert(parse_key(&config.keys.edit_new_line), Action::EditNewLine);
    map.insert(parse_key(&config.keys.open_menu), Action::OpenMenu);
    map.insert(parse_key(&config.keys.search), Action::Search);
    map.insert(parse_key(&config.keys.open_web), Action::OpenWeb);
    map.insert(parse_key(&config.keys.open_player), Action::OpenPlayer);
    map.insert(parse_key(&config.keys.quit), Action::Quit);
    map.insert(
        parse_key(&config.keys.rotate_clockwise),
        Action::PanelRotateClockwise,
    );
    map.insert(
        parse_key(&config.keys.rotate_counter_clockwise),
        Action::PanelRotateCounterClockwise,
    );
    map.insert(parse_key(&config.keys.exit_mode), Action::ExitMode);
    map.insert(parse_key(&config.keys.save_document), Action::SaveDocument);
    map.insert(parse_key(&config.keys.increase_view), Action::IncreaseView);
    map.insert(parse_key(&config.keys.decrease_view), Action::DecreaseView);
    map.insert(
        parse_key(&config.keys.increase_workspace),
        Action::IncreaseWorkspace,
    );
    map.insert(
        parse_key(&config.keys.decrease_workspace),
        Action::DecreaseWorkspace,
    );
    map.insert(parse_key(&config.keys.pageup), Action::PageUp);
    map.insert(parse_key(&config.keys.pagedown), Action::PageDown);
    map
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    PageUp,
    PageDown,
    MoveUp,
    MoveDown,
    MoveLeft,
    MoveRight,
    SelectLine,
    DeleteLine,
    Undo,
    Redo,
    Yank,
    Paste,
    DeleteSelection,
    ToggleFacet,
    ShowFrontFacet,
    ShowBackFacet,
    PanelRotateClockwise,
    PanelRotateCounterClockwise,
    GoTop,
    GoEnd,
    GoBottomPanel,
    GotTopPanel,
    GoLeftPanel,
    GoRightPanel,
    OpenFinder,
    Edit,
    EditNewLine,
    OpenMenu,
    OpenWeb,
    OpenPlayer,
    Search,
    Quit,
    SaveDocument,
    ExitMode,
    IncreaseView,
    IncreaseWorkspace,
    DecreaseView,
    DecreaseWorkspace,
}
#[derive(Deserialize, Serialize, Clone)]
pub struct QwxKeysConfig {
    decrease_view: String,
    increase_view: String,
    decrease_workspace: String,
    increase_workspace: String,
    pageup: String,
    pagedown: String,
    move_down: String,
    move_left: String,
    move_right: String,
    move_up: String,
    select_line: String,
    delete_line: String,
    redo: String,
    undo: String,
    yank: String,
    paste: String,
    toggle_facet: String,
    show_front_facet: String,
    show_back_facet: String,
    go_top: String,
    go_end: String,
    go_bottom_panel: String,
    go_top_panel: String,
    go_right_panel: String,
    go_left_panel: String,
    open_finder: String,
    edit: String,
    edit_new_line: String,
    open_menu: String,
    search: String,
    open_web: String,
    open_player: String,
    quit: String,
    rotate_clockwise: String,
    rotate_counter_clockwise: String,
    exit_mode: String,
    save_document: String,
    delete_selection: String,
}
#[derive(Deserialize, Serialize, Clone)]
pub struct QwxConfig {
    keys: QwxKeysConfig,
}
