use crossterm::style::{Attribute, Color};

pub enum QwxDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
#[doc = "A struct representing a style for a terminal UI component."]
pub struct QwxStyle {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub attr: Option<Attribute>,
}

#[derive(Clone, Copy)]
#[doc = "A struct representing a set of borders for a terminal UI component."]
pub struct QwxBorders {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub horizontal: &'static str,
    pub vertical: &'static str,
}

impl QwxBorders {
    /// A pre-defined set of rounded borders.
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
    /// A pre-defined set of double-line borders.
    pub const DOUBLE: Self = Self {
        top_left: "╔",
        top_right: "╗",
        bottom_left: "╚",
        bottom_right: "╝",
        horizontal: "═",
        vertical: "║",
    };
    /// A pre-defined set of heavy-line borders.
    pub const HEAVY: Self = Self {
        top_left: "┏",
        top_right: "┓",
        bottom_left: "┗",
        bottom_right: "┛",
        horizontal: "━",
        vertical: "┃",
    };
    /// A pre-defined set of dotted-line borders.
    pub const DOTTED: Self = Self {
        top_left: "┌",
        top_right: "┐",
        bottom_left: "└",
        bottom_right: "┘",
        horizontal: "┄",
        vertical: "┆",
    };
    //// A pre-defined set of dashed-line borders.
    pub const DASHED: Self = Self {
        top_left: "┌",
        top_right: "┐",
        bottom_left: "└",
        bottom_right: "┘",
        horizontal: "╌",
        vertical: "╎",
    };
    /// A pre-defined set of no borders (empty).
    pub const NONE: Self = Self {
        top_left: " ",
        top_right: " ",
        bottom_left: " ",
        bottom_right: " ",
        horizontal: " ",
        vertical: " ",
    };
}
