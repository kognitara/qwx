use crossterm::style::{Color, ResetColor, SetBackgroundColor};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Print, SetForegroundColor},
    terminal::size,
};
use is_executable::IsExecutable;
use std::fmt::{Display, Formatter};
use std::{
    io::{Result, Write},
    path::Path,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use walkdir::WalkDir;

use crate::editor::theme::{
    FINDER_ACTIVE_SELECT, FINDER_BORDER, FINDER_DIR_COLOR, FINDER_FILE_COLOR, UI_TEXT_MUTED,
};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FinderLayout {
    ///
    /// ```txt
    /// ┌──────────────────────────────────────────────────┐
    /// │                   RESEARCH                       │
    /// └──────────────────────────────────────────────────┘
    /// ┌───────────────────────┬──────────────────────────┐
    /// │ ROOT DIRECTORIES      │ SUB ROOTS DIRECTORIES    │
    /// │                       │                          │
    /// │                       │                          │
    /// │                       │                          │
    /// ├───────────────────────┼──────────────────────────┤
    /// │ ROOTS FILES           │ SUB ROOTS FILES          │
    /// │                       │                          │
    /// │                       │                          │
    /// │                       │                          │
    /// └───────────────────────┴──────────────────────────┘
    /// ┌───────────────────────┬──────────────────────────┐
    /// │ ROOT DIRS FOUNDED     │ SUB ROOTS DIRS FOUNDED   │
    /// ├───────────────────────┼──────────────────────────┤
    /// │ SUB ROOT DIRS FOUNDED │ SUB ROOT FILES FOUNDED   │
    /// └───────────────────────┴──────────────────────────┘
    ///```
    Grid,
    ///
    /// ```txt
    /// ┌─────────────────────────────────────────────────┐
    /// │                   RESEARCH                      │
    /// └─────────────────────────────────────────────────┘
    /// ┌───────────────────────┬─────────────────────────┐
    /// │ DIRS                  │ FILES                   │
    /// │                       │                         │
    /// │                       │                         │
    /// │                       │                         │
    /// │                       │                         │
    /// └───────────────────────┴─────────────────────────┘
    /// ┌───────────────────────┬─────────────────────────┐
    /// │ DIRS FOUNDED          │ FILES FOUNDED           │
    /// └───────────────────────┴─────────────────────────┘
    ///```
    SideBySide,
    /// ```txt
    /// ┌─────────────────────────────────────────────────┐
    /// │                   RESEARCH                      │
    /// └─────────────────────────────────────────────────┘
    /// ┌───────────────┬─────────────────┬───────────────┐
    /// │ PARENT DIRS   │ ACTIVE DIR      │ CHILD DIRS    │
    /// │               │                 │               │
    /// │               │                 │               │
    /// │               │                 │               │
    /// │               │                 │               │
    /// ├───────────────┴─────────────────┴───────────────┤
    /// │                    FILES                        │
    /// │                                                 │
    /// │                                                 │
    /// │                                                 │
    /// │                                                 │
    /// │                                                 │
    /// │                                                 │
    /// └─────────────────────────────────────────────────┘
    /// ┌───────────────┬─────────────────┬───────────────┐
    /// │ PARENT FOUNDED│ DIRS FOUNDED    │ CHILD FOUNDED │
    /// └───────────────┴─────────────────┴───────────────┘
    /// ```
    Miller,
    /// ```txt
    /// ┌─────────────────────────────────────────────────┐
    /// │                   RESEARCH                      │
    /// └─────────────────────────────────────────────────┘
    /// ┌─────────────────────────────────────────────────┐
    /// │                  DIRECTORIES                    │
    /// │                                                 │
    /// └─────────────────────────────────────────────────┘
    /// ┌──────────────────────┬──────────────────────────┐
    /// │                      │                          │
    /// │ CURRENT DIRECTORY    │ CURRENT FILES            │
    /// │                      │                          │
    /// │                      │                          │
    /// │                      │                          │
    /// │                      │                          │
    /// │                      │                          │
    /// └──────────────────────┴──────────────────────────┘
    /// ┌──────────────────────┬──────────────────────────┐
    /// │ DIRS FOUNDED         │ FILES FOUNDED            │
    /// └──────────────────────┴──────────────────────────┘
    /// ```
    Commander,
    /// ```txt
    /// ┌──────────────┬──────┬──────┐
    /// │              │ src/ │ app/ │
    /// │    Root /    ├──────┼──────┤
    /// │              │ doc/ │ lib/ │
    /// └──────────────┴──────┴──────┘
    /// ```
    Mosaic,
}

impl Display for FinderLayout {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
impl FinderLayout {
    pub fn next(&self) -> Self {
        match self {
            Self::Grid => Self::SideBySide,
            Self::SideBySide => Self::Miller,
            Self::Miller => Self::Commander,
            Self::Commander => Self::Mosaic,
            Self::Mosaic => Self::Grid,
        }
    }
    pub fn previous(&self) -> Self {
        match self {
            Self::Grid => Self::Mosaic,
            Self::SideBySide => Self::Grid,
            Self::Miller => Self::SideBySide,
            Self::Commander => Self::Miller,
            Self::Mosaic => Self::Commander,
        }
    }

    /// Draw the finder on the screen.
    pub fn draw<W: Write>(
        &self,
        finder: &Finder,
        w: &mut W,
        research: &str,
        start_x: u16,
        start_y: u16,
        width: u16,
        height: u16,
    ) -> Result<()> {
        match self {
            Self::Grid => draw_grid_finder(finder, w, research, start_x, start_y, width, height),
            Self::Commander => {
                draw_commander_finder(finder, w, research, start_x, start_y, width, height)
            }
            Self::Mosaic => {
                draw_mosaic_finder(finder, w, research, start_x, start_y, width, height)
            }
            Self::Miller => {
                draw_miller_finder(finder, w, research, start_x, start_y, width, height)
            }
            Self::SideBySide => {
                draw_side_by_side_finder(finder, w, research, start_x, start_y, width, height)
            }
        }
    }
}

/// Format and truncate or pad item name to protect panel borders
pub fn format_item_name(name: &str, max_width: usize) -> String {
    let clean_name = name.replace('\t', " ").replace('\r', "");
    let text_width = clean_name.width();

    if text_width > max_width {
        if max_width <= 2 {
            return ".".repeat(max_width);
        }
        let mut truncated = String::new();
        let mut acc_width = 0;
        for c in clean_name.chars() {
            let c_w = c.width().unwrap_or(0);
            if acc_width + c_w > max_width.saturating_sub(2) {
                break;
            }
            truncated.push(c);
            acc_width += c_w;
        }
        let padding = " ".repeat(max_width.saturating_sub(acc_width + 2));
        format!("{}..{}", truncated, padding)
    } else {
        let padding = " ".repeat(max_width.saturating_sub(text_width));
        format!("{}{}", clean_name, padding)
    }
}

fn render_list<W: Write>(
    w: &mut W,
    items: &[String],
    selected_index: usize,
    start_x: u16,
    start_y: u16,
    width: usize,
    height: usize,
    default_color: Color,
) -> Result<()> {
    if height == 0 || width == 0 {
        return Ok(());
    }
    let visible_items = height;
    let scroll_offset = if selected_index >= visible_items {
        selected_index - visible_items + 1
    } else {
        0
    };

    for (i, item) in items
        .iter()
        .skip(scroll_offset)
        .take(visible_items)
        .enumerate()
    {
        let absolute_index = scroll_offset + i;
        let y = start_y + i as u16;
        execute!(w, MoveTo(start_x, y))?;

        if absolute_index == selected_index {
            execute!(
                w,
                SetBackgroundColor(FINDER_ACTIVE_SELECT),
                SetForegroundColor(Color::Black)
            )?;
        } else {
            execute!(w, SetForegroundColor(default_color))?;
        }

        let formatted = format_item_name(item, width);
        execute!(w, Print(formatted), ResetColor)?;
    }
    Ok(())
}

fn draw_side_by_side_finder<W: Write>(
    finder: &Finder,
    w: &mut W,
    research: &str,
    start_x: u16,
    start_y: u16,
    width: u16,
    height: u16,
) -> Result<()> {
    // Draw research bar at top
    execute!(
        w,
        MoveTo(start_x, start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat((width - 2) as usize)),
        Print("┐"),
        MoveTo(start_x, start_y + 1),
        Print("│"),
        SetForegroundColor(FINDER_ACTIVE_SELECT),
        Print(format!(" {} ", research)),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, start_y + 1),
        Print("│"),
        MoveTo(start_x, start_y + 2),
        Print("└"),
        Print("─".repeat((width - 2) as usize)),
        Print("┘"),
    )?;

    // Calculate panel dimensions
    let panel_start_y = start_y + 3;
    let panel_height = height.saturating_sub(7);
    let half_width = width / 2;
    let left_inner_width = half_width.saturating_sub(1) as usize;
    let right_inner_width = width.saturating_sub(half_width).saturating_sub(2) as usize;
    let inner_height = panel_height.saturating_sub(1) as usize;

    // Draw left panel (DIRS) border
    execute!(
        w,
        MoveTo(start_x, panel_start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(left_inner_width)),
        Print("┬"),
        Print("─".repeat(right_inner_width)),
        Print("┐"),
    )?;

    // Draw DIRS and FILES labels
    execute!(
        w,
        MoveTo(start_x + 2, panel_start_y),
        SetForegroundColor(FINDER_DIR_COLOR),
        Print(" DIRS "),
        MoveTo(start_x + half_width + 2, panel_start_y),
        SetForegroundColor(FINDER_FILE_COLOR),
        Print(" FILES "),
    )?;

    // Draw middle divider and side borders
    for i in 1..panel_height {
        execute!(
            w,
            MoveTo(start_x, panel_start_y + i),
            SetForegroundColor(FINDER_BORDER),
            Print("│"),
            MoveTo(start_x + half_width, panel_start_y + i),
            Print("│"),
            MoveTo(start_x + width - 1, panel_start_y + i),
            Print("│"),
        )?;
    }

    // Render lists inside panels
    render_list(
        w,
        &finder.directories,
        finder.selected_dir,
        start_x + 1,
        panel_start_y + 1,
        left_inner_width,
        inner_height,
        FINDER_DIR_COLOR,
    )?;

    render_list(
        w,
        &finder.files,
        finder.selected_file,
        start_x + half_width + 1,
        panel_start_y + 1,
        right_inner_width,
        inner_height,
        FINDER_FILE_COLOR,
    )?;

    // Draw bottom border of panels
    let bottom_y = panel_start_y + panel_height;
    execute!(
        w,
        MoveTo(start_x, bottom_y),
        SetForegroundColor(FINDER_BORDER),
        Print("└"),
        Print("─".repeat(left_inner_width)),
        Print("┴"),
        Print("─".repeat(right_inner_width)),
        Print("┘"),
    )?;

    // Draw status footer
    let footer_y = bottom_y + 1;
    execute!(
        w,
        MoveTo(start_x, footer_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(left_inner_width)),
        Print("┬"),
        Print("─".repeat(right_inner_width)),
        Print("┐"),
        MoveTo(start_x, footer_y + 1),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(
            &format!(" DIRS FOUNDED: {}", finder.directories.len()),
            left_inner_width
        )),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + half_width, footer_y + 1),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(
            &format!(" FILES FOUNDED: {}", finder.files.len()),
            right_inner_width
        )),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, footer_y + 1),
        Print("│"),
        MoveTo(start_x, footer_y + 2),
        Print("└"),
        Print("─".repeat(left_inner_width)),
        Print("┴"),
        Print("─".repeat(right_inner_width)),
        Print("┘"),
    )?;

    Ok(())
}

fn draw_grid_finder<W: Write>(
    finder: &Finder,
    w: &mut W,
    research: &str,
    start_x: u16,
    start_y: u16,
    width: u16,
    height: u16,
) -> Result<()> {
    // Draw research bar at top
    execute!(
        w,
        MoveTo(start_x, start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat((width - 2) as usize)),
        Print("┐"),
        MoveTo(start_x, start_y + 1),
        Print("│"),
        SetForegroundColor(FINDER_ACTIVE_SELECT),
        Print(format!(" {} ", research)),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, start_y + 1),
        Print("│"),
        MoveTo(start_x, start_y + 2),
        Print("└"),
        Print("─".repeat((width - 2) as usize)),
        Print("┘"),
    )?;

    // Calculate panel dimensions for 2x2 grid
    let panel_start_y = start_y + 3;
    let panel_height = (height.saturating_sub(9)) / 2;
    let half_width = width / 2;
    let left_inner_width = half_width.saturating_sub(1) as usize;
    let right_inner_width = width.saturating_sub(half_width).saturating_sub(2) as usize;
    let inner_height = panel_height.saturating_sub(1) as usize;

    // Draw top border of 2x2 grid
    execute!(
        w,
        MoveTo(start_x, panel_start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(left_inner_width)),
        Print("┬"),
        Print("─".repeat(right_inner_width)),
        Print("┐"),
    )?;

    // Draw labels for top row
    execute!(
        w,
        MoveTo(start_x + 2, panel_start_y),
        SetForegroundColor(FINDER_DIR_COLOR),
        Print(" ROOT DIRECTORIES "),
        MoveTo(start_x + half_width + 2, panel_start_y),
        Print(" SUB ROOTS DIRECTORIES "),
    )?;

    // Draw sides and middle divider for top panels
    for i in 1..panel_height {
        execute!(
            w,
            MoveTo(start_x, panel_start_y + i),
            SetForegroundColor(FINDER_BORDER),
            Print("│"),
            MoveTo(start_x + half_width, panel_start_y + i),
            Print("│"),
            MoveTo(start_x + width - 1, panel_start_y + i),
            Print("│"),
        )?;
    }

    // Render top lists
    render_list(
        w,
        &finder.directories,
        finder.selected_dir,
        start_x + 1,
        panel_start_y + 1,
        left_inner_width,
        inner_height,
        FINDER_DIR_COLOR,
    )?;

    render_list(
        w,
        &finder.sub_directories,
        finder.selected_sub_dir,
        start_x + half_width + 1,
        panel_start_y + 1,
        right_inner_width,
        inner_height,
        FINDER_DIR_COLOR,
    )?;

    // Draw middle horizontal divider
    let middle_y = panel_start_y + panel_height;
    execute!(
        w,
        MoveTo(start_x, middle_y),
        SetForegroundColor(FINDER_BORDER),
        Print("├"),
        Print("─".repeat(left_inner_width)),
        Print("┼"),
        Print("─".repeat(right_inner_width)),
        Print("┤"),
    )?;

    // Draw labels for bottom row
    execute!(
        w,
        MoveTo(start_x + 2, middle_y),
        SetForegroundColor(FINDER_FILE_COLOR),
        Print(" ROOTS FILES "),
        MoveTo(start_x + half_width + 2, middle_y),
        Print(" SUB ROOTS FILES "),
    )?;

    // Draw sides and middle divider for bottom panels
    for i in 1..panel_height {
        execute!(
            w,
            MoveTo(start_x, middle_y + i),
            SetForegroundColor(FINDER_BORDER),
            Print("│"),
            MoveTo(start_x + half_width, middle_y + i),
            Print("│"),
            MoveTo(start_x + width - 1, middle_y + i),
            Print("│"),
        )?;
    }

    // Render bottom lists
    render_list(
        w,
        &finder.files,
        finder.selected_file,
        start_x + 1,
        middle_y + 1,
        left_inner_width,
        inner_height,
        FINDER_FILE_COLOR,
    )?;

    render_list(
        w,
        &finder.sub_files,
        finder.selected_sub_file,
        start_x + half_width + 1,
        middle_y + 1,
        right_inner_width,
        inner_height,
        FINDER_FILE_COLOR,
    )?;

    // Draw bottom border of main grid
    let bottom_y = middle_y + panel_height;
    execute!(
        w,
        MoveTo(start_x, bottom_y),
        SetForegroundColor(FINDER_BORDER),
        Print("└"),
        Print("─".repeat(left_inner_width)),
        Print("┴"),
        Print("─".repeat(right_inner_width)),
        Print("┘"),
    )?;

    // Draw status footer (2x2 grid for founded counts)
    let footer_y = bottom_y + 1;

    execute!(
        w,
        MoveTo(start_x, footer_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(left_inner_width)),
        Print("┬"),
        Print("─".repeat(right_inner_width)),
        Print("┐"),
        MoveTo(start_x, footer_y + 1),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(
            &format!(" ROOT DIRS FOUNDED: {}", finder.directories.len()),
            left_inner_width
        )),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + half_width, footer_y + 1),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(
            &format!(" SUB ROOTS DIRS FOUNDED: {}", finder.sub_directories.len()),
            right_inner_width
        )),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, footer_y + 1),
        Print("│"),
        MoveTo(start_x, footer_y + 2),
        Print("├"),
        Print("─".repeat(left_inner_width)),
        Print("┼"),
        Print("─".repeat(right_inner_width)),
        Print("┤"),
        MoveTo(start_x, footer_y + 3),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(
            &format!(" ROOTS FILES FOUNDED: {}", finder.files.len()),
            left_inner_width
        )),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + half_width, footer_y + 3),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(
            &format!(" SUB ROOT FILES FOUNDED: {}", finder.sub_files.len()),
            right_inner_width
        )),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, footer_y + 3),
        Print("│"),
        MoveTo(start_x, footer_y + 4),
        Print("└"),
        Print("─".repeat(left_inner_width)),
        Print("┴"),
        Print("─".repeat(right_inner_width)),
        Print("┘"),
    )?;

    Ok(())
}

fn draw_miller_finder<W: Write>(
    finder: &Finder,
    w: &mut W,
    research: &str,
    start_x: u16,
    start_y: u16,
    width: u16,
    height: u16,
) -> Result<()> {
    // Draw research bar at top
    execute!(
        w,
        MoveTo(start_x, start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat((width - 2) as usize)),
        Print("┐"),
        MoveTo(start_x, start_y + 1),
        Print("│"),
        SetForegroundColor(FINDER_ACTIVE_SELECT),
        Print(format!(" {} ", research)),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, start_y + 1),
        Print("│"),
        MoveTo(start_x, start_y + 2),
        Print("└"),
        Print("─".repeat((width - 2) as usize)),
        Print("┘"),
    )?;

    // Calculate panel dimensions
    let panel_start_y = start_y + 3;
    let third_width = width / 3;
    let col1_width = third_width.saturating_sub(1) as usize;
    let col2_width = third_width.saturating_sub(1) as usize;
    let col3_width = width.saturating_sub(2 * third_width).saturating_sub(2) as usize;
    let full_inner_width = width.saturating_sub(2) as usize;

    let available_height = height.saturating_sub(8);
    let dirs_height = available_height / 2;
    let files_height = available_height.saturating_sub(dirs_height);
    let dirs_inner_height = dirs_height.saturating_sub(2) as usize;
    let files_inner_height = files_height.saturating_sub(2) as usize;

    // Draw top border of three-column directory panels
    execute!(
        w,
        MoveTo(start_x, panel_start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(col1_width)),
        Print("┬"),
        Print("─".repeat(col2_width)),
        Print("┬"),
        Print("─".repeat(col3_width)),
        Print("┐"),
    )?;

    // Draw labels for directory columns
    execute!(
        w,
        MoveTo(start_x + 2, panel_start_y),
        SetForegroundColor(FINDER_DIR_COLOR),
        Print(" PARENT DIRS "),
        MoveTo(start_x + third_width + 2, panel_start_y),
        Print(" ACTIVE DIR "),
        MoveTo(start_x + 2 * third_width + 2, panel_start_y),
        Print(" CHILD DIRS "),
    )?;

    // Draw sides and vertical dividers for directory panels
    for i in 1..dirs_height {
        execute!(
            w,
            MoveTo(start_x, panel_start_y + i),
            SetForegroundColor(FINDER_BORDER),
            Print("│"),
            MoveTo(start_x + third_width, panel_start_y + i),
            Print("│"),
            MoveTo(start_x + 2 * third_width, panel_start_y + i),
            Print("│"),
            MoveTo(start_x + width - 1, panel_start_y + i),
            Print("│"),
        )?;
    }

    // Render lists for top 3 columns
    let parent_dirs = vec!["..".to_string()];
    render_list(
        w,
        &parent_dirs,
        usize::MAX,
        start_x + 1,
        panel_start_y + 1,
        col1_width,
        dirs_inner_height,
        FINDER_DIR_COLOR,
    )?;

    render_list(
        w,
        &finder.directories,
        finder.selected_dir,
        start_x + third_width + 1,
        panel_start_y + 1,
        col2_width,
        dirs_inner_height,
        FINDER_DIR_COLOR,
    )?;

    render_list(
        w,
        &finder.sub_directories,
        finder.selected_sub_dir,
        start_x + 2 * third_width + 1,
        panel_start_y + 1,
        col3_width,
        dirs_inner_height,
        FINDER_DIR_COLOR,
    )?;

    // Draw horizontal divider between directories and files
    let files_start_y = panel_start_y + dirs_height;
    execute!(
        w,
        MoveTo(start_x, files_start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("├"),
        Print("─".repeat(col1_width)),
        Print("┴"),
        Print("─".repeat(col2_width)),
        Print("┴"),
        Print("─".repeat(col3_width)),
        Print("┤"),
    )?;

    // Draw FILES label
    execute!(
        w,
        MoveTo(start_x + 2, files_start_y),
        SetForegroundColor(FINDER_FILE_COLOR),
        Print(" FILES "),
    )?;

    // Draw sides for files panel
    for i in 1..files_height {
        execute!(
            w,
            MoveTo(start_x, files_start_y + i),
            SetForegroundColor(FINDER_BORDER),
            Print("│"),
            MoveTo(start_x + width - 1, files_start_y + i),
            Print("│"),
        )?;
    }

    // Render files list
    render_list(
        w,
        &finder.files,
        finder.selected_file,
        start_x + 1,
        files_start_y + 1,
        full_inner_width,
        files_inner_height,
        FINDER_FILE_COLOR,
    )?;

    // Draw bottom border of files panel
    let bottom_y = files_start_y + files_height;
    execute!(
        w,
        MoveTo(start_x, bottom_y),
        SetForegroundColor(FINDER_BORDER),
        Print("└"),
        Print("─".repeat(full_inner_width)),
        Print("┘"),
    )?;

    // Draw status footer
    let footer_y = bottom_y + 1;
    execute!(
        w,
        MoveTo(start_x, footer_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(col1_width)),
        Print("┬"),
        Print("─".repeat(col2_width)),
        Print("┬"),
        Print("─".repeat(col3_width)),
        Print("┐"),
        MoveTo(start_x, footer_y + 1),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(" PARENT: ..", col1_width)),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + third_width, footer_y + 1),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(
            &format!(" DIRS FOUNDED: {}", finder.directories.len()),
            col2_width
        )),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + 2 * third_width, footer_y + 1),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(
            &format!(" CHILD FOUNDED: {}", finder.sub_directories.len()),
            col3_width
        )),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, footer_y + 1),
        Print("│"),
        MoveTo(start_x, footer_y + 2),
        Print("└"),
        Print("─".repeat(col1_width)),
        Print("┴"),
        Print("─".repeat(col2_width)),
        Print("┴"),
        Print("─".repeat(col3_width)),
        Print("┘"),
    )?;

    Ok(())
}

fn draw_commander_finder<W: Write>(
    finder: &Finder,
    w: &mut W,
    research: &str,
    start_x: u16,
    start_y: u16,
    width: u16,
    height: u16,
) -> Result<()> {
    // Draw research bar at top
    execute!(
        w,
        MoveTo(start_x, start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat((width - 2) as usize)),
        Print("┐"),
        MoveTo(start_x, start_y + 1),
        Print("│"),
        SetForegroundColor(FINDER_ACTIVE_SELECT),
        Print(format!(" {} ", research)),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, start_y + 1),
        Print("│"),
        MoveTo(start_x, start_y + 2),
        Print("└"),
        Print("─".repeat((width - 2) as usize)),
        Print("┘"),
    )?;

    // Calculate panel dimensions
    let panel_start_y = start_y + 3;
    let dirs_height = 3;
    let main_panel_start_y = panel_start_y + dirs_height + 1;
    let main_panel_height = height.saturating_sub(8 + dirs_height);
    let half_width = width / 2;
    let full_inner_width = width.saturating_sub(2) as usize;
    let left_inner_width = half_width.saturating_sub(1) as usize;
    let right_inner_width = width.saturating_sub(half_width).saturating_sub(2) as usize;
    let main_inner_height = main_panel_height.saturating_sub(1) as usize;

    // Draw directory section border
    execute!(
        w,
        MoveTo(start_x, panel_start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(full_inner_width)),
        Print("┐"),
        MoveTo(start_x, panel_start_y + 1),
        Print("│"),
        SetForegroundColor(FINDER_DIR_COLOR),
        Print(" DIRECTORIES "),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, panel_start_y + 1),
        Print("│"),
    )?;

    // Draw sides for directories section
    for i in 2..dirs_height {
        execute!(
            w,
            MoveTo(start_x, panel_start_y + i),
            SetForegroundColor(FINDER_BORDER),
            Print("│"),
            MoveTo(start_x + width - 1, panel_start_y + i),
            Print("│"),
        )?;
    }

    // Render directories in top bar if any
    let dirs_joined = finder.directories.join("   ");
    execute!(
        w,
        MoveTo(start_x + 15, panel_start_y + 1),
        SetForegroundColor(FINDER_DIR_COLOR),
        Print(format_item_name(
            &dirs_joined,
            full_inner_width.saturating_sub(14)
        )),
        ResetColor,
    )?;

    // Draw bottom border of directories section
    execute!(
        w,
        MoveTo(start_x, panel_start_y + dirs_height),
        SetForegroundColor(FINDER_BORDER),
        Print("└"),
        Print("─".repeat(full_inner_width)),
        Print("┘"),
    )?;

    // Draw main panel (split: CURRENT DIRECTORY | CURRENT FILES)
    execute!(
        w,
        MoveTo(start_x, main_panel_start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(left_inner_width)),
        Print("┬"),
        Print("─".repeat(right_inner_width)),
        Print("┐"),
    )?;

    // Draw labels for main panels
    execute!(
        w,
        MoveTo(start_x + 2, main_panel_start_y),
        SetForegroundColor(FINDER_DIR_COLOR),
        Print(" CURRENT DIRECTORY "),
        MoveTo(start_x + half_width + 2, main_panel_start_y),
        SetForegroundColor(FINDER_FILE_COLOR),
        Print(" CURRENT FILES "),
    )?;

    // Draw sides and middle divider for main panels
    for i in 1..main_panel_height {
        execute!(
            w,
            MoveTo(start_x, main_panel_start_y + i),
            SetForegroundColor(FINDER_BORDER),
            Print("│"),
            MoveTo(start_x + half_width, main_panel_start_y + i),
            Print("│"),
            MoveTo(start_x + width - 1, main_panel_start_y + i),
            Print("│"),
        )?;
    }

    // Render lists inside main panels
    render_list(
        w,
        &finder.directories,
        finder.selected_dir,
        start_x + 1,
        main_panel_start_y + 1,
        left_inner_width,
        main_inner_height,
        FINDER_DIR_COLOR,
    )?;

    render_list(
        w,
        &finder.files,
        finder.selected_file,
        start_x + half_width + 1,
        main_panel_start_y + 1,
        right_inner_width,
        main_inner_height,
        FINDER_FILE_COLOR,
    )?;

    // Draw bottom border of main panels
    let bottom_y = main_panel_start_y + main_panel_height;
    execute!(
        w,
        MoveTo(start_x, bottom_y),
        SetForegroundColor(FINDER_BORDER),
        Print("└"),
        Print("─".repeat(left_inner_width)),
        Print("┴"),
        Print("─".repeat(right_inner_width)),
        Print("┘"),
    )?;

    // Draw status footer
    let footer_y = bottom_y + 1;
    execute!(
        w,
        MoveTo(start_x, footer_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(left_inner_width)),
        Print("┬"),
        Print("─".repeat(right_inner_width)),
        Print("┐"),
        MoveTo(start_x, footer_y + 1),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(
            &format!(" DIRS FOUNDED: {}", finder.directories.len()),
            left_inner_width
        )),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + half_width, footer_y + 1),
        Print("│"),
        SetForegroundColor(UI_TEXT_MUTED),
        Print(format_item_name(
            &format!(" FILES FOUNDED: {}", finder.files.len()),
            right_inner_width
        )),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, footer_y + 1),
        Print("│"),
        MoveTo(start_x, footer_y + 2),
        Print("└"),
        Print("─".repeat(left_inner_width)),
        Print("┴"),
        Print("─".repeat(right_inner_width)),
        Print("┘"),
    )?;

    Ok(())
}

fn execute_label<W: Write>(
    w: &mut W,
    x: u16,
    y: u16,
    label: &str,
    available_width: usize,
    color: Color,
) -> Result<()> {
    if available_width <= 2 {
        return Ok(());
    }
    let max_len = available_width.saturating_sub(2);
    let display_text = if label.len() > max_len {
        &label[..max_len]
    } else {
        label
    };
    execute!(
        w,
        MoveTo(x, y),
        SetForegroundColor(color),
        Print(display_text),
    )?;
    Ok(())
}

fn render_mosaic_info<W: Write>(
    w: &mut W,
    finder: &Finder,
    start_x: u16,
    start_y: u16,
    width: usize,
    height: usize,
) -> Result<()> {
    if width == 0 || height == 0 {
        return Ok(());
    }

    let mut info_lines = Vec::new();
    info_lines.push(format!("Dirs:      {}", finder.directories.len()));
    info_lines.push(format!("Sub-dirs:  {}", finder.sub_directories.len()));
    info_lines.push(format!("Files:     {}", finder.files.len()));
    info_lines.push(format!("Sub-files: {}", finder.sub_files.len()));

    if let Some(sel) = finder.files.get(finder.selected_file) {
        info_lines.push(format!("Active:    {}", sel));
    } else if let Some(sel) = finder.directories.get(finder.selected_dir) {
        info_lines.push(format!("Active:    {}", sel));
    }

    for (i, line) in info_lines.iter().take(height).enumerate() {
        let y = start_y + i as u16;
        execute!(
            w,
            MoveTo(start_x, y),
            SetForegroundColor(UI_TEXT_MUTED),
            Print(format_item_name(line, width)),
            ResetColor
        )?;
    }
    Ok(())
}

fn draw_mosaic_finder<W: Write>(
    finder: &Finder,
    w: &mut W,
    research: &str,
    start_x: u16,
    start_y: u16,
    width: u16,
    height: u16,
) -> Result<()> {
    if width < 12 || height < 6 {
        return Ok(());
    }

    // 1. Draw top research bar
    let search_inner_w = width.saturating_sub(2) as usize;
    let truncated_research = format_item_name(&format!(" {} ", research), search_inner_w);
    execute!(
        w,
        MoveTo(start_x, start_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(search_inner_w)),
        Print("┐"),
        MoveTo(start_x, start_y + 1),
        Print("│"),
        SetForegroundColor(FINDER_ACTIVE_SELECT),
        Print(truncated_research),
        SetForegroundColor(FINDER_BORDER),
        MoveTo(start_x + width - 1, start_y + 1),
        Print("│"),
        MoveTo(start_x, start_y + 2),
        Print("└"),
        Print("─".repeat(search_inner_w)),
        Print("┘"),
    )?;

    // 2. Layout calculations
    let has_footer = height >= 14;
    let footer_height = if has_footer { 3 } else { 0 };
    let panel_start_y = start_y + 3;
    let available_panel_h =
        height.saturating_sub(3 + footer_height + if has_footer { 1 } else { 0 });
    if available_panel_h < 3 {
        return Ok(());
    }

    let top_y = panel_start_y;
    let total_cell_rows = available_panel_h.saturating_sub(3);
    let top_cell_h = total_cell_rows / 2;
    let bot_cell_h = total_cell_rows.saturating_sub(top_cell_h);
    let mid_y = top_y + 1 + top_cell_h;
    let bot_y = mid_y + 1 + bot_cell_h;
    let left_inner_h = (top_cell_h + 1 + bot_cell_h) as usize;

    let total_inner_w = width.saturating_sub(4) as usize;
    let inner_w_left = (total_inner_w * 35 / 100).max(1);
    let rem_w = total_inner_w.saturating_sub(inner_w_left);
    let inner_w_mid = (rem_w / 2).max(1);
    let inner_w_right = rem_w.saturating_sub(inner_w_mid).max(1);

    let x0 = start_x;
    let x1 = x0 + (inner_w_left as u16) + 1;
    let x2 = x1 + (inner_w_mid as u16) + 1;
    let x3 = start_x + width - 1;

    // 3. Draw top border of mosaic
    execute!(
        w,
        MoveTo(x0, top_y),
        SetForegroundColor(FINDER_BORDER),
        Print("┌"),
        Print("─".repeat(inner_w_left)),
        Print("┬"),
        Print("─".repeat(inner_w_mid)),
        Print("┬"),
        Print("─".repeat(inner_w_right)),
        Print("┐"),
    )?;

    // Draw top row labels
    execute_label(
        w,
        x0 + 2,
        top_y,
        " DIRECTORIES ",
        inner_w_left,
        FINDER_DIR_COLOR,
    )?;
    execute_label(
        w,
        x1 + 2,
        top_y,
        " SUB DIRECTORIES ",
        inner_w_mid,
        FINDER_DIR_COLOR,
    )?;
    execute_label(
        w,
        x2 + 2,
        top_y,
        " FILES ",
        inner_w_right,
        FINDER_FILE_COLOR,
    )?;

    // 4. Draw vertical dividers for top half
    for y in (top_y + 1)..mid_y {
        execute!(
            w,
            MoveTo(x0, y),
            SetForegroundColor(FINDER_BORDER),
            Print("│"),
            MoveTo(x1, y),
            Print("│"),
            MoveTo(x2, y),
            Print("│"),
            MoveTo(x3, y),
            Print("│"),
        )?;
    }

    // 5. Draw middle horizontal divider
    execute!(
        w,
        MoveTo(x0, mid_y),
        SetForegroundColor(FINDER_BORDER),
        Print("│"),
        MoveTo(x1, mid_y),
        Print("├"),
        Print("─".repeat(inner_w_mid)),
        Print("┼"),
        Print("─".repeat(inner_w_right)),
        Print("┤"),
    )?;

    // Draw middle labels
    execute_label(
        w,
        x1 + 2,
        mid_y,
        " SUB FILES ",
        inner_w_mid,
        FINDER_FILE_COLOR,
    )?;
    execute_label(w, x2 + 2, mid_y, " INFO ", inner_w_right, UI_TEXT_MUTED)?;

    // 6. Draw vertical dividers for bottom half
    for y in (mid_y + 1)..bot_y {
        execute!(
            w,
            MoveTo(x0, y),
            SetForegroundColor(FINDER_BORDER),
            Print("│"),
            MoveTo(x1, y),
            Print("│"),
            MoveTo(x2, y),
            Print("│"),
            MoveTo(x3, y),
            Print("│"),
        )?;
    }

    // 7. Draw bottom border of mosaic
    execute!(
        w,
        MoveTo(x0, bot_y),
        SetForegroundColor(FINDER_BORDER),
        Print("└"),
        Print("─".repeat(inner_w_left)),
        Print("┴"),
        Print("─".repeat(inner_w_mid)),
        Print("┴"),
        Print("─".repeat(inner_w_right)),
        Print("┘"),
    )?;

    // 8. Render contents in panels
    render_list(
        w,
        &finder.directories,
        finder.selected_dir,
        x0 + 1,
        top_y + 1,
        inner_w_left,
        left_inner_h,
        FINDER_DIR_COLOR,
    )?;

    render_list(
        w,
        &finder.sub_directories,
        finder.selected_sub_dir,
        x1 + 1,
        top_y + 1,
        inner_w_mid,
        top_cell_h as usize,
        FINDER_DIR_COLOR,
    )?;

    render_list(
        w,
        &finder.files,
        finder.selected_file,
        x2 + 1,
        top_y + 1,
        inner_w_right,
        top_cell_h as usize,
        FINDER_FILE_COLOR,
    )?;

    render_list(
        w,
        &finder.sub_files,
        finder.selected_sub_file,
        x1 + 1,
        mid_y + 1,
        inner_w_mid,
        bot_cell_h as usize,
        FINDER_FILE_COLOR,
    )?;

    render_mosaic_info(
        w,
        finder,
        x2 + 1,
        mid_y + 1,
        inner_w_right,
        bot_cell_h as usize,
    )?;

    // 9. Render status footer if space permits
    if has_footer {
        let footer_y = bot_y + 1;
        execute!(
            w,
            MoveTo(x0, footer_y),
            SetForegroundColor(FINDER_BORDER),
            Print("┌"),
            Print("─".repeat(inner_w_left)),
            Print("┬"),
            Print("─".repeat(inner_w_mid)),
            Print("┬"),
            Print("─".repeat(inner_w_right)),
            Print("┐"),
            MoveTo(x0, footer_y + 1),
            Print("│"),
            SetForegroundColor(UI_TEXT_MUTED),
            Print(format_item_name(
                &format!(" DIRS: {}", finder.directories.len()),
                inner_w_left,
            )),
            SetForegroundColor(FINDER_BORDER),
            MoveTo(x1, footer_y + 1),
            Print("│"),
            SetForegroundColor(UI_TEXT_MUTED),
            Print(format_item_name(
                &format!(" SUB DIRS: {}", finder.sub_directories.len()),
                inner_w_mid,
            )),
            SetForegroundColor(FINDER_BORDER),
            MoveTo(x2, footer_y + 1),
            Print("│"),
            SetForegroundColor(UI_TEXT_MUTED),
            Print(format_item_name(
                &format!(" FILES: {}", finder.files.len()),
                inner_w_right,
            )),
            SetForegroundColor(FINDER_BORDER),
            MoveTo(x3, footer_y + 1),
            Print("│"),
            MoveTo(x0, footer_y + 2),
            Print("└"),
            Print("─".repeat(inner_w_left)),
            Print("┴"),
            Print("─".repeat(inner_w_mid)),
            Print("┴"),
            Print("─".repeat(inner_w_right)),
            Print("┘"),
        )?;
    }

    Ok(())
}

/// Deep search recursive function
pub fn deep_search_recursive(query: &str, results: &mut Vec<String>) {
    let query_lower = query.to_lowercase();
    results.clear();

    let (modifier, target) = if let Some(t) = query_lower.strip_prefix('=') {
        ('=', t)
    } else if let Some(t) = query_lower.strip_prefix('^') {
        ('^', t)
    } else if let Some(t) = query_lower.strip_prefix('$') {
        ('$', t)
    } else if let Some(t) = query_lower.strip_prefix('!') {
        ('!', t)
    } else {
        ('*', query_lower.as_str())
    };

    let walk = ignore::WalkBuilder::new(".")
        .threads(num_cpus::get())
        .standard_filters(true)
        .add_custom_ignore_filename(".gitignore")
        .add_custom_ignore_filename(".awqignore")
        .add_custom_ignore_filename(".hgignore")
        .add_custom_ignore_filename(".dockerignore")
        .build();

    for entry in walk.flatten() {
        let path = entry.path();
        if path.is_file() {
            let path_str = path.to_string_lossy().to_string().replace("./", "");
            let name_lower = path_str.to_lowercase();

            // On applique la bonne règle de filtrage selon le modificateur
            let is_match = match modifier {
                '=' => {
                    name_lower == target
                        || path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n.to_lowercase() == target)
                            .unwrap_or(false)
                }
                '^' => name_lower.starts_with(target),
                '$' => name_lower.ends_with(target),
                '!' => !name_lower.contains(target),
                _ => name_lower.contains(target),
            };

            if is_match && !results.contains(&path_str) {
                results.push(path_str);
            }
        }
    }
}
/// Recursively lists all files in the specified directory, returning their canonicalized paths as a vector of strings.
///
/// This function uses the `WalkDir` crate to traverse the given directory at the specified `path`.
/// Only files (not directories) are included in the resulting list, and their paths are canonicalized
/// and returned as UTF-8 strings.
///
/// # Parameters
/// - `path`: A reference to a `Path` that represents the directory to be scanned for files.
///
/// # Returns
/// - A `Vec<String>` containing the canonicalized paths (as strings) of all files in the directory.
///
/// # Behavior
/// - The function performs a shallow scan of the directory using `WalkDir` with `max_depth(1)`
///   and `min_depth(1)`. This means it only lists files directly within the provided directory
///   without diving into subdirectories.
/// - File paths are canonicalized to provide absolute paths and are sanitized by removing
///   any leading `./` from the path string.
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use qwx::finder::list_files;
/// let files = list_files(Path::new("./my_directory"));
/// for file in files {
///     println!("{file}");
/// }
/// ```
///
/// # Dependencies
/// - The `WalkDir` crate is required for directory traversal. Ensure the crate is included
///   in your `Cargo.toml`.
///
/// # Panics
/// - The function will panic if the `canonicalize` method fails for any file path. This could
///   happen if there are underlying issues with the filesystem or permissions.
pub fn list_files(path: &Path) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    for entry in WalkDir::new(path)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .flatten()
    {
        if entry.path().is_file() && !entry.path().is_executable() {
            let path_str = entry
                .path()
                .canonicalize()
                .expect("failed to canonicalize file path")
                .to_string_lossy()
                .to_string()
                .replace("./", "");
            files.push(path_str);
        }
    }
    files
}
///
/// Recursively lists all directories at the root level of the specified path.
///
/// This function traverses a given path using `WalkDir` with a maximum and minimum depth of 1,
/// effectively listing only the immediate subdirectories present in the specified path.
/// For each directory found, its name (as a `String`) is added to the output vector.
///
/// # Arguments
///
/// * `path` - A reference to a `Path` specifying the root directory to search for subdirectories.
///
/// # Returns
///
/// A `Vec<String>` containing the names of all subdirectories found within the specified path.
///
/// # Example
///
/// ```rust
/// use std::path::Path;
/// use qwx::finder::list_dirs;
///
/// let path = Path::new("/some/directory");
/// let subdirectories = list_dirs(path);
///
/// for dir in subdirectories {
///     println!("{}", dir);
/// }
/// ```
///
/// # Dependencies
///
/// This function depends on the `WalkDir` crate for directory traversal. Make sure to include it
/// in your `Cargo.toml`:
///
/// ```toml
/// [dependencies]
/// walkdir = "2.3"
/// ```
///
/// # Notes
///
/// * Only immediate subdirectories are listed. Files and nested subdirectories are ignored.
/// * The directory names are returned as `String` in UTF-8 format.
/// * If a directory name cannot be converted to `String` (non-UTF-8), it will not be included
///   in the output vector.
pub fn list_dirs(path: &Path) -> Vec<String> {
    let mut dirs: Vec<String> = Vec::new();
    for entry in WalkDir::new(path)
        .max_depth(1)
        .min_depth(1)
        .into_iter()
        .flatten()
    {
        if entry.path().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                dirs.push(name.to_string());
            }
        }
    }
    dirs
}

/// Lists the subdirectories located at a depth of 2 relative to the given path.
///
/// This function traverses the directory structure starting from the specified `path`
/// and collects the relative paths of all subdirectories that are exactly at a depth of 2
/// from the root directory. The traversal is performed using the `WalkDir` crate.
///
/// # Parameters
/// - `path`: A reference to a `Path` specifying the root directory from which to start the search.
///
/// # Returns
/// A `Vec<String>` containing the relative paths of subdirectories at depth 2. Each string in the vector
/// represents a subdirectory path relative to the provided root directory. For example, if the input path
/// is `/root` and there is a subdirectory `/root/parent/child`, the function will return
/// `"parent/child"` if `parent` is at depth 1 and `child` is at depth 2.
///
/// # Panics
/// This function does not handle panics directly. However, if the provided path does not exist
/// or is not accessible, the iteration may result in errors which are silently ignored due
/// to the use of `.flatten()` when iterating over directory entries.
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use qwx::finder::list_sub_dirs;
/// let root_path = Path::new("/example/path");
/// let sub_dirs = list_sub_dirs(root_path);
///
/// for dir in sub_dirs {
///     println!("{}", dir);
/// }
/// ```
///
/// # Dependencies
/// This function depends on the external crate `walkdir` for directory traversal.
/// Ensure the crate is included as a dependency in your `Cargo.toml`:
///
/// ```toml
/// [dependencies]
/// walkdir = "2.3"
/// ```
pub fn list_sub_dirs(path: &Path) -> Vec<String> {
    let mut dirs: Vec<String> = Vec::new();
    for entry in WalkDir::new(path)
        .max_depth(2)
        .min_depth(2)
        .into_iter()
        .flatten()
    {
        if entry.path().is_dir() {
            // Pour les sous-dossiers, on garde le chemin relatif propre (ex: "parent/enfant")
            if let Ok(rel_path) = entry.path().strip_prefix(path) {
                dirs.push(rel_path.to_string_lossy().to_string());
            }
        }
    }
    dirs
}

/// Generates a list of files located within the subdirectories of a specified directory.
///
/// This function traverses the directory tree starting at the provided path and identifies
/// files in subdirectories. The function only includes files that are exactly two levels
/// deeper than the root directory (subdirectories of the provided path).
///
/// # Arguments
///
/// * `path` - A reference to a `Path` object pointing to the directory to traverse.
///
/// # Returns
///
/// * A `Vec<String>` containing the relative paths of files found in subdirectories.
///   Each path is relative to the root directory specified by the input `path`.
///
/// # Example
///
/// ```rust
/// use std::path::Path;
/// use qwx::finder::list_sub_files;
///
/// let path = Path::new("/path/to/root");
/// let files = list_sub_files(path);
/// for file in files {
///     println!("{file}");
/// }
/// ```
///
/// # Notes
///
/// * The function uses a depth-based filter. Files found directly in the `root` directory
///   or in subdirectories more than two levels deep are ignored.
/// * The `strip_prefix` method ensures that paths in the result are relative and clean
///   (e.g., `"parent/enfant"`).
///
/// # Dependencies
///
/// This function relies on the `walkdir` crate for directory traversal. Make sure to add
/// `walkdir` to your `Cargo.toml` to use this function:
///
/// ```toml
/// [dependencies]
/// walkdir = "2.3"
/// ```
pub fn list_sub_files(path: &Path) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    for entry in WalkDir::new(path)
        .max_depth(2)
        .min_depth(2)
        .into_iter()
        .flatten()
    {
        if entry.path().is_file() && !entry.path().is_executable() {
            if let Ok(rel_path) = entry.path().strip_prefix(path) {
                files.push(rel_path.to_string_lossy().to_string());
            }
        }
    }
    files
}
/// The `Finder` struct represents a file and directory management system
/// with mechanisms to navigate and select directories and files.
/// It is designed to facilitate operations such as deep searching,
/// directory and file selection, and maintaining layout-related settings.
///
/// # Fields
/// - `layout`:
///     The layout configuration of the `Finder`. This is of type `FinderLayout`,
///     which determines the UI structure of the finder tool.
///
/// - `directories`:
///     A vector of strings representing the directories currently loaded or available in the `Finder`.
///
/// - `sub_directories`:
///     A vector of strings containing subdirectories within the currently selected directory.
///
/// - `sub_files`:
///     A vector of strings containing files within the currently selected directory's subdirectories.
///
/// - `files`:
///     A vector of strings containing files directly accessible in the currently selected directory.
///
/// - `base_directories`:
///     A vector of strings containing the base directories available in the initial state of the `Finder`.
///
/// - `base_sub_directories`:
///     A vector of strings containing the subdirectories related to the `base_directories`.
///
/// - `base_sub_files`:
///     A vector of strings containing the base files related to the `base_directories` and `base_sub_directories`.
///
/// - `selected_file`:
///     A public field representing the index of the currently selected file in the `files` vector.
///
/// - `base_files`:
///     A vector of strings representing the files associated with or derived from the `base_directories`.
///
/// - `deep_search_cache`:
///     An optional tuple containing a string identifier and a vector of strings for caching the results of deep searches.
///     The first element is a keyword or identifier, and the second element represents the cached list of matching files or directories.
///
/// - `selected_dir`:
///     A public field representing the index of the currently selected directory in the `directories` vector.
///
/// - `selected_sub_dir`:
///     A public field representing the index of the currently selected subdirectory in the `sub_directories` vector.
///
/// - `selected_sub_file`:
///     A public field representing the index of the currently selected subfile in the `sub_files` vector.
///
/// - `width`:
///     The width dimension (in units) of the `Finder` layout, used for display or visualization purposes.
///
/// - `height`:
///     The height dimension (in units) of the `Finder` layout, used for display or visualization purposes.
#[derive(Clone)]
pub struct Finder {
    pub layout: FinderLayout,
    directories: Vec<String>,
    sub_directories: Vec<String>,
    sub_files: Vec<String>,
    files: Vec<String>,
    base_directories: Vec<String>,
    base_sub_directories: Vec<String>,
    base_sub_files: Vec<String>,
    pub selected_file: usize,
    base_files: Vec<String>,
    deep_search_cache: Option<(String, Vec<String>)>,
    pub selected_dir: usize,
    pub selected_sub_dir: usize,
    pub selected_sub_file: usize,
    width: u16,
    height: u16,
}

impl Finder {
    /// Creates a new `Finder` instance with the provided `path` and `layout`.
    ///
    /// # Arguments
    ///
    /// * `path` - A reference to the `Path` which represents the directory to be
    ///            used as the base for the finder.
    /// * `layout` - A `FinderLayout` instance that specifies how the finder should
    ///              be visually structured.
    ///
    /// # Returns
    ///
    /// A new instance
    #[must_use]
    pub fn new(path: &Path, layout: FinderLayout) -> Self {
        let (w, h) = size().unwrap_or((80, 100));
        let dirs = list_dirs(path);
        let files = list_files(path);
        let s_dirs = list_sub_dirs(path);
        let s_files = list_sub_files(path);
        Self {
            layout,
            directories: dirs.clone(),
            files: files.clone(),
            sub_directories: s_dirs.clone(),
            sub_files: s_files.clone(),
            selected_dir: 0,
            selected_sub_dir: 0,
            selected_sub_file: 0,
            selected_file: 0,
            deep_search_cache: None,
            width: w,
            height: h,
            base_directories: dirs,
            base_files: files,
            base_sub_directories: s_dirs,
            base_sub_files: s_files,
        }
    }
    pub fn show<W: Write>(
        &self,
        w: &mut W,
        f: &mut Finder,
        research: &mut str,
        start_x: u16,
        start_y: u16,
        width: u16,
        height: u16,
    ) -> Result<()> {
        execute!(w, Clear(ClearType::All))?;
        self.layout
            .draw(f, w, research, start_x, start_y, width, height)
    }

    /// Retrieves a list of subdirectory names.
    ///
    /// # Returns
    /// A `Vec<String>` containing the names of all subdirectories.
    pub fn get_sub_directories(&self) -> Vec<String> {
        self.sub_directories.to_vec()
    }

    /// Selects the next subdirectory in the list.
    pub fn next_sub_dir(&mut self) {
        if !self.sub_directories.is_empty() {
            self.selected_sub_dir = (self.selected_sub_dir + 1) % self.sub_directories.len();
        }
    }
    /// Selects the previous subdirectory in the list.
    pub fn prev_sub_dir(&mut self) {
        if !self.sub_directories.is_empty() {
            self.selected_sub_dir = if self.selected_sub_dir > 0 {
                self.selected_sub_dir - 1
            } else {
                self.sub_directories.len() - 1
            };
        }
    }
    /// Advances the selected file to the next file in the list.
    ///
    /// This function increments the index of the currently selected file (`selected_file`)
    /// by 1. If the end of the list is reached, it wraps around to the beginning (circular
    /// indexing). The function ensures that the operation is only performed when the
    /// `files` list is not empty.
    ///
    /// # Behavior
    /// - If the `files` list is empty, the function does nothing.
    /// - If the `files` list contains elements, the `selected_file` index is updated
    ///   to point to the next file in a circular manner.
    ///
    /// # Requirements
    /// - The `files` field must be a non-empty vector containing file names or paths.
    /// - The `selected_file` field must be a valid index within the bounds of the `files` vector.
    pub fn next_file(&mut self) {
        if !self.files.is_empty() {
            self.selected_file = (self.selected_file + 1) % self.files.len();
        }
    }
    /// Moves the selection to the previous file in the list of files.
    ///
    /// If the current selection is not the first file, the `selected_file` index is decremented by one.
    /// If the current selection is the first file, the `selected_file` wraps around to the last file
    /// in the list, implementing circular navigation.
    ///
    /// # Behavior
    /// - If the `files` list is empty, this function does nothing.
    /// - If `selected_file` is already at the beginning of the list, it wraps around to the end of the list.
    pub fn prev_file(&mut self) {
        if !self.files.is_empty() {
            self.selected_file = if self.selected_file > 0 {
                self.selected_file - 1
            } else {
                self.files.len() - 1
            };
        }
    }
    /// Resizes the dimensions of the current object.
    ///
    /// # Parameters
    /// - `width`: The new width to set for the object.
    /// - `height`: The new height to set for the object.
    ///
    /// # Notes
    /// This function updates the internal `width` and `height` fields of the object.
    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
    }
    /// Returns a vector containing the list of directories.
    ///
    /// # Description
    /// This method retrieves the directories stored within the instance by
    /// creating and returning a `Vec<String>` that represents the directories.
    ///
    /// # Returns
    /// A `Vec<String>` containing the directories that have been stored.
    pub fn get_directories(&self) -> Vec<String> {
        self.directories.to_vec()
    }
    /// Retrieves a list of files associated with the current instance.
    ///
    /// # Returns
    /// A `Vec<String>` containing the names of the files. This method clones
    /// the internal `files` attribute and returns its contents as a vector.
    ///
    /// # Note
    /// This function performs a cloning operation on the `files` attribute
    /// of the struct. If you need to avoid cloning for performance reasons,
    /// consider providing a reference to the files instead.
    pub fn get_files(&self) -> Vec<String> {
        self.files.to_vec()
    }

    /// Moves the selection to the next directory in the list.
    pub fn next_dir(&mut self) {
        if !self.directories.is_empty() {
            self.selected_dir = (self.selected_dir + 1) % self.directories.len();
        }
    }

    /// Moves the selection to the previous directory in the list.
    pub fn prev_dir(&mut self) {
        if !self.directories.is_empty() {
            self.selected_dir = if self.selected_dir > 0 {
                self.selected_dir - 1
            } else {
                self.directories.len() - 1
            };
        }
    }
    /// Filters and updates the file and directory listings based on the given search query.
    ///
    /// # Parameters
    /// - `&mut self`: The mutable reference to the current struct instance.
    /// - `research: String`: The search query string to filter files and directories. The query
    ///   supports modifiers for more specific filtering:
    ///   - `=`: Matches exact names.
    ///   - `^`: Matches names starting with the provided value.
    ///   - `$`: Matches names ending with the provided value.
    ///   - `!`: Excludes names containing the provided value.
    ///   - `*`: (Default) Matches names containing the provided value.
    ///
    ///   Alternatively, if the search query starts with `?`, a "deep search" mode is triggered
    ///   where further query modifiers can be applied to refine the results further.
    ///
    /// # Returns
    /// - `(Vec<String>, Vec<String>)`: A tuple where:
    ///   - The first element is a vector of filtered directory names.
    ///   - The second element is a vector of filtered file names.
    ///
    /// # Behavior
    /// - Resets the `selected_file` and `selected_dir` indices to `0`.
    /// - Converts the search query to lowercase for case-insensitive filtering.
    /// - Depending on the search query:
    ///   - If the query starts with `?`, deep search logic is applied:
    ///     - The query is split into multiple sub-queries using whitespace, with the first query acting
    ///       as the base search.
    ///     - Caches results from prior searches to minimize unnecessary disk scans.
    ///     - Applies additional modifiers (if any) to refine the cached results.
    ///   - If the query does not start with `?`, it performs basic filtering on existing `base_files`
    ///     and `base_directories`, updating current directories and files accordingly using the provided
    ///     query modifiers.
    /// - Updates the following fields based on filtering:
    ///   - `self.files`: List of matching files.
    ///   - `self.directories`: List of matching directories.
    ///   - `self.sub_directories` and `self.sub_files`: Sub-items filtered based on the provided query.
    /// - Clears `self.deep_search_cache` if deep search is not used.
    ///
    /// # Examples
    ///
    /// ## Case 1: Basic search without modifiers
    /// ```rust,ignore
    /// let query = "example".to_string();
    /// let (dirs, files) = instance.filter(query);
    /// ```
    /// Matches all files and directories containing the substring `"example"`.
    ///
    /// ## Case 2: Search with modifiers
    /// ```rust,ignore
    /// let query = "^start".to_string();
    /// let (dirs, files) = instance.filter(query);
    /// ```
    /// Matches all files and directories starting with `"start"`.
    ///
    /// ## Case 3: Deep search with query refinement
    /// ```rust,ignore
    /// let query = "?primary ^subquery".to_string();
    /// let (dirs, files) = instance.filter(query);
    /// ```
    /// Performs a deep search with `"primary"` as the base query and refines the results with
    /// the additional modifier `^subquery`.
    ///
    /// # Notes
    /// - Deep search uses recursion (`deep_search_recursive`) and a cache for optimization.
    /// - Uses case-insensitive string matching for all queries and refinements.
    ///
    /// # Caveats
    /// - The method assumes `self.base_files`, `self.base_directories`, `self.base_sub_directories`,
    ///   and `self.base_sub_files` are populated before invocation and accessible for filtering.
    /// - Potential performance impact if disk scans via `deep_search_recursive` are required for
    ///   extensive searches in large directories.
    pub fn filter(&mut self, research: String) -> (Vec<String>, Vec<String>) {
        self.selected_file = 0;
        self.selected_dir = 0;
        let research_lower = research.to_ascii_lowercase();

        if let Some(deep_query) = research_lower.strip_prefix('?') {
            let queries: Vec<&str> = deep_query.split_whitespace().collect();

            if queries.is_empty() {
                self.files.clear();
                self.deep_search_cache = None;
            } else {
                let primary_query = queries[0];

                let mut needs_disk_scan = true;
                if let Some((cached_query, _)) = &self.deep_search_cache
                    && primary_query.starts_with(cached_query)
                {
                    needs_disk_scan = false;
                }

                if needs_disk_scan {
                    let mut new_results = Vec::new();
                    deep_search_recursive(primary_query, &mut new_results);
                    self.deep_search_cache = Some((primary_query.to_string(), new_results));
                }

                let mut current_results = self.deep_search_cache.as_ref().unwrap().1.clone();

                let apply_modifier = |results: &mut Vec<String>, query: &str| {
                    let (modifier, target) = if let Some(t) = query.strip_prefix('=') {
                        ('=', t)
                    } else if let Some(t) = query.strip_prefix('^') {
                        ('^', t)
                    } else if let Some(t) = query.strip_prefix('$') {
                        ('$', t)
                    } else if let Some(t) = query.strip_prefix('!') {
                        ('!', t)
                    } else {
                        ('*', query)
                    };

                    results.retain(|name| {
                        let name_lower = name.to_lowercase();
                        match modifier {
                            '=' => name_lower == target,
                            '^' => name_lower.starts_with(target),
                            '$' => name_lower.ends_with(target),
                            '!' => !name_lower.contains(target),
                            _ => name_lower.contains(target),
                        }
                    });
                };

                if primary_query != self.deep_search_cache.as_ref().unwrap().0 {
                    apply_modifier(&mut current_results, primary_query);
                }

                for q in queries.iter().skip(1) {
                    apply_modifier(&mut current_results, q);
                }
                self.files = current_results;
            }

            self.directories.clear();
            self.sub_directories.clear();
            self.sub_files.clear();
            (self.get_directories(), self.get_files())
        } else {
            self.deep_search_cache = None;

            let matcher = |item_name: &String| -> bool {
                let item_lower = item_name.to_lowercase();
                if let Some(target) = research_lower.strip_prefix('=') {
                    item_lower == target
                } else if let Some(target) = research_lower.strip_prefix('^') {
                    item_lower.starts_with(target)
                } else if let Some(target) = research_lower.strip_prefix('$') {
                    item_lower.ends_with(target)
                } else if let Some(target) = research_lower.strip_prefix('!') {
                    !item_lower.contains(target)
                } else {
                    item_lower.contains(&research_lower)
                }
            };

            self.files = self
                .base_files
                .iter()
                .filter(|f| matcher(f))
                .cloned()
                .collect();
            self.directories = self
                .base_directories
                .iter()
                .filter(|d| matcher(d))
                .cloned()
                .collect();
            self.sub_directories = self
                .base_sub_directories
                .iter()
                .filter(|d| matcher(d))
                .cloned()
                .collect();

            self.sub_files = self
                .base_sub_files
                .iter()
                .filter(|d| matcher(d))
                .cloned()
                .collect();
            (self.get_directories(), self.get_files())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mosaic_finder_drawing_standard() {
        let finder = Finder::new(Path::new("."), FinderLayout::Mosaic);
        let mut buffer = Vec::new();
        let result = finder
            .layout
            .draw(&finder, &mut buffer, "query", 0, 0, 80, 24);
        assert!(result.is_ok());
        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("query"));
        assert!(output.contains("DIRECTORIES"));
        assert!(output.contains("SUB DIRECTORIES"));
        assert!(output.contains("FILES"));
        assert!(output.contains("SUB FILES"));
        assert!(output.contains("INFO"));
        assert!(output.contains("┌"));
        assert!(output.contains("┬"));
        assert!(output.contains("┐"));
        assert!(output.contains("├"));
        assert!(output.contains("┼"));
        assert!(output.contains("┤"));
        assert!(output.contains("└"));
        assert!(output.contains("┴"));
        assert!(output.contains("┘"));
    }

    #[test]
    fn test_mosaic_finder_drawing_compact() {
        let finder = Finder::new(Path::new("."), FinderLayout::Mosaic);
        let mut buffer = Vec::new();
        let result = finder
            .layout
            .draw(&finder, &mut buffer, "compact", 0, 0, 60, 10);
        assert!(result.is_ok());
        let output = String::from_utf8_lossy(&buffer);
        assert!(output.contains("compact"));
        assert!(output.contains("DIRECTORIES"));
    }

    #[test]
    fn test_mosaic_finder_drawing_tiny_bounds() {
        let finder = Finder::new(Path::new("."), FinderLayout::Mosaic);
        let mut buffer = Vec::new();
        let result = finder.layout.draw(&finder, &mut buffer, "tiny", 0, 0, 5, 3);
        assert!(result.is_ok());
    }

    #[test]
    fn test_mosaic_layout_transitions() {
        assert_eq!(FinderLayout::Commander.next(), FinderLayout::Mosaic);
        assert_eq!(FinderLayout::Mosaic.next(), FinderLayout::Grid);
        assert_eq!(FinderLayout::Mosaic.previous(), FinderLayout::Commander);
        assert_eq!(FinderLayout::Grid.previous(), FinderLayout::Mosaic);
    }
}
