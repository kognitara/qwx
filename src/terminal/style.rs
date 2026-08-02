use crossterm::style::{Attribute, Color};

pub enum QwxDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub struct QwxStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub attr: Option<Attribute>,
}

#[derive(Clone, Copy)]
pub struct QwxBorders {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub horizontal: &'static str,
    pub vertical: &'static str,
}

impl QwxBorders {
    /// A pre-defined set of smooth, rounded borders often used in modern TUIs.
    pub const ROUNDED: Self = Self {
        top_left: "╭",
        top_right: "╮",
        bottom_left: "╰",
        bottom_right: "╯",
        horizontal: "─",
        vertical: "│",
    };

    /// A pre-defined set of sharp, classic rectangular borders.
    pub const SHARP: Self = Self {
        top_left: "┌",
        top_right: "┐",
        bottom_left: "└",
        bottom_right: "┘",
        horizontal: "─",
        vertical: "│",
    };
}
