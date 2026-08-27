use crate::editor::theme::{
    UI_BORDER_ACTIVE, UI_BORDER_INACTIVE, UI_DMENU_BG, UI_DMENU_FG, UI_TEXT_MUTED,
    get_color_for_capture,
};
use crate::finder::{Finder, FinderLayout, list_files};
use crate::player::MusicPlayer;
use crossterm::cursor::{
    Hide, MoveDown, MoveLeft, MoveRight, MoveTo, MoveUp, SetCursorStyle, Show,
};
use crossterm::event::{Event, KeyCode, KeyModifiers, poll, read};
use crossterm::style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor};
use crossterm::terminal::{
    self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, size,
};
use crossterm::{execute, queue};
use ropey::Rope;
use std::fs::{File, create_dir_all};
use std::io::{self, BufRead, BufReader, Error, Write, stdout};
use std::path::Path;
use std::path::PathBuf;
use tree_sitter::{InputEdit, Language, Point, QueryCursor};
use tree_sitter::{Parser, Tree};
use tree_sitter::{Query, StreamingIterator};
use tree_sitter_highlight::HighlightConfiguration;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub mod theme;
/// Represents the initial state of a `PaneState` in the application.
///
/// This constant defines default values for the pane's configuration:
/// - `workspace`: The default workspace index, set to `1`.
/// - `view`: The default view index, set to `1`.
/// - `cursor`: The initial cursor position, set to `0`.
///
/// # Example
/// ```rust
/// use qwx::editor::INIT_PANE_STATE;
/// let pane_state = INIT_PANE_STATE;
/// assert_eq!(pane_state.workspace, 1);
/// assert_eq!(pane_state.view, 1);
/// assert_eq!(pane_state.cursor, 0);
/// ```
///
/// This constant can be used to initialize or reset pane states in the system.
pub const INIT_PANE_STATE: PaneState = PaneState {
    workspace: 1,
    view: 1,
    cursor: 0,
};

/// Converts a numerical value (0-9) into its corresponding superscript Unicode character.
///
/// # Parameters
/// - `num`: A `u8` value representing the number to be converted to superscript. Valid input is between 0 and 9.
///
/// # Returns
/// A string slice (`&'static str`) representing the superscript Unicode character for the given number:
/// - `1` → "¹"
/// - `2` → "²"
/// - `3` → "³"
/// - `4` → "⁴"
/// - `5` → "⁵"
/// - `6` → "⁶"
/// - `7` → "⁷"
/// - `8` → "⁸"
/// - `9` → "⁹"
/// - Any other value (including 0) → "⁰"
///
/// # Examples
/// ```
/// use qwx::editor::get_superscript;
/// let result = get_superscript(3);
/// assert_eq!(result, "³");
///
/// let result = get_superscript(9);
/// assert_eq!(result, "⁹");
///
/// let result = get_superscript(0);
/// assert_eq!(result, "⁰");
/// ```
pub fn get_superscript(num: u8) -> &'static str {
    match num {
        1 => "¹",
        2 => "²",
        3 => "³",
        4 => "⁴",
        5 => "⁵",
        6 => "⁶",
        7 => "⁷",
        8 => "⁸",
        9 => "⁹",
        _ => "⁰",
    }
}
/// The `QwxUi` trait defines a user interface element that can be drawn to a given writer.
///
/// # Type Parameters
/// - `W`: A type that implements the `Write` trait, representing the output stream where
///        the UI element will be rendered.
///
/// # Required Methods
/// - `draw`:
///   - Draws the user interface element to the specified writer.
///   - Accepts a mutable reference to the writer where the content should be drawn.
///   - Returns a `Result` indicating success (`Ok(())`) or an error (`Err(Error)`).
///
/// # Errors
/// - If an error occurs while writing to the given writer, the method can return an `Error`.
///
/// # Example
/// ```
/// use std::io::{Write, Error};
/// use qwx::editor::QwxUi;
/// struct MyUiElement;
///
/// impl<W: Write> QwxUi<W> for MyUiElement {
///     fn draw(&mut self, w: &mut W) -> Result<(), Error> {
///         write!(w, "Drawing MyUiElement...")?;
///         Ok(())
///     }
/// }
///
/// let mut output = Vec::new(); // Example writer
/// let mut ui_element = MyUiElement;
/// ui_element.draw(&mut output).unwrap();
/// println!("{}", String::from_utf8(output).unwrap());
/// ```
pub trait QwxUi<W: Write> {
    fn draw(&mut self, w: &mut W) -> Result<(), Error>;
}

impl<W: Write> QwxUi<W> for Qwx {
    fn draw(&mut self, w: &mut W) -> Result<(), Error> {
        if self.mode == Mode::WebSearch {
            self.search_hub.draw(w, 0, 0, self.width, self.height)?;
            w.flush()?;
            return Ok(());
        }
        if self.mode == Mode::Player {
            self.player.draw_player(w, self.width, self.height)?;
            w.flush()?;
            return Ok(());
        }
        execute!(w, Hide)?;
        let max_width = 180.min(self.width);

        let left_x = (self.width.saturating_sub(max_width)) / 2;

        let right_x = left_x + max_width.saturating_sub(1);
        let mid_x = left_x + (max_width / 2);

        let top_y = 0;
        let bottom_y = self.height.saturating_sub(1);
        let mid_y = self.height / 2;

        let horiz_line = "─".repeat(max_width.saturating_sub(2) as usize);

        queue!(
            w,
            MoveTo(left_x, top_y),
            SetForegroundColor(UI_BORDER_INACTIVE),
            Print(format!("┌{}┐", horiz_line)),
            MoveTo(left_x, bottom_y),
            Print(format!("└{}┘", horiz_line))
        )?;

        // Vertical extern right and left
        for y in (top_y + 1)..bottom_y {
            if y != mid_y {
                queue!(
                    w,
                    MoveTo(left_x, y),
                    SetForegroundColor(UI_BORDER_INACTIVE),
                    Print("│"),
                    MoveTo(right_x, y),
                    Print("│")
                )?;
            }
        }

        // Horizontal separator line
        for x in (left_x + 1)..right_x {
            if x != mid_x {
                queue!(
                    w,
                    MoveTo(x, mid_y),
                    SetForegroundColor(UI_BORDER_INACTIVE),
                    Print("─")
                )?;
            }
        }

        // Vertical separator line
        for y in (top_y + 1)..bottom_y {
            if y != mid_y {
                queue!(
                    w,
                    MoveTo(mid_x, y),
                    SetForegroundColor(UI_BORDER_INACTIVE),
                    Print("│")
                )?;
            }
        }

        // Intersections
        queue!(
            w,
            SetForegroundColor(UI_BORDER_INACTIVE),
            MoveTo(left_x, mid_y),
            Print("├"),
            MoveTo(right_x, mid_y),
            Print("┤"),
            MoveTo(mid_x, top_y),
            Print("┬"),
            MoveTo(mid_x, bottom_y),
            Print("┴"),
            MoveTo(mid_x, mid_y),
            Print("┼")
        )?;

        let panes_bounds = [
            (
                PaneFocus::TopLeft,
                left_x + 1,
                top_y + 1,
                (mid_x - left_x).saturating_sub(1),
                (mid_y - top_y).saturating_sub(1),
            ),
            (
                PaneFocus::TopRight,
                mid_x + 1,
                top_y + 1,
                (right_x - mid_x).saturating_sub(1),
                (mid_y - top_y).saturating_sub(1),
            ),
            (
                PaneFocus::BottomLeft,
                left_x + 1,
                mid_y + 1,
                (mid_x - left_x).saturating_sub(1),
                (bottom_y - mid_y).saturating_sub(1),
            ),
            (
                PaneFocus::BottomRight,
                mid_x + 1,
                mid_y + 1,
                (right_x - mid_x).saturating_sub(1),
                (bottom_y - mid_y).saturating_sub(1),
            ),
        ];

        // 3. Draw the content of each pane
        for (i, &(pane_focus, start_x, start_y, p_width, p_height)) in
            panes_bounds.iter().enumerate()
        {
            let pane = self.panes[i];
            let is_active = self.focus == pane_focus;

            if let Some(view) = self.views.get(i)
                && let Some(node) = self.nodes.iter().find(|n| n.id == view.active_node_id)
                && node.is_file
            {
                let selection =
                    if is_active && (self.mode == Mode::Editor || self.mode == Mode::Normal) {
                        self.editor.selection
                    } else {
                        None
                    };
                let _ = self.preview(
                    node,
                    start_x,
                    start_y,
                    p_width,
                    p_height,
                    pane.cursor as usize,
                    selection,
                );
            }

            let percentage_str = if let Some(view) = self.views.get(i)
                && let Some(node) = self.nodes.iter().find(|n| n.id == view.active_node_id)
                && node.is_file
            {
                let len = node.content.len();
                if len <= 1 {
                    100
                } else {
                    ((pane.cursor as usize * 100) / (len - 1)).min(100)
                }
            } else {
                0
            };
            let expo = get_superscript(pane.view);
            let dirty_prefix = if is_active && self.editor.is_dirty {
                "*"
            } else {
                ""
            };
            let info_display = format!(
                "{}{} % {}{}",
                dirty_prefix, percentage_str, pane.workspace, expo
            );
            let indicator_x = start_x + p_width.saturating_sub(info_display.len() as u16);
            let indicator_y = start_y + p_height.saturating_sub(1);
            queue!(w, MoveTo(indicator_x, indicator_y))?;
            if is_active {
                queue!(
                    w,
                    SetForegroundColor(UI_BORDER_ACTIVE),
                    Print(format!("{}{} % ", dirty_prefix, percentage_str)),
                    Print(pane.workspace),
                    Print(expo)
                )?;
            } else {
                queue!(
                    w,
                    SetForegroundColor(UI_TEXT_MUTED),
                    Print(format!("{} % ", percentage_str)),
                    Print(pane.workspace),
                    Print(expo)
                )?;
            }
        }

        if self.is_finder_open() {
            self.draw_finder(w)?;
        } else if self.mode == Mode::Menu {
            let (start_x, start_y, pane_width) = match self.focus {
                PaneFocus::TopLeft => (left_x, top_y, (mid_x - left_x)),
                PaneFocus::TopRight => (mid_x + 1, top_y, (right_x - mid_x)),
                PaneFocus::BottomLeft => (left_x, mid_y + 1, (mid_x - left_x)),
                PaneFocus::BottomRight => (mid_x + 1, mid_y + 1, (right_x - mid_x)),
            };

            let prompt = format!(" {} ", self.menu_input);
            let padded_prompt = format!("{:<width$}", prompt, width = pane_width as usize);

            queue!(
                w,
                MoveTo(start_x, start_y),
                SetBackgroundColor(UI_DMENU_BG),
                SetForegroundColor(UI_DMENU_FG),
                Print(padded_prompt),
                ResetColor
            )?;
        } else if self.mode == Mode::Search {
            let (start_x, start_y, pane_width) = match self.focus {
                PaneFocus::TopLeft => (left_x, top_y, (mid_x - left_x)),
                PaneFocus::TopRight => (mid_x + 1, top_y, (right_x - mid_x)),
                PaneFocus::BottomLeft => (left_x, mid_y + 1, (mid_x - left_x)),
                PaneFocus::BottomRight => (mid_x + 1, mid_y + 1, (right_x - mid_x)),
            };

            let prompt = format!(" /{} ", self.search_input);
            let padded_prompt = format!("{:<width$}", prompt, width = pane_width as usize);
            queue!(
                w,
                MoveTo(start_x, start_y),
                SetBackgroundColor(UI_DMENU_BG),
                SetForegroundColor(UI_DMENU_FG),
                Print(padded_prompt),
                ResetColor
            )?;
        }

        if self.mode == Mode::Editor || self.mode == Mode::Normal {
            queue!(w, Show)?;

            if self.mode == Mode::Editor {
                queue!(w, SetCursorStyle::SteadyBlock)?;
            } else {
                queue!(w, SetCursorStyle::SteadyUnderScore)?;
            }

            let active_bounds = panes_bounds
                .iter()
                .find(|(focus, _, _, _, _)| *focus == self.focus);

            if let Some(&(_, start_x, start_y, p_width, p_height)) = active_bounds {
                let active_pane = self.panes[self.focus as usize];
                let scroll_y = active_pane.cursor as usize;
                let line_idx = self.editor.cursor_line;
                let col_idx = self.editor.cursor_col;
                if line_idx >= scroll_y && line_idx < scroll_y + (p_height as usize) {
                    let screen_y = start_y + (line_idx - scroll_y) as u16;
                    let screen_x = start_x + (col_idx as u16).min(p_width.saturating_sub(1));

                    queue!(w, MoveTo(screen_x, screen_y))?;
                } else {
                    queue!(w, Hide)?;
                }
            }
        } else {
            queue!(w, Hide)?;
        }

        queue!(w, ResetColor)?;
        w.flush()?;
        Ok(())
    }
}

/// A trait for managing and navigating through a sequence of finder layouts.
///
/// The `QwxFinder` trait provides methods for transitioning between layouts in a sequence,
/// allowing forward and backward traversal. This trait is particularly useful for implementations
/// that involve dynamic layout switching, such as editors, UI/UX flows, or complex state systems.
pub trait QwxFinder {
    /// Updates the state by shifting the current layout to the previous one in a sequence of layouts.
    ///
    /// # Description
    /// This method is used to transition to the previous finder layout in a predefined sequence
    /// of layouts. It modifies the internal state of the object accordingly, ensuring that the
    /// layout reflects the one prior to the currently active layout. This can be useful in scenarios
    /// where navigating backward through a series of layouts is required (e.g., in UI/UX workflows
    /// or state management systems).
    ///
    /// # Notes
    /// - Make sure that the implementation handles edge cases, such as when the current layout is the
    ///   first one in the sequence.
    /// - Internal dependencies and state changes, if any, should be documented and handled appropriately.
    fn previous_finder_layout(&mut self);

    /// Updates the current finder layout to the next available layout configuration.
    ///
    /// # Description
    /// This method modifies the current finder layout to cycle through predefined layout configurations.
    /// It allows switching or progressing to the next layout arrangement, facilitating dynamic adjustments
    /// to the visual or functional setup of the finder component.
    ///
    /// # Behavior
    /// - The method maintains an internal state to track the current layout.
    /// - Upon invocation, it advances to the next layout option in the sequence.
    /// - If the last layout configuration in the sequence is currently active, this method wraps around
    ///   and returns to the first layout configuration.
    ///
    /// # Note
    /// Ensure the internal layout configurations have been properly initialized before calling this method
    /// to prevent any unexpected behavior.
    ///
    /// # Errors
    /// This method does not return any errors.
    fn next_finder_layout(&mut self);
}

impl QwxFinder for Qwx {
    fn previous_finder_layout(&mut self) {
        self.finder_layout = self.finder_layout.previous();
        self.finder.layout = self.finder_layout.clone();
    }

    fn next_finder_layout(&mut self) {
        self.finder_layout = self.finder_layout.next();
        self.finder.layout = self.finder_layout.clone();
    }
}

/// The `QwxPanel` trait defines the behavior and functionality for managing
/// and interacting with a panel that contains panes, specifically focusing
/// on loading files for the active pane and providing mutable access to it.
pub trait QwxPanel {
    /// Loads the file associated with the currently active pane.
    ///
    /// This function retrieves and processes the file linked to the active pane
    /// in the user interface. It typically refreshes or updates the displayed
    /// content to reflect the changes or current state of the file associated
    /// with that pane.
    ///
    /// # Parameters
    ///
    /// This is a method of a struct containing a mutable state, so it operates on `&mut self`.
    ///
    /// # Behavior
    ///
    /// - If there is no active pane, the function may do nothing or handle the
    ///   case accordingly (e.g., logging an error or providing a default behavior).
    /// - If the active pane has a file associated with it, the function ensures
    ///   that the file is properly loaded and its contents made available for
    ///   further operations.
    ///
    /// # Errors
    ///
    /// This function may handle errors internally or propagate them, such as if
    /// the file fails to load or is inaccessible. Refer to the implementation
    /// details for specific error handling strategies.
    fn load_active_pane_file(&mut self);
    /// Returns a mutable reference to the currently active pane's state.
    ///
    /// This function provides mutable access to the `PaneState` of the active pane,
    /// allowing modifications to its properties. The active pane refers to the
    /// currently selected or focused pane in the system.
    ///
    /// # Returns
    /// * `&mut PaneState` - A mutable reference to the state of the active pane.
    ///
    /// # Notes
    /// Ensure that no other mutable references to the active pane exist when this
    /// function is called, as per Rust's borrowing rules.
    fn active_pane_mut(&mut self) -> &mut PaneState;
}

impl QwxPanel for Qwx {
    fn load_active_pane_file(&mut self) {
        let active_idx = self.focus as usize;
        if let Some(view) = self.views.get(active_idx)
            && let Some(node) = self.nodes.get(view.active_node_id)
            && node.is_file
        {
            let full_path = self.current_dir.join(&node.name);
            if let Some(path_str) = full_path.to_str()
                && let Ok(editor) = Ji::open(path_str)
            {
                self.editor = editor;
                self.editor.cursor_line = self.panes[active_idx].cursor as usize;
                self.editor.cursor_col = 0;
            }
        }
    }

    fn active_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[self.focus as usize]
    }
}
/// Reads all lines from a file at the given path and returns them as a `Vec<String>`.
///
/// # Parameters
/// - `path`: The file system path to the file. It can be anything that implements the `AsRef<Path>` trait,
///           such as a `&str`, `String`, or `Path`.
///
/// # Returns
/// - `Ok(Vec<String>)`: A vector of strings, where each string represents a single line from the file.
/// - `Err(Error)`: An I/O error encountered during the file operation, except for invalid UTF-8 data errors,
///                 which are silently ignored.
///
/// # Behavior
/// - This function uses a buffered reader to read the file line by line.
/// - Lines that cannot be read due to invalid UTF-8 data are skipped without stopping the process.
/// - Any other types of errors, such as file permissions issues or file not found, cause the function to return an error.
///
/// # Example
/// ```
/// use std::path::Path;
/// use qwx::editor::qwx_read_lines;
/// let lines = qwx_read_lines(Path::new("example.txt"));
/// match lines {
///     Ok(lines) => {
///         for line in lines {
///             println!("{}", line);
///         }
///     }
///     Err(e) => eprintln!("Error reading file: {e}"),
/// }
/// ```
///
/// # Errors
/// The function propagates errors from the `File::open` method and the `BufReader` while reading the lines,
/// except when an error is caused by invalid UTF-8 data (`ErrorKind::InvalidData`), which is ignored.
pub fn qwx_read_lines(path: impl AsRef<Path>) -> Result<Vec<String>, Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = Vec::new();

    for line_result in reader.lines() {
        match line_result {
            Ok(line) => lines.push(line),
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                continue;
            }
            Err(e) => return Err(e), // On continue de propager les autres erreurs (ex: droits d'accès)
        }
    }

    Ok(lines)
}

/// Loads a `Node` from the given file path and assigns it a specific ID.
///
/// # Parameters
///
/// - `id`: The unique numeric identifier for the `Node`.
/// - `path`: A reference to the file path from which to load the `Node`.
///
/// # Returns
///
/// This function returns a `Result<Node, Error>`:
/// - `Ok(Node)` on successful creation of a `Node`.
/// - `Err(Error)` if there is an error reading the file or processing its contents.
///
/// # Functionality
///
/// - Reads the file at the specified `path` and extracts its lines into a vector of strings.
/// - The file name is extracted from the `path` and used as the name of the `Node`.
/// - Attempts to determine whether the `path` points to a valid file (`is_file` flag).
/// - If the file is valid, it uses the `Ji` library to parse it and generate syntax-colored spans (if possible).
///   - The spans are processed into a structure where each line is annotated with its corresponding syntax color.
/// - If syntax-colored spans could not be generated (e.g., empty file or no compatible Tree-sitter support),
///   assigns plain white-colored text as a fallback.
/// - Constructs and returns a `Node` instance with the following information:
///   - `id`: The provided ID.
///   - `name`: The file name derived from the `path`.
///   - `content`: The raw lines of text extracted from the file.
///   - `colored_lines`: Syntax-colored text or fallback white-colored text.
///   - `is_file`: Boolean indicating whether the path is a valid file.
///
/// # Errors
///
/// - Returns an error if the file cannot be read (e.g., due to missing permissions or invalid path).
///
/// # Examples
///
/// ```
/// use std::path::Path;
/// use qwx::editor::qwx_load_node;
/// let path = Path::new("example.txt");
/// let node = qwx_load_node(1, path);
/// match node {
///     Ok(n) => println!("Node loaded with name: {}", n.name),
///     Err(e) => eprintln!("Failed to load Node: {:?}", e),
/// }
/// ```
///
/// # Notes
///
/// - This function uses the `Ji` library to generate syntax-colored spans.
/// - If the file is empty or incompatible with Tree-sitter parsing, it defaults to displaying plain text in white color.
/// - Ensure that the `Color` type and `Ji` library functionalities (e.g., `open`, `get_colored_spans`) are properly implemented and accessible.
pub fn qwx_load_node(id: usize, path: &Path) -> Result<Node, Error> {
    let content = qwx_read_lines(path)?;
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default()
        .to_string();
    let is_file = path.is_file();

    let mut colored_lines = Vec::new();

    if is_file {
        if let Ok(temp_ji) = Ji::open(path) {
            let spans = temp_ji.get_colored_spans();

            // On ne peuple le cache que si Tree-sitter a vraiment renvoyé des couleurs
            if !spans.is_empty() {
                colored_lines.push(vec![]);
                for (text, color) in spans {
                    let mut is_first = true;
                    for part in text.split('\n') {
                        if !is_first {
                            colored_lines.push(vec![]);
                        }
                        if !part.is_empty() {
                            colored_lines
                                .last_mut()
                                .unwrap()
                                .push((part.to_string(), color));
                        }
                        is_first = false;
                    }
                }
            }
        }
    }

    // Fallback : Si le fichier est vide, ou qu'il n'y a pas de Tree-sitter pour lui, texte en blanc
    if colored_lines.is_empty() {
        for line in &content {
            colored_lines.push(vec![(line.clone(), Color::White)]);
        }
    }

    Ok(Node {
        id,
        name,
        content,
        colored_lines,
        is_file,
    })
}

/// The `Qwx` struct represents the core state and configuration of the application.
/// It encapsulates the layout, navigation structure, user inputs, and various modes to define the behavior and appearance of the system.
///
/// # Fields
///
/// * `finder_layout` - Defines the layout configuration of the finder, which determines how components are arranged visually.
/// * `finder` - Represents the main file navigation component used for browsing files and directories.
/// * `finder_research` - Holds the current search query string for the finder functionality.
/// * `nodes` - A collection of `Node` structures that represent the raw in-memory representation of the filesystem structure.
/// * `views` - A collection of `View` objects representing the active scrollable windows or panes in the application.
/// * `width` - The width of the application's available screen space, measured in character or pixel units.
/// * `height` - The height of the application's available screen space, measured in character or pixel units.
/// * `running` - A boolean flag indicating whether the application is currently running or should terminate.
/// * `current_dir` - A `Box<Path>` pointing to the current working directory within the application.
/// * `focus` - Specifies the current focus of the application, detailing which pane or area is currently active for user interactions.
/// * `panes` - An array of four `PaneState` objects representing the state of up to four panes, used for organizing views or sections of the application.
/// * `mode` - Represents the current operating mode of the application, which affects its behavior and user interface patterns.
/// * `menu_input` - Holds the input string provided by the user for dynamic menu operations.
/// * `editor` - Represents the `Ji` editor instance, which is used for any text editing functionalities within the application.
/// * `search_input` - Stores the user's input string for search operations within the application.
pub struct Qwx {
    pub finder_layout: FinderLayout,
    pub finder: Finder,
    pub finder_research: String,
    nodes: Vec<Node>,
    views: Vec<View>,
    width: u16,
    height: u16,
    running: bool,
    current_dir: Box<Path>,
    focus: PaneFocus,
    panes: [PaneState; 4],
    mode: Mode,
    menu_input: String,
    editor: Ji,
    search_input: String,
    pub last_search_query: Option<String>,
    pub search_hub: crate::search::SearchHub,
    pub player: MusicPlayer,
}

/// A `View` structure that represents the current state of a view in the application.
///
/// # Fields
/// * `active_node_id` - An identifier of type `usize` that represents the currently active node within the view.
///                      This field is publicly accessible and can be used to track or modify the active node as needed.
///
/// # Example
/// ```
/// use qwx::editor::View;
/// let view = View {
///     active_node_id: 42,
/// };
/// println!("Active node ID: {}", view.active_node_id);
/// ```
pub struct View {
    pub active_node_id: usize,
}
/// An enumeration representing possible directions for movement or orientation.
///
/// # Variants
///
/// - `Left`:
///   Represents movement or orientation to the left.
///
/// - `Right`:
///   Represents movement or orientation to the right.
///
/// - `Down`:
///   Represents movement or orientation downward.
///
/// - `Up`:
///   Represents movement or orientation upward.
///
/// - `Vertical`:
///   Represents a general vertical orientation, encompassing both `Up` and `Down`.
///
/// - `Horizontal`:
///   Represents a general horizontal orientation, encompassing both `Left` and `Right`.
///
/// # Examples
///
/// ```rust
/// use qwx::editor::QwxDirection;
///
/// let direction = QwxDirection::Left;
/// match direction {
///     QwxDirection::Left => println!("Going left!"),
///     QwxDirection::Up => println!("Going up!"),
///     _ => println!("Other direction."),
/// }
/// ```
pub enum QwxDirection {
    Left,
    Right,
    Down,
    Up,
    Vertical,
    Horizontal,
}

/// Represents the possible directions for a scroll action along with an associated value.
///
/// The `QwxScrollDirection` enum is designed to specify the direction of scrolling
/// and includes an associated `u16` value, which can represent details such as the
/// magnitude or units of scrolling. Each variant corresponds to a particular scrolling
/// direction.
///
/// # Variants
///
/// - `Left(u16)`
///     Represents scrolling to the left with an associated magnitude.
///
/// - `Right(u16)`
///     Represents scrolling to the right with an associated magnitude.
///
/// - `Down(u16)`
///     Represents scrolling downwards with an associated magnitude.
///
/// - `Up(u16)`
///     Represents scrolling upwards with an associated magnitude.
///
/// - `Vertical(u16)`
///     Represents vertical scrolling (either up or down) with an associated magnitude.
///
/// - `Horizontal(u16)`
///     Represents horizontal scrolling (either left or right) with an associated magnitude.
///
/// # Examples
///
/// ```rust
/// use qwx::editor::QwxScrollDirection;
///
/// let scroll_left = QwxScrollDirection::Left(10);
/// let scroll_down = QwxScrollDirection::Down(20);
/// ```
///
/// This enum can be used to control or represent scrolling behavior in a UI or other
/// directional context where quantified scrolling is required.
pub enum QwxScrollDirection {
    Left(u16),
    Right(u16),
    Down(u16),
    Up(u16),
    Vertical(u16),
    Horizontal(u16),
}
/// Represents the focus of a pane in a user interface or layout system.
///
/// The `PaneFocus` enum is used to signify the specific section of a pane
/// that currently has focus. It supports copying, cloning, and comparison
/// due to the derived traits.
///
/// # Variants
///
/// * `TopLeft` - Represents the top-left section of the pane.
/// * `TopRight` - Represents the top-right section of the pane.
/// * `BottomLeft` - Represents the bottom-left section of the pane.
/// * `BottomRight` - Represents the bottom-right section of the pane.
///
/// # Trait Implementations
///
/// * `Copy` - Allows for bitwise copying of the `PaneFocus` value.
/// * `Clone` - Enables the creation of a new instance of `PaneFocus`
///   with the same value as the original.
/// * `PartialEq` - Allows for comparison between two `PaneFocus`
///   instances to check for equality.
///
/// # Usage
///
/// The `PaneFocus` enum can be used in layout management or UI interactions
/// to track and modify the focus state of different sections of a pane.
///
/// ```rust
/// use qwx::editor::PaneFocus;
///
/// let focus = PaneFocus::TopLeft;
/// match focus {
///     PaneFocus::TopLeft => println!("Top-left pane is focused."),
///     PaneFocus::TopRight => println!("Top-right pane is focused."),
///     PaneFocus::BottomLeft => println!("Bottom-left pane is focused."),
///     PaneFocus::BottomRight => println!("Bottom-right pane is focused."),
/// }
/// ```
#[derive(Copy, Clone, PartialEq)]
pub enum PaneFocus {
    TopLeft = 0,
    TopRight = 1,
    BottomLeft = 2,
    BottomRight = 3,
}

/// Represents a node in a data structure that can either be a file or a directory.
///
/// # Fields
///
/// * `id` (`usize`) -
///   A unique identifier for the node. This can be used to uniquely distinguish nodes within a structure.
///
/// * `name` (`String`) -
///   The name of the node. Typically, represents the filename if `is_file` is true or the directory name otherwise.
///
/// * `content` (`Vec<String>`) -
///   A vector of strings containing the content of the node. This is typically used for file nodes to store each line of the file's text.
///
/// * `colored_lines` (`Vec<Vec<(String, Color)>>`) -
///   A vector of lines where each line contains colored text. Each line is represented as a vector of tuples,
///   where each tuple consists of a `String` (the text) and a `Color` (the color applied to the text).
///
/// * `is_file` (`bool`) -
///   A boolean flag indicating whether the node represents a file (`true`) or a directory (`false`).
///
/// # Derives
///
/// * `Default` -
///   Provides a default implementation for the `Node` struct, initializing all fields with their default values.
///
/// * `Clone` -
///   Enables the `Node` struct to be cloned, creating an exact copy of the instance.
#[derive(Default, Clone)]
pub struct Node {
    pub id: usize,
    pub name: String,
    pub content: Vec<String>,
    pub colored_lines: Vec<Vec<(String, Color)>>,
    pub is_file: bool,
}

/// Represents the state of a pane in an application.
///
/// The `PaneState` struct is marked with `Copy` and `Clone` traits,
/// allowing instances to be easily duplicated without ownership concerns.
///
/// # Fields
/// - `workspace` (`u8`): Indicates the current workspace index the pane belongs to.
/// - `view` (`u8`): Represents the current view index within the pane.
/// - `cursor` (`u16`): Stores the position of the cursor in the pane.
#[derive(Copy, Clone)]
pub struct PaneState {
    pub workspace: u8,
    pub view: u8,
    pub cursor: u16,
}

/// An enumeration representing the different operational modes of an application.
///
/// The `Mode` enum is defined with the `#[derive(PartialEq)]` attribute, allowing for
/// comparison between its variants. This can be useful in scenarios where the current
/// mode needs to be compared or checked within the application logic.
///
/// # Variants
///
/// - `Normal`:
///     The default operational mode, typically representing the standard state of the application.
///
/// - `Menu`:
///     Represents a state where the application is in a menu interaction mode.
///
/// - `Finder`:
///     Represents a state where the application is in a finder or search browsing mode.
///
/// - `Editor`:
///     Represents a state where the application is in editing mode, allowing users to modify content or files.
///
/// - `Search`:
///     Represents a state where the application is focused on searching functionality.
///
/// # Examples
///
/// ```rust
/// use qwx::editor::Mode;
///
/// let current_mode = Mode::Normal;
///
/// if current_mode == Mode::Menu {
///     println!("The application is in menu mode.");
/// } else {
///     println!("The application is not in menu mode.");
/// }
/// ```
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Mode {
    Normal,
    Menu,
    Finder,
    Editor,
    Search,
    WebSearch,
    Player,
}
///
/// A trait that provides functionality for managing a cursor in the context of a writable output.
/// This includes operations to scroll, show, and hide the cursor.
///
/// # Type Parameters
/// - `W`: A type that implements the `Write` trait, representing the writable output,
///         such as a terminal or other output stream.
///
/// # Required Methods
///
/// ## `scroll`
/// Scrolls the cursor in the specified direction.
///
/// ### Parameters
/// - `w`: A mutable reference to an instance of type `W` that represents the writable output.
/// - `direction`: A `QwxScrollDirection` value specifying the direction in which to scroll the cursor.
///
/// ### Returns
/// - `Result<(), Error>`: Returns `Ok(())` if the operation is successful, or an `Err` variant
/// containing an `Error` if the operation fails.
///
/// ## `show`
/// Makes the cursor visible.
///
/// ### Parameters
/// - `w`: A mutable reference to an instance of type `W` that represents the writable output.
///
/// ### Returns
/// - `Result<(), Error>`: Returns `Ok(())` if the operation is successful, or an `Err` variant
/// containing an `Error` if the operation fails.
///
/// ## `hide`
/// Hides the cursor from being visible.
///
/// ### Parameters
/// - `w`: A mutable reference to an instance of type `W` that represents the writable output.
///
/// ### Returns
/// - `Result<(), Error>`: Returns `Ok(())` if the operation is successful, or an `Err` variant
/// containing an `Error` if the operation fails.
///
pub trait QwxCursor<W: Write> {
    /// Scrolls the content in the specified direction and updates the given writable output.
    ///
    /// # Parameters
    /// - `w`: A mutable reference to a writable object of type `W`.
    ///         This is used to output or update any relevant changes caused by the scroll action.
    /// - `direction`: The direction of the scroll, specified using the `QwxScrollDirection` enum.
    ///                This determines the scrolling behavior (e.g., up, down, left, right).
    ///
    /// # Returns
    /// - `Ok(())`: If the scroll operation was performed successfully.
    /// - `Err(Error)`: If an error occurred during the scroll operation.
    ///
    /// # Errors
    /// This function may return an error in cases such as:
    /// - An invalid operation on the writable object.
    /// - An unsupported scroll direction or other internal state failures depending on implementation.
    ///
    /// This example demonstrates how to invoke the `scroll` method to scroll downward with proper handling.
    ///
    /// # Notes
    /// This method mutably borrows the writable object `w` and the struct on which it is called.
    fn scroll(&mut self, w: &mut W, direction: QwxScrollDirection) -> Result<(), Error>;
    /// Displays content using the provided writer.
    ///
    /// This function renders or outputs content to the given writable instance `w`.
    /// The specific content and behavior depend on the implementation of this function
    /// in the corresponding type.
    ///
    /// # Parameters
    /// - `w`: A mutable reference to the writer where the content will be displayed.
    ///        The writer must implement the `Write` trait.
    ///
    /// # Returns
    /// - `Ok(())`: If the content was successfully displayed.
    /// - `Err(Error)`: If an error occurred during the operation.
    ///
    /// # Errors
    /// This function may return an error if there are issues written to the provided writer.
    fn show(&mut self, w: &mut W) -> Result<(), Error>;
    /// Hides the current object, performing operations related to the provided writer.
    ///
    /// # Parameters
    /// - `w`: A mutable reference to a writer of type `W` that the object interacts with during the hide operation.
    ///
    /// # Returns
    /// - `Result<(), Error>`: Returns `Ok(())` if the operation is successful, or an `Error` if something goes wrong during the process.
    ///
    /// # Errors
    /// This function returns an `Error` if it encounters any issues while attempting to hide the object.
    fn hide(&mut self, w: &mut W) -> Result<(), Error>;
}
/// A trait that defines the rendering behavior for a QwxRenderer.
///
/// This trait is implemented for structures that handle rendering operations, intended to write
/// output using a generic `Write` trait implementation. It provides methods for clearing rendering targets,
/// drawing text, and updating the rendered output.
///
/// # Type Parameters
/// - `W`: A type that implements the `Write` trait, used for writing rendered output.
///
pub trait QwxRenderer<W: Write> {
    /// Clears the rendering target with the specified mode.
    fn clear(&mut self, w: &mut W, mode: ClearType) -> Result<(), Error>;
    /// Clears the entire rendering target.
    fn clear_screen(&mut self, w: &mut W) -> Result<(), Error>;

    /// Draws text at the specified position with the given color.
    fn draw_text(
        &mut self,
        w: &mut W,
        x: u16,
        y: u16,
        text: &str,
        color: Color,
    ) -> Result<(), Error>;
    /// Flushes the rendering buffer to the output.
    fn flush(&mut self, w: &mut W) -> Result<(), Error>;
}
/// The `QwxBuffer` trait provides an abstraction for a text buffer. It defines
/// operations for manipulating and querying the contents of the buffer, such as
/// inserting and deleting characters, retrieving specific lines of text, and
/// getting the total number of lines in the buffer.
pub trait QwxBuffer {
    /// Insert a character at the specified position in the buffer.
    fn insert_char(&mut self, line: usize, col: usize, c: char);
    /// Delete a character at the specified position in the buffer.
    fn delete_char(&mut self, line: usize, col: usize);
    /// Get the line at the specified position in the buffer.
    fn get_line(&self, line: usize) -> Option<&str>;
    /// Get the length of the buffer.
    fn len_lines(&self) -> usize;
}

impl<W: Write> QwxCursor<W> for Qwx {
    fn scroll(&mut self, w: &mut W, direction: QwxScrollDirection) -> Result<(), Error> {
        match direction {
            QwxScrollDirection::Vertical(x) => execute!(w, MoveRight(x)),
            QwxScrollDirection::Horizontal(x) => execute!(w, MoveDown(x)),
            QwxScrollDirection::Left(x) => execute!(w, MoveLeft(x)),
            QwxScrollDirection::Right(x) => execute!(w, MoveRight(x)),
            QwxScrollDirection::Down(x) => execute!(w, MoveDown(x)),
            QwxScrollDirection::Up(x) => execute!(w, MoveUp(x)),
        }
    }
    /// Displays the associated content or performs an action tied to the `Show` command.
    ///
    /// This method writes a `Show` command to the provided mutable writer `w`
    /// and executes it. It integrates with the `execute!` macro to handle
    /// the operation and returns a `Result` indicating success or failure.
    ///
    /// # Parameters
    /// - `w`: A mutable reference to a writer that implements the `Write`
    ///   trait. This is the output target where the `Show` command will be executed.
    ///
    /// # Returns
    /// - `Ok(())`: If the `Show` command was successfully executed.
    /// - `Err(Error)`: If an error occurred while attempting to execute the command.
    ///
    /// # Errors
    /// This function returns an error in cases where the `execute!` macro fails
    /// to perform the intended operation, such as problems with the writer or internal execution.
    fn show(&mut self, w: &mut W) -> Result<(), Error> {
        execute!(w, Show)
    }
    /// Hides the cursor in the given writable stream.
    ///
    /// This function sends a `Hide` command to the specified writable stream `w`,
    /// which hides the cursor in terminal-based applications. The modification is
    /// applied to the `w` writable stream passed as a mutable reference.
    ///
    /// # Arguments
    ///
    /// * `w` - A mutable reference to a writable stream implementing the Write trait.
    ///          This will receive the `Hide` command to hide the cursor.
    ///
    /// # Returns
    ///
    /// Returns a `Result` which is `Ok(())` if the command is executed successfully.
    /// If an error occurs during the execution of the `Hide` command, it will return
    /// an `Err` variant containing the associated `Error`.
    ///
    /// # Errors
    ///
    /// An error may occur if the `execute!` macro fails to send the `Hide` command to
    /// the writable stream, for instance, due to IO or stream-related issues.
    fn hide(&mut self, w: &mut W) -> Result<(), Error> {
        execute!(w, Hide)
    }
}

impl<W: Write> QwxRenderer<W> for Qwx {
    fn clear(&mut self, w: &mut W, mode: ClearType) -> Result<(), Error> {
        queue!(w, Clear(mode))
    }

    fn clear_screen(&mut self, w: &mut W) -> Result<(), Error> {
        queue!(w, Clear(ClearType::All))
    }

    fn draw_text(
        &mut self,
        w: &mut W,
        x: u16,
        y: u16,
        text: &str,
        color: Color,
    ) -> Result<(), Error> {
        queue!(
            w,
            SetBackgroundColor(Color::Black),
            SetForegroundColor(color),
            MoveTo(x, y),
            Print(text),
            SetBackgroundColor(Color::Reset),
            SetForegroundColor(Color::Reset)
        )
    }

    fn flush(&mut self, w: &mut W) -> Result<(), Error> {
        w.flush()
    }
}

impl Qwx {
    fn sync_node_content(&mut self) {
        let active_idx = self.focus as usize;

        if let Some(view) = self.views.get(active_idx) {
            let node_id = view.active_node_id;

            if let Some(node) = self.nodes.iter_mut().find(|n| n.id == node_id) {
                let mut new_content = Vec::new();

                for line in self.editor.rope.lines() {
                    let clean_line = line
                        .to_string()
                        .trim_end_matches(&['\n', '\r'][..])
                        .to_string();
                    new_content.push(clean_line);
                }

                node.content = new_content;
                let mut new_colored = Vec::new();
                let spans = self.editor.get_colored_spans();

                if !spans.is_empty() {
                    new_colored.push(vec![]);
                    for (text, color) in spans {
                        let mut is_first = true;
                        for part in text.split('\n') {
                            if !is_first {
                                new_colored.push(vec![]);
                            }
                            if !part.is_empty() {
                                new_colored
                                    .last_mut()
                                    .unwrap()
                                    .push((part.to_string(), color));
                            }
                            is_first = false;
                        }
                    }
                } else {
                    for line in &node.content {
                        new_colored.push(vec![(line.clone(), Color::White)]);
                    }
                }
                node.colored_lines = new_colored;
            }
        }
        self.follow();
    }
    fn active_pane_mut(&mut self) -> &mut PaneState {
        &mut self.panes[self.focus as usize]
    }
    pub fn follow(&mut self) {
        let cursor_line = self.editor.cursor_line;
        let mid_y = self.height / 2;
        let bottom_y = self.height.saturating_sub(1);
        let p_height = match self.focus {
            PaneFocus::TopLeft | PaneFocus::TopRight => mid_y.saturating_sub(1),
            PaneFocus::BottomLeft | PaneFocus::BottomRight => (bottom_y - mid_y).saturating_sub(1),
        } as usize;

        let pane = self.active_pane_mut();
        let scroll_y = pane.cursor as usize;

        let margin = 3.min(p_height / 3);

        if cursor_line < scroll_y + margin {
            pane.cursor = cursor_line.saturating_sub(margin) as u16;
        } else if cursor_line + margin >= scroll_y + p_height {
            pane.cursor = (cursor_line + margin + 1).saturating_sub(p_height) as u16;
        }
    }
    fn handle_normal(&mut self) {
        match read().expect("failed to get terminal input") {
            Event::Key(key) => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Char('j')) => {
                    if self.editor.cursor_line + 1 < self.editor.rope.len_lines() {
                        self.editor.cursor_line += 1;

                        let max_col = self
                            .editor
                            .rope
                            .line(self.editor.cursor_line)
                            .len_chars()
                            .saturating_sub(1);

                        // Sécurité pour ne pas déborder sur une ligne vide
                        self.editor.cursor_col = self.editor.cursor_col.min(max_col);
                    }
                    // C'est follow qui se charge de faire défiler le panneau si nécessaire !
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('k')) => {
                    if self.editor.cursor_line > 0 {
                        self.editor.cursor_line -= 1;

                        let max_col = self
                            .editor
                            .rope
                            .line(self.editor.cursor_line)
                            .len_chars()
                            .saturating_sub(1);

                        self.editor.cursor_col = self.editor.cursor_col.min(max_col);
                    }
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('h')) => {
                    if self.editor.cursor_col > 0 {
                        self.editor.cursor_col -= 1;
                    } else if self.editor.cursor_line > 0 {
                        self.editor.cursor_line -= 1;
                        self.editor.cursor_col = self
                            .editor
                            .rope
                            .line(self.editor.cursor_line)
                            .len_chars()
                            .saturating_sub(1);
                    }
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('l')) => {
                    let max_col = self
                        .editor
                        .rope
                        .line(self.editor.cursor_line)
                        .len_chars()
                        .saturating_sub(1);
                    if self.editor.cursor_col < max_col {
                        self.editor.cursor_col += 1;
                    } else if self.editor.cursor_line + 1 < self.editor.rope.len_lines() {
                        self.editor.cursor_line += 1;
                        self.editor.cursor_col = 0;
                    }
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::PageDown) => {
                    let active_idx = self.focus as usize;
                    let node_len = if let Some(view) = self.views.get(active_idx) {
                        self.nodes
                            .get(view.active_node_id)
                            .map(|n| n.content.len())
                            .unwrap_or(0)
                    } else {
                        0
                    };
                    let step = 15;
                    let active_pane = self.active_pane_mut();
                    if (active_pane.cursor as usize) + step < node_len {
                        active_pane.cursor += step as u16;
                    } else {
                        active_pane.cursor = node_len.saturating_sub(1) as u16;
                    }
                    self.editor.cursor_line = self.active_pane_mut().cursor as usize;
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::PageUp) => {
                    let step = 15;
                    let active_pane = self.active_pane_mut();
                    active_pane.cursor = active_pane.cursor.saturating_sub(step);
                    self.editor.cursor_line = self.active_pane_mut().cursor as usize;
                    self.follow();
                }

                (KeyModifiers::NONE, KeyCode::Char('x')) => {
                    self.editor.select_line();
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('d')) => {
                    if self.editor.selection.is_some() {
                        self.editor.delete_selection();
                        self.sync_node_content();
                        self.follow();
                    }
                }
                (KeyModifiers::NONE, KeyCode::Char('u')) => {
                    self.editor.undo();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('U'))
                | (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
                    self.editor.redo();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('y')) => {
                    self.editor.yank();
                }
                (KeyModifiers::NONE, KeyCode::Char('p')) => {
                    self.editor.paste();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('n')) => {
                    self.search_next();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Char('N')) => {
                    self.search_prev();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                    let _ = self.editor.save();
                }
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    if self.editor.selection.is_some() {
                        self.editor.selection = None;
                    }
                }
                (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                    self.focus = match self.focus {
                        PaneFocus::TopLeft => PaneFocus::TopRight,
                        PaneFocus::BottomLeft => PaneFocus::BottomRight,
                        _ => self.focus,
                    };
                    self.load_active_pane_file();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
                    self.focus = match self.focus {
                        PaneFocus::TopRight => PaneFocus::TopLeft,
                        PaneFocus::BottomRight => PaneFocus::BottomLeft,
                        _ => self.focus,
                    };
                    self.load_active_pane_file();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('j')) => {
                    self.focus = match self.focus {
                        PaneFocus::TopLeft => PaneFocus::BottomLeft,
                        PaneFocus::TopRight => PaneFocus::BottomRight,
                        _ => self.focus,
                    };
                    self.load_active_pane_file();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                    self.focus = match self.focus {
                        PaneFocus::BottomLeft => PaneFocus::TopLeft,
                        PaneFocus::BottomRight => PaneFocus::TopRight,
                        _ => self.focus,
                    };
                    self.load_active_pane_file();
                }

                // --- TRANSITIONS DE MODES ---
                (KeyModifiers::NONE, KeyCode::Char('o')) => {
                    let max_col = self
                        .editor
                        .rope
                        .line(self.editor.cursor_line)
                        .len_chars()
                        .saturating_sub(1);
                    self.editor.cursor_col = max_col;
                    self.editor.insert_char('\n');
                    self.sync_node_content();
                    self.follow();
                    self.mode = Mode::Editor;
                }
                (KeyModifiers::NONE, KeyCode::Char('e')) => {
                    self.mode = Mode::Editor;
                }
                (KeyModifiers::ALT, KeyCode::Char('f')) => {
                    self.mode = Mode::Finder;
                }
                (KeyModifiers::ALT, KeyCode::Char('d')) => {
                    self.mode = Mode::Menu;
                    self.menu_input.clear();
                }
                (KeyModifiers::ALT, KeyCode::Char('/')) => {
                    self.mode = Mode::Search;
                    self.search_input.clear();
                }
                (KeyModifiers::ALT, KeyCode::Char('s'))
                | (KeyModifiers::ALT, KeyCode::Char('w'))
                | (KeyModifiers::NONE, KeyCode::Char('s')) => {
                    self.mode = Mode::WebSearch;
                }
                (KeyModifiers::ALT, KeyCode::Char('p'))
                | (KeyModifiers::ALT, KeyCode::Char('m')) => {
                    self.mode = Mode::Player;
                    self.player.refresh_playback_state();
                    let _ = execute!(stdout(), Clear(ClearType::All));
                }
                (KeyModifiers::NONE, KeyCode::Char('q')) => {
                    self.running = false;
                }
                // --- Rotation Horaire (Ctrl + r) ---
                (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                    let old_panes = self.panes;
                    self.panes[1] = old_panes[0];
                    self.panes[3] = old_panes[1];
                    self.panes[2] = old_panes[3];
                    self.panes[0] = old_panes[2];

                    if self.views.len() < 4 {
                        self.views.resize_with(4, || View { active_node_id: 0 });
                    }

                    let v0 = self.views[0].active_node_id;
                    let v1 = self.views[1].active_node_id;
                    let v2 = self.views[2].active_node_id;
                    let v3 = self.views[3].active_node_id;

                    self.views[1].active_node_id = v0;
                    self.views[3].active_node_id = v1;
                    self.views[2].active_node_id = v3;
                    self.views[0].active_node_id = v2;

                    self.load_active_pane_file();
                }
                (KeyModifiers::ALT, KeyCode::Char('r')) => {
                    let old_panes = self.panes;
                    self.panes[2] = old_panes[0];
                    self.panes[3] = old_panes[2];
                    self.panes[1] = old_panes[3];
                    self.panes[0] = old_panes[1];

                    if self.views.len() < 4 {
                        self.views.resize_with(4, || View { active_node_id: 0 });
                    }

                    let v0 = self.views[0].active_node_id;
                    let v1 = self.views[1].active_node_id;
                    let v2 = self.views[2].active_node_id;
                    let v3 = self.views[3].active_node_id;

                    self.views[2].active_node_id = v0;
                    self.views[3].active_node_id = v2;
                    self.views[1].active_node_id = v3;
                    self.views[0].active_node_id = v1;

                    self.load_active_pane_file();
                }
                _ => {}
            },
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
                let _ = execute!(stdout(), Clear(ClearType::All));
            }
            _ => {}
        }
    }

    fn handle_menu(&mut self) {
        match read().expect("msg") {
            Event::Key(key) => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    self.mode = Mode::Normal;
                    self.menu_input.clear();
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    if let Some(cmd) = self.menu_input.strip_prefix('!') {
                        let cmd_clean = cmd.trim();
                        if let Some(dir_name) = cmd_clean.strip_prefix("mkdir ") {
                            let target_path = self.current_dir.join(dir_name.trim());
                            let _ = create_dir_all(&target_path);
                        } else if let Some(file_name) = cmd_clean.strip_prefix("touch ") {
                            let target_path = self.current_dir.join(file_name.trim());
                            let _ = File::create(&target_path);
                        } else {
                            let _ = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(cmd_clean)
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .status();

                            for node in self.nodes.iter_mut() {
                                if node.is_file {
                                    let full_path = self.current_dir.join(&node.name);
                                    if let Ok(fresh_node) = qwx_load_node(node.id, &full_path) {
                                        node.content = fresh_node.content;
                                        node.colored_lines = fresh_node.colored_lines;
                                    }
                                }
                            }
                        }
                    } else if let Some(url) = self.menu_input.strip_prefix(":web ") {
                        let url_clean = url.trim().to_string();
                        self.mode = Mode::WebSearch;
                        self.search_hub.web_browser.open_url(&url_clean, self.width);
                        self.search_hub.show_web_reader = true;
                        self.menu_input.clear();
                        let _ = execute!(stdout(), Clear(ClearType::All));
                        return;
                    } else if let Some(query) = self.menu_input.strip_prefix(":search ") {
                        let q_clean = query.trim().to_string();
                        self.mode = Mode::WebSearch;
                        self.search_hub.query = q_clean;
                        self.search_hub.show_web_reader = false;
                        self.search_hub.perform_search(&self.current_dir);
                        self.menu_input.clear();
                        let _ = execute!(stdout(), Clear(ClearType::All));
                        return;
                    } else if self.menu_input.trim() == ":player"
                        || self.menu_input.trim() == ":music"
                        || self.menu_input.trim() == ":spotify"
                    {
                        self.mode = Mode::Player;
                        self.player.refresh_playback_state();
                        self.menu_input.clear();
                        let _ = execute!(stdout(), Clear(ClearType::All));
                        return;
                    }
                    self.mode = Mode::Normal;
                    self.menu_input.clear();
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    self.menu_input.pop();
                }
                (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                    self.menu_input.push(c);
                }
                _ => {}
            },
            Event::Paste(x) => {
                self.menu_input.push_str(x.as_str());
            }
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
                let _ = execute!(stdout(), Clear(ClearType::All));
            }
            _ => {}
        }
    }

    fn handle_editor(&mut self) {
        match read().expect("msg") {
            Event::Key(key) => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    if self.editor.selection.is_some() {
                        self.editor.selection = None;
                    } else {
                        self.mode = Mode::Normal;
                    }
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    self.editor.insert_char('\n');
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Delete) => {
                    self.editor.delete();
                    self.sync_node_content();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('s')) => {
                    if self.editor.save().is_err() {
                        eprintln!("Failed to save");
                    }
                }
                (KeyModifiers::CONTROL, KeyCode::Char('z')) => {
                    self.editor.undo();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('y')) => {
                    self.editor.redo();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('v')) => {
                    self.editor.paste();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                    let current_line = self.editor.cursor_line;
                    if current_line < self.editor.rope.len_lines() {
                        let chars_to_delete = self.editor.rope.line(current_line).len_chars();
                        self.editor.cursor_col = 0;
                        for _ in 0..chars_to_delete {
                            self.editor.delete();
                        }
                        if current_line >= self.editor.rope.len_lines() && current_line > 0 {
                            self.editor.cursor_line -= 1;
                        }
                        self.sync_node_content();
                    }
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    self.editor.backspace();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::ALT, KeyCode::Char('x')) => {
                    self.editor.select_line();
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::ALT, KeyCode::Char('d')) => {
                    if self.editor.selection.is_some() {
                        self.editor.delete_selection();
                    }
                    self.sync_node_content();
                    self.follow();
                }
                (KeyModifiers::NONE, KeyCode::Tab) => {
                    for _ in 0..4 {
                        self.editor.insert_char(' ');
                    }
                    self.sync_node_content();
                }
                (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                    self.editor.insert_char(c);
                    self.sync_node_content();
                    self.follow();
                }
                _ => {}
            },
            Event::Paste(x) => {
                self.editor.record_undo();
                for ch in x.chars() {
                    self.editor.insert_char_raw(ch);
                }
                self.editor.update_syntax_tree();
                self.sync_node_content();
                self.follow();
            }
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
                let _ = execute!(stdout(), Clear(ClearType::All));
            }
            _ => {}
        }
    }

    fn handle_finder(&mut self) {
        match read().expect("msg") {
            Event::Key(key) => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    self.mode = Mode::Normal;
                    self.finder_research.clear();
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    self.finder_research.pop();
                    self.finder.filter(self.finder_research.clone());
                }
                (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                    self.finder_research.push(c);
                    self.finder.filter(self.finder_research.clone());
                }
                (KeyModifiers::CONTROL, KeyCode::Char('j')) => {
                    self.finder.next_file();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
                    self.finder.prev_file();
                }
                (KeyModifiers::ALT, KeyCode::Char('j')) => {
                    self.finder.next_dir();
                }
                (KeyModifiers::ALT, KeyCode::Char('k')) => {
                    self.finder.prev_dir();
                }
                (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
                    if let Some(parent) = self.current_dir.parent() {
                        self.current_dir = parent.into();
                        self.finder = Finder::new(&self.current_dir, self.finder_layout.clone());
                        self.finder_research.clear();
                    }
                }
                (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                    let dirs = self.finder.get_directories();
                    if !dirs.is_empty() && self.finder.selected_dir < dirs.len() {
                        let dirname = &dirs[self.finder.selected_dir];
                        let new_path = self.current_dir.join(dirname);
                        self.current_dir = new_path.clone().into();
                        self.finder = Finder::new(&new_path, self.finder_layout.clone());
                        self.finder_research.clear();
                    }
                }
                (m, KeyCode::Char('j'))
                    if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) =>
                {
                    self.finder.next_sub_dir();
                }
                (m, KeyCode::Char('k'))
                    if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) =>
                {
                    self.finder.prev_sub_dir();
                }
                (m, KeyCode::Char('l'))
                    if m.contains(KeyModifiers::CONTROL) && m.contains(KeyModifiers::SHIFT) =>
                {
                    let sub_dirs = self.finder.get_sub_directories();
                    if !sub_dirs.is_empty() && self.finder.selected_sub_dir < sub_dirs.len() {
                        let dirname = &sub_dirs[self.finder.selected_sub_dir];
                        let new_path = self.current_dir.join(dirname);
                        self.current_dir = new_path.clone().into();
                        self.finder = Finder::new(&new_path, self.finder_layout.clone());
                        self.finder_research.clear();
                    }
                }
                (KeyModifiers::ALT, KeyCode::Right) => {
                    self.previous_finder_layout();
                }
                (KeyModifiers::ALT, KeyCode::Left) => {
                    self.next_finder_layout();
                }
                (KeyModifiers::NONE, KeyCode::F(5)) => {
                    self.finder = Finder::new(Path::new("."), self.finder_layout.clone());
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    let files = self.finder.get_files();
                    if !files.is_empty() && self.finder.selected_file < files.len() {
                        let filename = &files[self.finder.selected_file];
                        let full_path = self.current_dir.join(filename);

                        let node_id = if let Some(existing_node) =
                            self.nodes.iter().find(|n| n.name == *filename)
                        {
                            existing_node.id
                        } else {
                            let new_id = self.nodes.len();
                            if let Ok(node) = qwx_load_node(new_id, &full_path) {
                                self.nodes.push(node);
                                new_id
                            } else {
                                self.finder_research.clear();
                                return;
                            }
                        };
                        let active_idx = self.focus as usize;

                        if self.views.len() <= active_idx {
                            self.views
                                .resize_with(active_idx + 1, || View { active_node_id: 0 });
                        }

                        if let Some(view) = self.views.get_mut(active_idx) {
                            view.active_node_id = node_id;
                        }

                        self.panes[active_idx].cursor = 0;
                        if let Some(path_str) = full_path.to_str()
                            && let Ok(editor) = Ji::open(path_str)
                        {
                            self.editor = editor;
                        }
                    }
                    self.mode = Mode::Normal;
                    self.finder_research.clear();
                }
                _ => {}
            },
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
                self.finder.resize(cols, rows);
                let _ = execute!(stdout(), Clear(ClearType::All));
            }
            _ => {}
        }
    }

    fn handle_search(&mut self) {
        match read().expect("msg") {
            Event::Key(key) => match (key.modifiers, key.code) {
                (KeyModifiers::NONE, KeyCode::Esc) => {
                    self.mode = Mode::Normal;
                    self.search_input.clear();
                }
                (KeyModifiers::NONE, KeyCode::Enter) => {
                    if !self.search_input.is_empty() {
                        self.last_search_query = Some(self.search_input.clone());
                        self.search_next();
                        self.sync_node_content();
                        self.follow();
                    }
                    self.mode = Mode::Normal;
                    self.search_input.clear();
                }
                (KeyModifiers::NONE, KeyCode::Backspace) => {
                    self.search_input.pop();
                }
                (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                    self.search_input.push(c);
                }
                _ => {}
            },
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
                let _ = execute!(stdout(), Clear(ClearType::All));
            }
            _ => {}
        }
    }
    fn handle_web_search(&mut self) {
        match read().expect("msg") {
            Event::Key(key) => {
                // 1. If Web Reader is currently active inside the Search Hub
                if self.search_hub.show_web_reader {
                    // 1.1 Web Reader active input prompts (URL bar, jump to link ID, in-page search)
                    if self.search_hub.web_browser.url_prompt_active {
                        match (key.modifiers, key.code) {
                            (KeyModifiers::NONE, KeyCode::Esc) => {
                                self.search_hub.web_browser.url_prompt_active = false;
                                self.search_hub.web_browser.url_input.clear();
                                let _ = execute!(stdout(), Clear(ClearType::All));
                            }
                            (KeyModifiers::NONE, KeyCode::Enter) => {
                                let input = self.search_hub.web_browser.url_input.clone();
                                self.search_hub.web_browser.url_prompt_active = false;
                                self.search_hub.web_browser.url_input.clear();
                                if !input.trim().is_empty() {
                                    self.search_hub.web_browser.open_url(&input, self.width);
                                }
                                let _ = execute!(stdout(), Clear(ClearType::All));
                            }
                            (KeyModifiers::NONE, KeyCode::Backspace) => {
                                self.search_hub.web_browser.url_input.pop();
                            }
                            (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                                self.search_hub.web_browser.url_input.push(c);
                            }
                            _ => {}
                        }
                        return;
                    }

                    if self.search_hub.web_browser.link_prompt_active {
                        match (key.modifiers, key.code) {
                            (KeyModifiers::NONE, KeyCode::Esc) => {
                                self.search_hub.web_browser.link_prompt_active = false;
                                self.search_hub.web_browser.link_input.clear();
                                let _ = execute!(stdout(), Clear(ClearType::All));
                            }
                            (KeyModifiers::NONE, KeyCode::Enter) => {
                                let input = self.search_hub.web_browser.link_input.clone();
                                self.search_hub.web_browser.link_prompt_active = false;
                                self.search_hub.web_browser.link_input.clear();
                                if let Ok(id) = input.trim().parse::<usize>() {
                                    self.search_hub
                                        .web_browser
                                        .follow_link_by_id(id, self.width);
                                }
                                let _ = execute!(stdout(), Clear(ClearType::All));
                            }
                            (KeyModifiers::NONE, KeyCode::Backspace) => {
                                self.search_hub.web_browser.link_input.pop();
                            }
                            (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                                if c.is_ascii_digit() {
                                    self.search_hub.web_browser.link_input.push(c);
                                }
                            }
                            _ => {}
                        }
                        return;
                    }

                    if self.search_hub.web_browser.search_mode {
                        match (key.modifiers, key.code) {
                            (KeyModifiers::NONE, KeyCode::Esc) => {
                                self.search_hub.web_browser.search_mode = false;
                                self.search_hub.web_browser.search_query.clear();
                                let _ = execute!(stdout(), Clear(ClearType::All));
                            }
                            (KeyModifiers::NONE, KeyCode::Enter) => {
                                let query = self.search_hub.web_browser.search_query.clone();
                                self.search_hub.web_browser.search_mode = false;
                                if !query.trim().is_empty() {
                                    self.search_hub.web_browser.search_page(&query);
                                }
                                let _ = execute!(stdout(), Clear(ClearType::All));
                            }
                            (KeyModifiers::NONE, KeyCode::Backspace) => {
                                self.search_hub.web_browser.search_query.pop();
                            }
                            (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                                self.search_hub.web_browser.search_query.push(c);
                            }
                            _ => {}
                        }
                        return;
                    }

                    // 1.2 Web Reader Navigation Keys
                    match (key.modifiers, key.code) {
                        (KeyModifiers::NONE, KeyCode::Esc) => {
                            self.search_hub.close_web_reader();
                            let _ = execute!(stdout(), Clear(ClearType::All));
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('o'))
                        | (KeyModifiers::CONTROL, KeyCode::Char('g')) => {
                            self.search_hub.web_browser.url_prompt_active = true;
                            self.search_hub.web_browser.url_input.clear();
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('f')) => {
                            self.search_hub.web_browser.link_prompt_active = true;
                            self.search_hub.web_browser.link_input.clear();
                        }
                        (KeyModifiers::NONE, KeyCode::Char('/')) => {
                            self.search_hub.web_browser.search_mode = true;
                            self.search_hub.web_browser.search_query.clear();
                        }
                        (KeyModifiers::ALT, KeyCode::Char('n')) => {
                            self.search_hub.web_browser.next_search_match();
                        }
                        (KeyModifiers::SHIFT, KeyCode::Char('N'))
                        | (KeyModifiers::ALT, KeyCode::Char('N')) => {
                            self.search_hub.web_browser.prev_search_match();
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
                            self.search_hub.web_browser.go_back(self.width);
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('F')) => {
                            self.search_hub.web_browser.go_forward(self.width);
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('r')) => {
                            self.search_hub.web_browser.reload(self.width);
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('m')) => {
                            self.search_hub.web_browser.toggle_view_mode();
                        }
                        (KeyModifiers::CONTROL, KeyCode::Char('B')) => {
                            self.search_hub.web_browser.bookmark_current_page();
                        }
                        (KeyModifiers::ALT, KeyCode::Char('T')) => {
                            self.search_hub.web_browser.next_link();
                        }
                        (KeyModifiers::ALT, KeyCode::Char('t')) => {
                            self.search_hub.web_browser.prev_link();
                        }
                        (KeyModifiers::SHIFT, KeyCode::BackTab)
                        | (KeyModifiers::SHIFT, KeyCode::Tab) => {
                            self.search_hub.web_browser.prev_link();
                        }
                        (KeyModifiers::NONE, KeyCode::Enter) => {
                            self.search_hub.web_browser.follow_selected_link(self.width);
                        }
                        (KeyModifiers::NONE, KeyCode::Up)
                        | (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                            self.search_hub.web_browser.scroll_up(1);
                        }
                        (KeyModifiers::NONE, KeyCode::Down)
                        | (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                            self.search_hub.web_browser.scroll_down(1);
                        }
                        (KeyModifiers::NONE, KeyCode::PageUp) => {
                            self.search_hub.web_browser.scroll_up(10);
                        }
                        (KeyModifiers::NONE, KeyCode::PageDown) => {
                            self.search_hub.web_browser.scroll_down(10);
                        }
                        _ => {}
                    }
                    return;
                }

                // 2. Active Git / PR Modal Prompts in Search Hub
                if let Some(ref mut prompt) = self.search_hub.prompt {
                    match (key.modifiers, key.code) {
                        (KeyModifiers::NONE, KeyCode::Esc) => {
                            self.search_hub.prompt = None;
                            self.search_hub.status_message = Some("Action cancelled.".to_string());
                            let _ = execute!(stdout(), Clear(ClearType::All));
                        }
                        (KeyModifiers::NONE, KeyCode::Enter) => match prompt {
                            crate::search::ActionPrompt::CloneRepo {
                                repo_url,
                                dest_input,
                            } => {
                                let dest_path = self.current_dir.join(dest_input.trim());
                                let url = repo_url.clone();
                                let target_str = dest_path.display().to_string();

                                self.search_hub.prompt =
                                    Some(crate::search::ActionPrompt::CloneInProgress {
                                        repo_url: url.clone(),
                                        dest_path: target_str.clone(),
                                        progress_pct: 10,
                                        status_text: "Connecting and negotiating Git objects..."
                                            .to_string(),
                                    });

                                let res = crate::search::clone_repository_with_progress(
                                    &url,
                                    &dest_path,
                                    None::<fn(crate::search::CloneProgress)>,
                                );
                                match res {
                                    Ok(msg) => {
                                        self.search_hub.status_message = Some(msg);
                                        self.search_hub.prompt = None;
                                    }
                                    Err(err) => {
                                        self.search_hub.status_message =
                                            Some(format!("Error: {}", err));
                                        self.search_hub.prompt = None;
                                    }
                                }
                            }
                            crate::search::ActionPrompt::CloneInProgress { .. } => {}
                            crate::search::ActionPrompt::CreateBranch { branch_input } => {
                                let branch_name = branch_input.trim().to_string();
                                let res = crate::search::create_git_branch(
                                    &self.current_dir,
                                    &branch_name,
                                );
                                match res {
                                    Ok(msg) => {
                                        self.search_hub.status_message = Some(msg);
                                        self.search_hub.prompt = None;
                                    }
                                    Err(err) => {
                                        self.search_hub.status_message =
                                            Some(format!("Error: {}", err));
                                        self.search_hub.prompt = None;
                                    }
                                }
                            }
                            crate::search::ActionPrompt::CheckoutBranch { branch_input } => {
                                let branch_name = branch_input.trim().to_string();
                                let res = crate::search::checkout_git_branch(
                                    &self.current_dir,
                                    &branch_name,
                                );
                                match res {
                                    Ok(msg) => {
                                        self.search_hub.status_message = Some(msg);
                                        self.search_hub.prompt = None;
                                    }
                                    Err(err) => {
                                        self.search_hub.status_message =
                                            Some(format!("Error: {}", err));
                                        self.search_hub.prompt = None;
                                    }
                                }
                            }
                            crate::search::ActionPrompt::ExportReport { path_input } => {
                                let dest_path = self.current_dir.join(path_input.trim());
                                let res = crate::search::export_report_to_file(
                                    &dest_path,
                                    &self.search_hub.results,
                                );
                                match res {
                                    Ok(msg) => {
                                        self.search_hub.status_message = Some(msg);
                                        self.search_hub.prompt = None;
                                    }
                                    Err(err) => {
                                        self.search_hub.status_message =
                                            Some(format!("Error: {}", err));
                                        self.search_hub.prompt = None;
                                    }
                                }
                            }
                            crate::search::ActionPrompt::CreatePullRequest {
                                repo_input,
                                title_input,
                                body_input,
                                head_input,
                                base_input,
                                token_input,
                                step,
                            } => {
                                if *step < 5 {
                                    *step += 1;
                                    self.search_hub.status_message = Some(format!(
                                        "Creating Pull Request - Step {}/6",
                                        *step + 1
                                    ));
                                } else {
                                    let repo = repo_input.clone();
                                    let title = title_input.clone();
                                    let body = body_input.clone();
                                    let head = head_input.clone();
                                    let base = base_input.clone();
                                    let token = if token_input.trim().is_empty() {
                                        None
                                    } else {
                                        Some(token_input.as_str())
                                    };
                                    self.search_hub.status_message =
                                        Some("Submitting Pull Request...".to_string());
                                    let res = crate::search::create_github_pull_request(
                                        &repo, &title, &body, &head, &base, token,
                                    );
                                    match res {
                                        Ok(msg) => {
                                            self.search_hub.status_message = Some(msg);
                                            self.search_hub.prompt = None;
                                        }
                                        Err(err) => {
                                            self.search_hub.status_message =
                                                Some(format!("PR Error: {}", err));
                                            self.search_hub.prompt = None;
                                        }
                                    }
                                }
                            }
                        },
                        (KeyModifiers::NONE, KeyCode::Backspace) => match prompt {
                            crate::search::ActionPrompt::CloneRepo { dest_input, .. } => {
                                dest_input.pop();
                            }
                            crate::search::ActionPrompt::CreateBranch { branch_input } => {
                                branch_input.pop();
                            }
                            crate::search::ActionPrompt::CheckoutBranch { branch_input } => {
                                branch_input.pop();
                            }
                            crate::search::ActionPrompt::ExportReport { path_input } => {
                                path_input.pop();
                            }
                            crate::search::ActionPrompt::CreatePullRequest {
                                repo_input,
                                title_input,
                                body_input,
                                head_input,
                                base_input,
                                token_input,
                                step,
                            } => match step {
                                0 => {
                                    repo_input.pop();
                                }
                                1 => {
                                    title_input.pop();
                                }
                                2 => {
                                    body_input.pop();
                                }
                                3 => {
                                    head_input.pop();
                                }
                                4 => {
                                    base_input.pop();
                                }
                                5 => {
                                    token_input.pop();
                                }
                                _ => {}
                            },
                            crate::search::ActionPrompt::CloneInProgress { .. } => {}
                        },
                        (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                            match prompt {
                                crate::search::ActionPrompt::CloneRepo { dest_input, .. } => {
                                    dest_input.push(c);
                                }
                                crate::search::ActionPrompt::CreateBranch { branch_input } => {
                                    branch_input.push(c);
                                }
                                crate::search::ActionPrompt::CheckoutBranch { branch_input } => {
                                    branch_input.push(c);
                                }
                                crate::search::ActionPrompt::ExportReport { path_input } => {
                                    path_input.push(c);
                                }
                                crate::search::ActionPrompt::CreatePullRequest {
                                    repo_input,
                                    title_input,
                                    body_input,
                                    head_input,
                                    base_input,
                                    token_input,
                                    step,
                                } => match step {
                                    0 => {
                                        repo_input.push(c);
                                    }
                                    1 => {
                                        title_input.push(c);
                                    }
                                    2 => {
                                        body_input.push(c);
                                    }
                                    3 => {
                                        head_input.push(c);
                                    }
                                    4 => {
                                        base_input.push(c);
                                    }
                                    5 => {
                                        token_input.push(c);
                                    }
                                    _ => {}
                                },
                                crate::search::ActionPrompt::CloneInProgress { .. } => {}
                            }
                        }
                        _ => {}
                    }
                    return;
                }

                // 3. SearchHub Navigation & Input (Results Grid View)
                match (key.modifiers, key.code) {
                    (KeyModifiers::NONE, KeyCode::Esc) => {
                        self.mode = Mode::Normal;
                        let _ = execute!(stdout(), Clear(ClearType::All));
                    }
                    (KeyModifiers::NONE, KeyCode::Tab) => {
                        self.search_hub.next_provider();
                    }
                    (KeyModifiers::SHIFT, KeyCode::BackTab)
                    | (KeyModifiers::SHIFT, KeyCode::Tab) => {
                        self.search_hub.prev_provider();
                    }
                    (KeyModifiers::NONE, KeyCode::Up) => {
                        self.search_hub.prev_result();
                    }
                    (KeyModifiers::NONE, KeyCode::Down) => {
                        self.search_hub.next_result();
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
                        self.search_hub.prev_result();
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
                        self.search_hub.next_result();
                    }
                    (KeyModifiers::NONE, KeyCode::PageUp) => {
                        self.search_hub.scroll_preview_up();
                    }
                    (KeyModifiers::NONE, KeyCode::PageDown) => {
                        self.search_hub.scroll_preview_down();
                    }
                    (KeyModifiers::ALT, KeyCode::Char('w'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('w')) => {
                        self.search_hub.open_selected_in_web_reader(self.width);
                        let _ = execute!(stdout(), Clear(ClearType::All));
                    }
                    (KeyModifiers::ALT, KeyCode::Char('v'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('v')) => {
                        self.search_hub.view_results_as_web_page(self.width);
                        let _ = execute!(stdout(), Clear(ClearType::All));
                    }
                    (KeyModifiers::ALT, KeyCode::Char('o'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('o')) => {
                        self.search_hub.open_selected_in_browser();
                    }
                    (KeyModifiers::ALT, KeyCode::Char('e'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('e')) => {
                        self.search_hub.start_export_report();
                    }
                    (KeyModifiers::ALT, KeyCode::Char('s')) => {
                        self.search_hub.start_checkout_branch();
                    }
                    (KeyModifiers::ALT, KeyCode::Char('c'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        self.search_hub.start_clone_selected();
                    }
                    (KeyModifiers::ALT, KeyCode::Char('b'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
                        self.search_hub.start_create_branch();
                    }
                    (KeyModifiers::ALT, KeyCode::Char('p')) => {
                        self.search_hub.start_create_pull_request();
                    }
                    (KeyModifiers::ALT, KeyCode::Char('a'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('a')) => {
                        self.search_hub
                            .set_provider(crate::search::SearchProvider::LocalAudit);
                        self.search_hub.perform_search(&self.current_dir);
                    }
                    (KeyModifiers::ALT, KeyCode::Char('1'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('1')) => {
                        self.search_hub
                            .set_provider(crate::search::SearchProvider::All);
                    }
                    (KeyModifiers::ALT, KeyCode::Char('2'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('2')) => {
                        self.search_hub
                            .set_provider(crate::search::SearchProvider::Crates);
                    }
                    (KeyModifiers::ALT, KeyCode::Char('3'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('3')) => {
                        self.search_hub
                            .set_provider(crate::search::SearchProvider::GitHub);
                    }
                    (KeyModifiers::ALT, KeyCode::Char('4'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('4')) => {
                        self.search_hub
                            .set_provider(crate::search::SearchProvider::GitLab);
                    }
                    (KeyModifiers::ALT, KeyCode::Char('5'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('5')) => {
                        self.search_hub
                            .set_provider(crate::search::SearchProvider::Wikipedia);
                    }
                    (KeyModifiers::ALT, KeyCode::Char('6'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('6')) => {
                        self.search_hub
                            .set_provider(crate::search::SearchProvider::Cve);
                    }
                    (KeyModifiers::ALT, KeyCode::Char('7'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('7')) => {
                        self.search_hub
                            .set_provider(crate::search::SearchProvider::HackerNews);
                    }
                    (KeyModifiers::ALT, KeyCode::Char('8'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('8')) => {
                        self.search_hub
                            .set_provider(crate::search::SearchProvider::LocalAudit);
                    }
                    (KeyModifiers::ALT, KeyCode::Char('9'))
                    | (KeyModifiers::CONTROL, KeyCode::Char('9')) => {
                        self.search_hub
                            .set_provider(crate::search::SearchProvider::Web);
                    }
                    (KeyModifiers::NONE, KeyCode::Enter) => {
                        self.search_hub.perform_search(&self.current_dir);
                    }
                    (KeyModifiers::NONE, KeyCode::Backspace) => {
                        self.search_hub.query.pop();
                    }
                    (m, KeyCode::Char(c)) if m.is_empty() || m == KeyModifiers::SHIFT => {
                        self.search_hub.query.push(c);
                    }
                    _ => {}
                }
            }
            Event::Resize(cols, rows) => {
                self.width = cols;
                self.height = rows;
                let _ = execute!(stdout(), Clear(ClearType::All));
            }
            _ => {}
        }
    }

    pub fn handle_events(&mut self) {
        match self.mode {
            Mode::Normal => self.handle_normal(),
            Mode::Finder => self.handle_finder(),
            Mode::Menu => self.handle_menu(),
            Mode::Editor => self.handle_editor(),
            Mode::Search => self.handle_search(),
            Mode::WebSearch => self.handle_web_search(),
            Mode::Player => self.handle_player(),
        }
    }

    fn handle_player(&mut self) {
        if poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
            match read().expect("failed to get terminal input") {
                Event::Key(key) => {
                    if !self.player.handle_key(key.code, key.modifiers) {
                        self.mode = Mode::Normal;
                        let _ = execute!(stdout(), Clear(ClearType::All));
                    }
                }
                Event::Resize(cols, rows) => {
                    self.width = cols;
                    self.height = rows;
                    let _ = execute!(stdout(), Clear(ClearType::All));
                }
                _ => {}
            }
        }
    }
    /// Creates a new instance of the editor with the specified path and open mode.
    pub fn is_finder_open(&self) -> bool {
        self.mode == Mode::Finder
    }
    pub fn run(&mut self) -> Result<(), Error> {
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;
        execute!(stdout, EnterAlternateScreen)?;
        self.clear_screen(&mut stdout)?;
        while self.running {
            self.draw(&mut stdout)?;
            self.handle_events();
        }
        execute!(stdout, LeaveAlternateScreen, Show)?;
        terminal::disable_raw_mode()?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    /// Displays a preview of a given node within a specified area.
    pub fn preview(
        &self,
        node: &Node,
        start_x: u16,
        start_y: u16,
        p_width: u16,
        p_height: u16,
        scroll_y: usize,
        selection: Option<(usize, usize)>,
    ) -> Result<(), Error> {
        let mut w = stdout();
        let mut drawn_lines = 0;

        for (line_idx, line_spans) in node
            .colored_lines
            .iter()
            .skip(scroll_y)
            .take(p_height as usize)
            .enumerate()
        {
            queue!(w, MoveTo(start_x, start_y + line_idx as u16))?;

            let current_absolute_line = scroll_y + line_idx;
            let is_selected = match selection {
                Some((start, end)) => {
                    current_absolute_line >= start && current_absolute_line <= end
                }
                None => false,
            };

            if is_selected {
                queue!(
                    w,
                    SetBackgroundColor(Color::Rgb {
                        r: 55,
                        g: 65,
                        b: 85
                    })
                )?;
            }

            let mut current_width = 0;
            for (text, color) in line_spans {
                let clean_text = text.replace('\t', "    ").replace('\r', "");
                let text_width = clean_text.width();
                let remaining_width = p_width.saturating_sub(current_width) as usize;

                if remaining_width == 0 {
                    break;
                }

                let display_text = if text_width > remaining_width {
                    let mut acc_width = 0;
                    let mut truncated = String::new();
                    for c in clean_text.chars() {
                        let c_width = c.width().unwrap_or(0);
                        if acc_width + c_width > remaining_width {
                            break;
                        }
                        truncated.push(c);
                        acc_width += c_width;
                    }
                    truncated
                } else {
                    clean_text
                };

                queue!(w, SetForegroundColor(*color), Print(&display_text))?;
                current_width += display_text.width() as u16;
            }

            if current_width < p_width {
                let padding = " ".repeat((p_width - current_width) as usize);
                queue!(w, Print(padding))?;
            }

            queue!(w, ResetColor)?;
            drawn_lines += 1;
        }

        for empty_y in drawn_lines..(p_height as usize) {
            let padding = " ".repeat(p_width as usize);
            queue!(
                w,
                MoveTo(start_x, start_y + empty_y as u16),
                ResetColor,
                Print(padding)
            )?;
        }
        Ok(())
    }
    /// Creates a new instance of the editor with the specified path and open mode.
    pub fn new(path: &Path, open_mode: Mode) -> Result<Self, Error> {
        let (width, height) = size()?;
        let (dir_path, target_file) = if path.is_file() {
            (path.parent().unwrap_or_else(|| Path::new(".")), Some(path))
        } else {
            (path, None)
        };

        let mut nodes: Vec<Node> = Vec::new();
        let mut views: Vec<View> = Vec::new();
        let mut target_node_id = 0;

        let file_list = list_files(dir_path);
        for (i, filename) in file_list.iter().enumerate() {
            let fpath = PathBuf::from(filename);
            if let Ok(node) = qwx_load_node(i, fpath.as_path()) {
                if let Some(target) = target_file {
                    if let (Ok(p1), Ok(p2)) = (fpath.canonicalize(), target.canonicalize()) {
                        if p1 == p2 {
                            target_node_id = i;
                        }
                    } else if fpath == target {
                        target_node_id = i;
                    }
                }
                nodes.push(node);
                views.push(View { active_node_id: i });
            }
        }

        let mut editor = Ji::default();
        if let Some(target) = target_file {
            if let Ok(ed) = Ji::open(target) {
                editor = ed;
            }
        } else if let Some(first_file) = file_list.first() {
            if let Ok(ed) = Ji::open(Path::new(first_file)) {
                editor = ed;
            }
        }

        let mut panes = [
            INIT_PANE_STATE,
            INIT_PANE_STATE,
            INIT_PANE_STATE,
            INIT_PANE_STATE,
        ];
        if target_file.is_some() && target_node_id < views.len() {
            panes[0].view = target_node_id as u8;
        }

        Ok(Self {
            width,
            height,
            running: true,
            focus: PaneFocus::TopLeft,
            panes,
            mode: open_mode,
            menu_input: String::new(),
            nodes: nodes.clone(),
            views,
            finder_layout: FinderLayout::Grid,
            finder: Finder::new(dir_path, FinderLayout::Grid),
            finder_research: String::new(),
            current_dir: dir_path.into(),
            editor,
            search_input: String::new(),
            last_search_query: None,
            search_hub: crate::search::SearchHub::new(),
            player: MusicPlayer::default(),
        })
    }

    pub fn search_next(&mut self) {
        if let Some(ref pattern) = self.last_search_query {
            if let Ok(re) = regex::Regex::new(pattern) {
                let text = self.editor.rope.to_string();
                let cursor_char =
                    self.editor.rope.line_to_char(self.editor.cursor_line) + self.editor.cursor_col;
                let cursor_byte = self.editor.rope.char_to_byte(cursor_char);

                let search_start = (cursor_byte + 1).min(text.len());
                let found = re.find_at(&text, search_start).or_else(|| re.find(&text));

                if let Some(m) = found {
                    let match_char = self.editor.rope.byte_to_char(m.start());
                    self.editor.cursor_line = self.editor.rope.char_to_line(match_char);
                    self.editor.cursor_col =
                        match_char - self.editor.rope.line_to_char(self.editor.cursor_line);
                    self.active_pane_mut().cursor = self.editor.cursor_line as u16;
                }
            }
        }
    }

    pub fn search_prev(&mut self) {
        if let Some(ref pattern) = self.last_search_query {
            if let Ok(re) = regex::Regex::new(pattern) {
                let text = self.editor.rope.to_string();
                let cursor_char =
                    self.editor.rope.line_to_char(self.editor.cursor_line) + self.editor.cursor_col;
                let cursor_byte = self.editor.rope.char_to_byte(cursor_char);

                let mut prev_match = None;
                let mut last_match = None;
                for m in re.find_iter(&text) {
                    if m.start() < cursor_byte {
                        prev_match = Some(m);
                    }
                    last_match = Some(m);
                }

                let target = prev_match.or(last_match);
                if let Some(m) = target {
                    let match_char = self.editor.rope.byte_to_char(m.start());
                    self.editor.cursor_line = self.editor.rope.char_to_line(match_char);
                    self.editor.cursor_col =
                        match_char - self.editor.rope.line_to_char(self.editor.cursor_line);
                    self.active_pane_mut().cursor = self.editor.cursor_line as u16;
                }
            }
        }
    }
    /// Creates a new instance of the editor with the specified path and open mode.
    pub fn draw_finder<W: Write>(&mut self, w: &mut W) -> io::Result<()> {
        let max_width = 180.min(self.width);
        let left_x = (self.width.saturating_sub(max_width)) / 2;
        self.finder.clone().show(
            w,
            &mut self.finder,
            &mut self.finder_research.as_mut(),
            left_x,
            0,
            max_width,
            self.height,
        )
    }
}
/// Creates a configuration object for syntax highlighting based on the provided parameters.
///
/// # Parameters
/// - `scope`: A string slice representing the scope of the language for syntax highlighting.
/// - `lang`: The `Language` enum or object that represents the programming language.
/// - `query`: A static string reference containing the query definitions for text parsing.
/// - `theme_keys`: A slice of static string references representing the highlight theme keys to enable.
///
/// # Returns
/// - `Option<LangConfig>`:
///   - Returns `Some(LangConfig)` containing the configured `LangConfig` if the operation succeeds.
///   - Returns `None` if the function fails to create a `HighlightConfiguration`.
///
/// # Errors
/// This function may return `None` if the `HighlightConfiguration::new` fails to initialize with the provided arguments.
pub fn create_config(
    scope: &str,
    lang: Language,
    query: &'static str,
    theme_keys: &[&'static str],
) -> Option<LangConfig> {
    let mut ts_config = HighlightConfiguration::new(lang, scope, query, "", "").ok()?;
    ts_config.configure(theme_keys);
    Some(LangConfig {
        ts_config,
        query_string: query,
    })
}
/// Detects the programming language based on the given file extension.
///
/// This function maps a file extension (e.g., "cpp", "html") to a specific language
/// configuration using the appropriate tree-sitter language grammars and highlight queries.
///
/// # Arguments
///
/// * `extension` - A string slice representing the file extension (e.g., "cpp", "md", "html").
/// * `theme_keys` - A reference to an array slice of theme-specific keys for syntax highlighting.
///
/// # Returns
///
/// Returns an `Option<LangConfig>`:
/// - Some(`LangConfig`) if the file extension matches a known language.
/// - `None` if the file extension is not mapped to a known language.
///
/// # Supported Extensions and Corresponding Languages
///
/// Below are some of the file extensions supported and their mapped languages:
///
/// | Extensions            | Language       | Tree-Sitter Grammar                       | Highlight Query              |
/// |-----------------------|----------------|------------------------------------------|------------------------------|
/// | `ada`, `adb`          | Ada            | `tree_sitter_ada::LANGUAGE`              | None                         |
/// | `ps1`, `psm1`, `psd1` | PowerShell     | `tree_sitter_powershell::LANGUAGE`       | `tree_sitter_powershell::HIGHLIGHTS_QUERY` |
/// | `c`, `h`              | C              | `tree_sitter_c::LANGUAGE`                | `tree_sitter_c::HIGHLIGHT_QUERY`          |
/// | `cpp`, `cc`, `hpp`    | C++            | `tree_sitter_cpp::LANGUAGE`              | `tree_sitter_cpp::HIGHLIGHT_QUERY`        |
/// | `html`, `htm`         | HTML           | `tree_sitter_html::LANGUAGE`             | `tree_sitter_html::HIGHLIGHTS_QUERY`      |
///
/// Check the function implementation for an exhaustive list of supported extensions and languages.
///
/// # Notes
///
/// - Some languages, such as `d`, `hcl`, and `glsl`, do not have associated highlight queries.
/// - File extensions for the same language may vary (e.g., `cpp`, `cc`, `hpp` for C++).
/// - This function relies on the `create_config` helper for building language configurations.
fn detect_language(extension: &str, theme_keys: &[&'static str]) -> Option<LangConfig> {
    match extension {
        "ada" | "adb" => create_config(
            "ada",
            Language::from(tree_sitter_ada::LANGUAGE),
            "",
            theme_keys,
        ),
        "ps1" | "psm1" | "psd1" => create_config(
            "powershell",
            Language::from(tree_sitter_powershell::LANGUAGE),
            tree_sitter_powershell::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "scss" | "sass" => create_config(
            "scss",
            Language::from(tree_sitter_sas::LANGUAGE),
            tree_sitter_sas::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "Kconfig" => create_config(
            "Kconfig",
            Language::from(tree_sitter_kconfig::LANGUAGE),
            tree_sitter_kconfig::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "vhdl" => create_config(
            "vhdl",
            Language::from(tree_sitter_vhdl::LANGUAGE),
            tree_sitter_vhdl::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "jinja2" => create_config(
            "jinja2",
            Language::from(tree_sitter_jinja2::LANGUAGE),
            tree_sitter_jinja2::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "nginx" => create_config(
            "nginx",
            Language::from(tree_sitter_nginx::LANGUAGE),
            "",
            theme_keys,
        ),
        "zsh" => create_config(
            "zsh",
            Language::from(tree_sitter_zsh::LANGUAGE),
            tree_sitter_zsh::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "md" => create_config(
            "md",
            Language::from(tree_sitter_md::LANGUAGE),
            "",
            theme_keys,
        ),
        "agda" => create_config(
            "agda",
            Language::from(tree_sitter_agda::LANGUAGE),
            tree_sitter_agda::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "asm" | "s" => create_config(
            "asm",
            Language::from(tree_sitter_asm::LANGUAGE),
            tree_sitter_asm::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "sh" | "bash" => create_config(
            "bash",
            Language::from(tree_sitter_bash::LANGUAGE),
            tree_sitter_bash::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "bat" | "cmd" => create_config(
            "batch",
            Language::from(tree_sitter_batch::LANGUAGE),
            tree_sitter_batch::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "c" | "h" => create_config(
            "c",
            Language::from(tree_sitter_c::LANGUAGE),
            tree_sitter_c::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "cs" => create_config(
            "c_sharp",
            Language::from(tree_sitter_c_sharp::LANGUAGE),
            tree_sitter_c_sharp::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "cmake" => create_config(
            "cmake",
            Language::from(tree_sitter_cmake::LANGUAGE),
            tree_sitter_cmake::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "cpp" | "cc" | "cxx" | "hpp" => create_config(
            "cpp",
            Language::from(tree_sitter_cpp::LANGUAGE),
            tree_sitter_cpp::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "css" => create_config(
            "css",
            Language::from(tree_sitter_css::LANGUAGE),
            tree_sitter_css::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "d" => create_config("d", Language::from(tree_sitter_d::LANGUAGE), "", theme_keys),
        "dart" => create_config(
            "dart",
            Language::from(tree_sitter_dart::LANGUAGE),
            tree_sitter_dart::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "diff" | "patch" => create_config(
            "diff",
            Language::from(tree_sitter_diff::LANGUAGE),
            "",
            theme_keys,
        ),
        "ex" | "exs" => create_config(
            "elixir",
            Language::from(tree_sitter_elixir::LANGUAGE),
            tree_sitter_elixir::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "elm" => create_config(
            "elm",
            Language::from(tree_sitter_elm::LANGUAGE),
            tree_sitter_elm::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "erl" | "hrl" => create_config(
            "erlang",
            Language::from(tree_sitter_erlang::LANGUAGE),
            tree_sitter_erlang::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "fish" => create_config(
            "fish",
            tree_sitter_fish::language(),
            tree_sitter_fish::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "fs" | "fsi" | "fsx" => create_config(
            "fsharp",
            Language::from(tree_sitter_fsharp::LANGUAGE_FSHARP),
            tree_sitter_fsharp::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "glsl" | "vert" | "frag" => create_config(
            "glsl",
            Language::from(tree_sitter_glsl::LANGUAGE_GLSL),
            "",
            theme_keys,
        ),
        "go" => create_config(
            "go",
            Language::from(tree_sitter_go::LANGUAGE),
            tree_sitter_go::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "gql" | "graphql" => create_config(
            "graphql",
            Language::from(tree_sitter_graphql::LANGUAGE),
            "",
            theme_keys,
        ),
        "hs" => create_config(
            "haskell",
            Language::from(tree_sitter_haskell::LANGUAGE),
            tree_sitter_haskell::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "hcl" | "tf" => create_config(
            "hcl",
            Language::from(tree_sitter_hcl::LANGUAGE),
            "",
            theme_keys,
        ),
        "hlsl" => create_config(
            "hlsl",
            Language::from(tree_sitter_hlsl::LANGUAGE_HLSL),
            "",
            theme_keys,
        ),
        "html" | "htm" => create_config(
            "html",
            Language::from(tree_sitter_html::LANGUAGE),
            tree_sitter_html::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "ini" => create_config(
            "ini",
            Language::from(tree_sitter_ini::LANGUAGE),
            tree_sitter_ini::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "java" => create_config(
            "java",
            Language::from(tree_sitter_java::LANGUAGE),
            tree_sitter_java::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "js" | "mjs" | "cjs" => create_config(
            "javascript",
            Language::from(tree_sitter_javascript::LANGUAGE),
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "json" => create_config(
            "json",
            Language::from(tree_sitter_json::LANGUAGE),
            tree_sitter_json::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "lua" => create_config(
            "lua",
            Language::from(tree_sitter_lua::LANGUAGE),
            tree_sitter_lua::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "make" | "makefile" | "Makefile" => create_config(
            "make",
            Language::from(tree_sitter_make::LANGUAGE),
            tree_sitter_make::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "nix" => create_config(
            "nix",
            Language::from(tree_sitter_nix::LANGUAGE),
            tree_sitter_nix::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "m" => create_config(
            "objc",
            Language::from(tree_sitter_objc::LANGUAGE),
            tree_sitter_objc::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "odin" => create_config(
            "odin",
            Language::from(tree_sitter_odin::LANGUAGE),
            tree_sitter_odin::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "pl" | "pm" => create_config(
            "perl",
            Language::from(tree_sitter_perl::LANGUAGE),
            "",
            theme_keys,
        ),
        "php" => create_config(
            "php",
            Language::from(tree_sitter_php::LANGUAGE_PHP),
            tree_sitter_php::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "py" | "pyw" => create_config(
            "python",
            Language::from(tree_sitter_python::LANGUAGE),
            tree_sitter_python::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "r" => create_config(
            "r",
            Language::from(tree_sitter_r::LANGUAGE),
            tree_sitter_r::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "rb" => create_config(
            "ruby",
            Language::from(tree_sitter_ruby::LANGUAGE),
            tree_sitter_ruby::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "rs" => create_config(
            "rust",
            Language::from(tree_sitter_rust::LANGUAGE),
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "scala" | "sc" => create_config(
            "scala",
            Language::from(tree_sitter_scala::LANGUAGE),
            tree_sitter_scala::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "swift" => create_config(
            "swift",
            Language::from(tree_sitter_swift::LANGUAGE),
            tree_sitter_swift::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "ts" | "mts" | "cts" => create_config(
            "typescript",
            Language::from(tree_sitter_typescript::LANGUAGE_TYPESCRIPT),
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "xml" | "xsd" => create_config(
            "xml",
            Language::from(tree_sitter_xml::LANGUAGE_XML),
            tree_sitter_xml::XML_HIGHLIGHT_QUERY,
            theme_keys,
        ),
        "yaml" | "yml" => create_config(
            "yaml",
            Language::from(tree_sitter_yaml::LANGUAGE),
            tree_sitter_yaml::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        "zig" => create_config(
            "zig",
            Language::from(tree_sitter_zig::LANGUAGE),
            tree_sitter_zig::HIGHLIGHTS_QUERY,
            theme_keys,
        ),
        _ => None, // Unknown extension
    }
}
/// Represents the configuration of highlighting for a specific language.
pub struct LangConfig {
    pub ts_config: HighlightConfiguration,
    pub query_string: &'static str,
}
/// Snapshot representing an editor state for undo/redo.
#[derive(Clone, Debug)]
pub struct EditSnapshot {
    pub rope: Rope,
    pub cursor_line: usize,
    pub cursor_col: usize,
}

/// Represents the core state and metadata for a text editing structure.
///
/// The `Ji` struct holds various components necessary for managing text,
/// syntax parsing, and other related features in a text editor.
#[derive(Default)]
pub struct Ji {
    pub rope: Rope,
    pub file_path: Option<PathBuf>,
    pub query: Option<Query>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub parser: Parser,
    pub syntax_tree: Option<Tree>,
    pub lang_config: Option<LangConfig>,
    pub selection: Option<(usize, usize)>,
    pub undo_stack: Vec<EditSnapshot>,
    pub redo_stack: Vec<EditSnapshot>,
    pub clipboard: Option<String>,
    pub is_dirty: bool,
}

impl Ji {
    /// Records the current state onto the undo stack and clears the redo stack.
    pub fn record_undo(&mut self) {
        if self.undo_stack.len() >= 100 {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(EditSnapshot {
            rope: self.rope.clone(),
            cursor_line: self.cursor_line,
            cursor_col: self.cursor_col,
        });
        self.redo_stack.clear();
        self.is_dirty = true;
    }

    /// Reverts the text editor to the previous state on the undo stack.
    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            if self.redo_stack.len() >= 100 {
                self.redo_stack.remove(0);
            }
            self.redo_stack.push(EditSnapshot {
                rope: self.rope.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            });
            self.rope = snapshot.rope;
            self.cursor_line = snapshot.cursor_line;
            self.cursor_col = snapshot.cursor_col;
            self.selection = None;
            self.update_syntax_tree();
        }
    }

    /// Restores the next state from the redo stack.
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            if self.undo_stack.len() >= 100 {
                self.undo_stack.remove(0);
            }
            self.undo_stack.push(EditSnapshot {
                rope: self.rope.clone(),
                cursor_line: self.cursor_line,
                cursor_col: self.cursor_col,
            });
            self.rope = snapshot.rope;
            self.cursor_line = snapshot.cursor_line;
            self.cursor_col = snapshot.cursor_col;
            self.selection = None;
            self.update_syntax_tree();
        }
    }

    /// Copies selected lines or current line into the internal clipboard.
    pub fn yank(&mut self) -> Option<String> {
        let text = if let Some((start, end)) = self.selection {
            let mut s = String::new();
            for line_idx in start..=end.min(self.rope.len_lines().saturating_sub(1)) {
                if line_idx < self.rope.len_lines() {
                    s.push_str(&self.rope.line(line_idx).to_string());
                }
            }
            s
        } else if self.cursor_line < self.rope.len_lines() {
            self.rope.line(self.cursor_line).to_string()
        } else {
            return None;
        };
        self.clipboard = Some(text.clone());
        Some(text)
    }

    /// Pastes text from the internal clipboard at the current cursor position.
    pub fn paste(&mut self) {
        if let Some(text) = self.clipboard.clone() {
            self.record_undo();
            for ch in text.chars() {
                self.insert_char_raw(ch);
            }
            self.update_syntax_tree();
        }
    }

    /// Selects the entire line where the cursor is currently positioned.
    pub fn select_line(&mut self) {
        if let Some((start, end)) = self.selection {
            if end + 1 < self.rope.len_lines() {
                self.selection = Some((start, end + 1));
                self.cursor_line = end + 1; // Le curseur descend visuellement
            }
        } else {
            self.selection = Some((self.cursor_line, self.cursor_line));
        }
    }

    /// Deletes the currently selected text in the editor.
    pub fn delete_selection(&mut self) {
        if let Some((start, end)) = self.selection {
            self.record_undo();
            let start_char = self.rope.line_to_char(start);
            let end_char = if end + 1 < self.rope.len_lines() {
                self.rope.line_to_char(end + 1)
            } else {
                self.rope.len_chars()
            };

            let start_byte = self.rope.char_to_byte(start_char);
            let end_byte = self.rope.char_to_byte(end_char);

            // Mise à jour chirurgicale de Tree-sitter pour ne pas casser la coloration
            if let Some(ref mut tree) = self.syntax_tree {
                let edit = InputEdit {
                    start_byte,
                    old_end_byte: end_byte,
                    new_end_byte: start_byte,
                    start_position: Point::new(start, 0),
                    old_end_position: Point::new(end + 1, 0),
                    new_end_position: Point::new(start, 0),
                };
                tree.edit(&edit);
            }

            self.rope.remove(start_char..end_char);
            self.cursor_line = start;
            self.cursor_col = 0;
            self.selection = None;
            self.update_syntax_tree();
        }
    }

    /// Deletes the character immediately under the cursor (Delete key).
    pub fn delete(&mut self) {
        // 1. Calculate the absolute index of the cursor
        let cursor_char_idx = self.rope.line_to_char(self.cursor_line) + self.cursor_col;

        // If we're at the very end of the file, there's nothing to delete
        if cursor_char_idx >= self.rope.len_chars() {
            return;
        }

        self.record_undo();

        // 2. Identify the target character (exactly under the cursor)
        let target_char = self.rope.char(cursor_char_idx);
        let char_len_bytes = target_char.len_utf8();
        let byte_idx = self.rope.char_to_byte(cursor_char_idx);

        // 3. Determine the graphical positions for Tree-sitter
        let start_point = Point::new(self.cursor_line, self.cursor_col);

        let mut old_end_point = start_point;
        if target_char == '\n' {
            old_end_point.row += 1;
            old_end_point.column = 0;
        } else {
            old_end_point.column += char_len_bytes;
        }

        // 4. Notify the syntax tree of the deletion
        if let Some(ref mut tree) = self.syntax_tree {
            let edit = InputEdit {
                start_byte: byte_idx,
                old_end_byte: byte_idx + char_len_bytes,
                new_end_byte: byte_idx,
                start_position: start_point,
                old_end_position: old_end_point,
                new_end_position: start_point, // The cursor doesn't move
            };
            tree.edit(&edit);
        }

        // 5. Delete the character in the Rope structure
        self.rope.remove(cursor_char_idx..(cursor_char_idx + 1));

        self.update_syntax_tree();
    }

    /// Saves the current state of the data to a file at the specified file path.
    pub fn save(&mut self) -> io::Result<()> {
        if let Some(ref path) = self.file_path {
            let file = File::create(path)?;
            let writer = std::io::BufWriter::new(file);
            self.rope.write_to(writer)?;
            self.is_dirty = false;
        }
        Ok(())
    }

    /// Opens a file at the specified path and initializes a custom editor instance.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref)?;
        let rope = Rope::from_reader(BufReader::new(file))?;

        let theme_keys = vec![
            "keyword",
            "keyword.function",
            "keyword.return",
            "keyword.operator",
            "function",
            "function.macro",
            "function.method",
            "method",
            "string",
            "string_literal",
            "character",
            "number",
            "integer",
            "float",
            "boolean",
            "comment",
            "line_comment",
            "block_comment",
            "type",
            "primitive_type",
            "type.builtin",
            "operator",
            "punctuation.bracket",
            "punctuation.delimiter",
            "variable",
            "variable.parameter",
            "variable.builtin",
            "property",
            "attribute",
            "label",
            "constant",
            "constant.builtin",
            "constant.character.escape",
            "namespace",
            "keyword.directive",
            "punctuation.special",
        ];

        let filename = path_ref.file_name().expect("");
        let ext = path_ref.extension().unwrap_or(filename);

        let mut ji = Self {
            rope,
            file_path: Some(path_ref.to_path_buf()),
            cursor_line: 0,
            cursor_col: 0,
            parser: Parser::new(),
            syntax_tree: None,
            lang_config: None,
            query: None,
            selection: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            clipboard: None,
            is_dirty: false,
        };

        if let Some(config) = detect_language(ext.to_str().expect(""), &theme_keys) {
            ji.query = Query::new(&config.ts_config.language, config.query_string).ok();
            let _ = ji.parser.set_language(&config.ts_config.language);
            ji.lang_config = Some(config);

            ji.update_syntax_tree();
        }
        Ok(ji)
    }

    /// Raw insertion of a character without pushing to undo stack.
    pub fn insert_char_raw(&mut self, ch: char) {
        let char_idx = self.rope.line_to_char(self.cursor_line) + self.cursor_col;
        let byte_idx = self.rope.char_to_byte(char_idx);

        let start_point = Point::new(self.cursor_line, self.cursor_col);

        let mut new_end_point = start_point;
        if ch == '\n' {
            new_end_point.row += 1;
            new_end_point.column = 0;
        } else {
            new_end_point.column += ch.len_utf8();
        }

        if let Some(ref mut tree) = self.syntax_tree {
            let edit = InputEdit {
                start_byte: byte_idx,
                old_end_byte: byte_idx,
                new_end_byte: byte_idx + ch.len_utf8(),
                start_position: start_point,
                old_end_position: start_point,
                new_end_position: new_end_point,
            };
            tree.edit(&edit);
        }

        self.rope.insert_char(char_idx, ch);
        if ch == '\n' {
            self.cursor_line += 1;
            self.cursor_col = 0;
        } else {
            self.cursor_col += 1;
        }
    }

    /// Insert a character at the current cursor position (line, col)
    pub fn insert_char(&mut self, ch: char) {
        self.record_undo();
        self.insert_char_raw(ch);
        self.update_syntax_tree();
    }

    /// Deletes the character positioned just before the current cursor position in the text editor.
    pub fn backspace(&mut self) {
        if self.cursor_line == 0 && self.cursor_col == 0 {
            return;
        }

        self.record_undo();

        let cursor_char_idx = self.rope.line_to_char(self.cursor_line) + self.cursor_col;
        let target_char_idx = cursor_char_idx - 1;

        let target_char = self.rope.char(target_char_idx);
        let char_len_bytes = target_char.len_utf8();
        let byte_idx = self.rope.char_to_byte(target_char_idx);

        let old_end_point = Point::new(self.cursor_line, self.cursor_col);
        let mut start_point = old_end_point;

        if target_char == '\n' {
            start_point.row -= 1;
            start_point.column = self.rope.line(start_point.row).len_chars() - 1;
        } else {
            start_point.column -= 1;
        }
        if let Some(ref mut tree) = self.syntax_tree {
            let edit = InputEdit {
                start_byte: byte_idx,
                old_end_byte: byte_idx + char_len_bytes,
                new_end_byte: byte_idx,
                start_position: start_point,
                old_end_position: old_end_point,
                new_end_position: start_point,
            };
            tree.edit(&edit);
        }

        self.rope.remove(target_char_idx..cursor_char_idx);

        self.cursor_line = start_point.row;
        self.cursor_col = start_point.column;

        self.update_syntax_tree();
    }
    /// Updates the current syntax tree for the editor or document using the internal Ji
    /// parser (based on tree-sitter). This method regenerates the syntax tree
    /// by parsing the underlying text content while supporting incremental parsing.
    ///
    /// # Behavior
    /// - If no language configuration is set (`lang_config` is `None`),
    ///   the method exits early without making any changes.
    /// - The parsing process is performed incrementally by reusing the existing
    ///   syntax tree if one exists (`self.syntax_tree`). This mechanism optimizes
    ///   performance by recalculating only changes in the text content.
    ///
    /// # Implementation Details
    /// - Uses `Ropey` to efficiently provide chunks of text as byte slices to the parser
    ///   when required. The parser requests these slices dynamically during parsing.
    ///
    /// # Parsing Process
    /// - The parser requests byte offsets from the rope and computes offsets using chunks.
    /// - If the requested byte offset exceeds the text length, an empty slice is returned
    ///   to indicate the end of the text.
    ///
    /// # Post-Parsing
    /// - After parsing, the updated syntax tree is saved in `self.syntax_tree`.
    ///
    /// # Errors
    /// - This method handles parsing gracefully and exits early if any preconditions are
    ///   not met (e.g., missing language configuration).
    pub fn update_syntax_tree(&mut self) {
        if self.lang_config.is_none() {
            return;
        }
        let rope = &self.rope;

        let tree = self.parser.parse_with_options(
            &mut |byte_offset, _position| {
                if byte_offset < rope.len_bytes() {
                    let (chunk, chunk_byte_idx, _, _) = rope.chunk_at_byte(byte_offset);
                    &chunk.as_bytes()[byte_offset - chunk_byte_idx..]
                } else {
                    &[] as &[u8]
                }
            },
            self.syntax_tree.as_ref(),
            None,
        );
        self.syntax_tree = tree;
    }
    /// Generates a vector of colorized spans based on syntax highlighting and text content.
    ///
    /// This function processes the raw syntax tree and associated query, if available,
    /// to extract syntax-highlighting information. It then linearizes the highlights,
    /// ensuring no overlapping or invalid regions, and associates text spans with their
    /// corresponding colors.
    ///
    /// ### Returns
    /// A `Vec` of tuples where each tuple contains:
    /// - `String`: A segment of text from the source.
    /// - `crossterm::style::Color`: The color associated with the text segment.
    ///
    /// ### Process Overview
    /// 1. **Extract Raw Highlights**:
    ///    - Iterates over matches in the syntax tree using the query to collect
    ///      capture regions (start byte, end byte, and capture name).
    ///
    /// 2. **Sort Highlights**:
    ///    - Sorts captures first by ascending start byte and then by descending
    ///      end byte to prioritize broader captures.
    ///
    /// 3. **Linearize Highlights**:
    ///    - Iterates over sorted captures and creates non-overlapping spans that
    ///      map text to their corresponding colors. Default styling is applied to
    ///      regions not covered by any capture.
    ///
    /// 4. **Handle Remaining Text**:
    ///    - Appends the remaining text, if any, with default styling after processing
    ///      all highlighted regions.
    ///
    /// ### Edge Cases
    /// - If no syntax tree or query is available, or the text content is empty,
    ///   the function returns an empty vector.
    /// - Gaps between highlighted regions are filled with default color styling.
    /// - Any overlapping or embedded regions are handled by linearization logic.
    ///
    /// ### Dependencies
    /// - `rope` (Text storage and manipulation library)
    /// - `tree-sitter` (For syntax highlighting and querying)
    /// - `crossterm::style::Color` (For terminal-based color representation)
    ///
    /// ### Notes
    /// - This function assumes that `self.rope` contains the text content,
    ///   `self.syntax_tree` is a parsed syntax tree (if available), and
    ///   `self.query` contains the highlighting rules query.
    /// - Custom color logic is implemented in `get_color_for_capture(&name)`.
    ///
    /// ### Theming
    /// - The function uses `theme::FG_DEFAULT` for default, non-highlighted regions.
    /// - Colors for highlights are determined dynamically based on `get_color_for_capture`.
    ///
    /// ### Parameters
    /// - `&self`: A reference to the instance containing the text and parser information.
    ///
    /// ### Returns
    /// - `Vec<(String, crossterm::style::Color)>`: A colorized representation of text spans.
    pub fn get_colored_spans(&self) -> Vec<(String, Color)> {
        let mut spans = Vec::new();
        let total_bytes = self.rope.len_bytes();
        if total_bytes == 0 {
            return spans;
        }

        let mut raw_highlights = Vec::new();
        if let (Some(tree), Some(query)) = (&self.syntax_tree, &self.query) {
            let mut cursor = QueryCursor::new();
            let text_bytes = self.rope.to_string().into_bytes();
            let mut matches = cursor.matches(query, tree.root_node(), text_bytes.as_slice());

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    let start = capture.node.start_byte();
                    let end = capture.node.end_byte();
                    let name = &query.capture_names()[capture.index as usize];
                    raw_highlights.push((start, end, name.to_string()));
                }
            }
        }

        raw_highlights.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));

        let mut current_byte = 0;
        let text_string = self.rope.to_string();
        let text_bytes = text_string.as_bytes();

        for (start, end, name) in raw_highlights {
            if start < current_byte {
                continue;
            }

            if start > current_byte {
                if let Ok(text_slice) = std::str::from_utf8(&text_bytes[current_byte..start]) {
                    spans.push((text_slice.to_string(), theme::FG_DEFAULT));
                }
                current_byte = start;
            }

            if let Ok(text_slice) = std::str::from_utf8(&text_bytes[start..end]) {
                let color = get_color_for_capture(&name);
                spans.push((text_slice.to_string(), color));
                current_byte = end;
            }
        }

        if current_byte < total_bytes
            && let Ok(text_slice) = std::str::from_utf8(&text_bytes[current_byte..total_bytes])
        {
            spans.push((text_slice.to_string(), theme::FG_DEFAULT));
        }
        spans
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ji_insert_and_undo_redo() {
        let mut ji = Ji::default();
        assert!(!ji.is_dirty);
        assert_eq!(ji.rope.to_string(), "");

        ji.insert_char('a');
        ji.insert_char('b');
        ji.insert_char('c');
        assert_eq!(ji.rope.to_string(), "abc");
        assert!(ji.is_dirty);

        ji.undo();
        assert_eq!(ji.rope.to_string(), "ab");

        ji.undo();
        assert_eq!(ji.rope.to_string(), "a");

        ji.redo();
        assert_eq!(ji.rope.to_string(), "ab");

        ji.redo();
        assert_eq!(ji.rope.to_string(), "abc");
    }

    #[test]
    fn test_ji_backspace_and_delete() {
        let mut ji = Ji::default();
        ji.insert_char('h');
        ji.insert_char('e');
        ji.insert_char('l');
        ji.insert_char('l');
        ji.insert_char('o');

        assert_eq!(ji.rope.to_string(), "hello");

        ji.backspace();
        assert_eq!(ji.rope.to_string(), "hell");

        ji.undo();
        assert_eq!(ji.rope.to_string(), "hello");

        ji.cursor_col = 0;
        ji.delete();
        assert_eq!(ji.rope.to_string(), "ello");

        ji.undo();
        assert_eq!(ji.rope.to_string(), "hello");
    }

    #[test]
    fn test_ji_yank_and_paste() {
        let mut ji = Ji::default();
        ji.insert_char('l');
        ji.insert_char('i');
        ji.insert_char('n');
        ji.insert_char('e');
        ji.insert_char('1');
        ji.insert_char('\n');
        ji.insert_char('l');
        ji.insert_char('i');
        ji.insert_char('n');
        ji.insert_char('e');
        ji.insert_char('2');

        ji.cursor_line = 0;
        ji.cursor_col = 0;

        let yanked = ji.yank();
        assert_eq!(yanked, Some("line1\n".to_string()));

        ji.cursor_line = 1;
        ji.cursor_col = 5;
        ji.insert_char('\n');
        ji.paste();

        assert!(ji.rope.to_string().contains("line1\nline2\nline1\n"));

        ji.undo();
        assert_eq!(ji.rope.to_string(), "line1\nline2\n");
    }

    #[test]
    fn test_ji_selection_and_delete() {
        let mut ji = Ji::default();
        ji.insert_char('f');
        ji.insert_char('i');
        ji.insert_char('r');
        ji.insert_char('s');
        ji.insert_char('t');
        ji.insert_char('\n');
        ji.insert_char('s');
        ji.insert_char('e');
        ji.insert_char('c');
        ji.insert_char('o');
        ji.insert_char('n');
        ji.insert_char('d');

        ji.cursor_line = 0;
        ji.select_line();
        assert_eq!(ji.selection, Some((0, 0)));

        let yanked = ji.yank();
        assert_eq!(yanked, Some("first\n".to_string()));

        ji.delete_selection();
        assert_eq!(ji.rope.to_string(), "second");

        ji.undo();
        assert_eq!(ji.rope.to_string(), "first\nsecond");
    }
}
