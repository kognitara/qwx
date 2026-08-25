use crossterm::terminal::{Clear, ClearType};
use crossterm::{
    cursor::MoveTo,
    execute, queue,
    style::{Color, Print, SetForegroundColor},
};
use std::io::{Result, Write};

/// Painter for the QWX terminal emulator
pub struct QwxPainter<'a, W: Write> {
    writer: &'a mut W,
    height: u16,
    width: u16,
}

/// A constant array representing border characters used for drawing box-like structures.
///
/// # Elements
/// - `BORDERS[0]`: `'─'` - Represents a horizontal border character.
/// - `BORDERS[1]`: `'│'` - Represents a vertical border character.
/// - `BORDERS[2]`: `'┌'` - Represents the top-left corner character.
/// - `BORDERS[3]`: `'└'` - Represents the bottom-left corner character.
///
/// # Usage
/// The `BORDERS` constant can be used when constructing text-based
/// visual elements, such as boxes, tables, or other rectangular structures.
///
/// # Example
/// ```rust
/// use qwx::painter::BORDERS;
/// println!("Top corner: {}", BORDERS[2]);
/// println!("Horizontal line: {}", BORDERS[0]);
/// ```
pub const BORDERS: [char; 4] = ['─', '│', '┌', '└'];

/// A constant array defining characters used for creating rounded borders in text-based UI elements.
///
/// - `ROUNDED_BORDERS[0]` ('─'): Represents the horizontal line used for the top and bottom borders.
/// - `ROUNDED_BORDERS[1]` ('│'): Represents the vertical line used for the left and right borders.
/// - `ROUNDED_BORDERS[2]` ('╭'): Represents the top-left corner of the border.
/// - `ROUNDED_BORDERS[3]` ('╰'): Represents the bottom-left corner of the border.
///
/// # Example
/// ```rust
/// use qwx::painter::ROUNDED_BORDERS;
///
/// println!("{}", ROUNDED_BORDERS[2]); // Prints: ╭
/// ```
///
/// These characters are commonly used when constructing visually appealing
/// text-based layouts, such as in terminal-based user interfaces.
pub const ROUNDED_BORDERS: [char; 4] = ['─', '│', '╭', '╰'];

/// A constant array representing characters used for drawing double borders in a terminal UI.
///
/// The array contains the following characters:
/// - `'═'`: The horizontal double line character.
/// - `'║'`: The vertical double line character.
/// - `'╔'`: The top-left corner double line character.
/// - `'╚'`: The bottom-left corner double line character.
///
/// This array can be used when constructing UI components, such as tables or windows,
/// that require double border styling.
pub const DOUBLE_BORDER: [char; 4] = ['═', '║', '╔', '╚'];
/// A constant representing a double horizontal line character ('═').
///
/// This can be used in applications for creating UI borders, tables,
/// or other visual elements that require a double horizontal line
/// style.
///
/// # Example
/// ```
/// use qwx::painter::DOUBLE_LINE;
///
/// println!("{}", DOUBLE_LINE); // Output: ═
/// ```
///
/// # Unicode Information
/// - Unicode: U+2550
/// - Name: Box Drawings Double Horizontal
pub const DOUBLE_LINE: char = '═';
/// A constant character representing a horizontal line, often used for
/// drawing or formatting purposes in text-based UI or console output.
///
/// # Example
/// ```
/// use qwx::painter::LINE;
/// println!("{}", LINE); // Output: '─'
/// ```
///
/// This character is commonly used in applications that require
/// horizontal dividers or borders in their output.
pub const LINE: char = '─';

impl<'a, W: Write> QwxPainter<'a, W> {
    /// Creates a new instance of the struct.
    ///
    /// # Parameters
    /// - `writer`: A mutable reference to a writer of generic type `W`.
    /// - `height`: The height parameter, represented as a 16-bit unsigned integer.
    /// - `width`: The width parameter, represented as a 16-bit unsigned integer.
    ///
    /// # Returns
    /// A new instance of the struct initialized with the provided writer, height, and width values.
    pub fn new(writer: &'a mut W, height: u16, width: u16) -> Self {
        Self {
            writer,
            height,
            width,
        }
    }

    pub fn text(&mut self, text: String) -> Result<&mut Self> {
        queue!(self.writer, Print(text))?;
        Ok(self)
    }

    /// Sets the height of the object.
    ///
    /// # Arguments
    /// * `height` - A `u16` value representing the new height to set.
    ///
    /// # Returns
    /// Get a mutable reference to `self` to allow for method chaining.
    pub fn set_height(&mut self, height: u16) -> &mut Self {
        self.height = height;
        self
    }
    /// Sets the width of the current object.
    ///
    /// # Parameters
    /// - `width`: The new width value to set, represented as an unsigned 16-bit integer.
    ///
    /// # Returns
    /// A mutable reference to the current instance, allowing for method chaining.
    pub fn set_width(&mut self, width: u16) -> &mut Self {
        self.width = width;
        self
    }
    /// Sets the foreground color for the terminal output.
    ///
    /// This function changes the color of the text that will be written to the terminal.
    /// It uses the `queue!` macro to queue the color change command for the writer.
    ///
    /// # Arguments
    /// * `color` - The `Color` to set as the foreground color for the terminal text.
    ///
    /// # Returns
    /// * `Result<&mut Self>` - Returns a mutable reference to `self` on success, or an error if the
    ///   operation fails.
    ///
    /// # Errors
    /// This function will return an error if queuing the `SetForegroundColor` command fails.
    ///
    pub fn set_color(&mut self, color: Color) -> Result<&mut Self> {
        queue!(self.writer, SetForegroundColor(color))?;
        Ok(self)
    }

    /// Draws a horizontal line starting at a specified position on the terminal.
    ///
    /// This function renders a horizontal line composed of the `─` character, starting
    /// at the specified `(x, y)` position and extending for the specified `length`.
    ///
    /// # Parameters
    /// - `x`: The horizontal starting position (column index) of the line.
    /// - `y`: The vertical starting position (row index) of the line.
    /// - `length`: The number of `─` characters to draw. Passing a `length` of `0`
    ///   will result in no output.
    ///
    /// # Returns
    /// - `Ok(&mut Self)`: Returns a mutable reference to the current instance on
    ///   success, allowing method chaining.
    /// - `Err(std::io::Error)`: Returns an error if any I/O operation fails while
    ///   attempting to render the line.
    ///
    /// # Behavior
    /// - If the `length` parameter is `0`, the method returns immediately without
    ///   performing any rendering.
    /// - The terminal cursor is moved to the specified `(x, y)` position before
    ///   rendering the line, and the `length` characters are printed consecutively.
    ///
    pub fn horizontal_line(&mut self, x: u16, y: u16, length: u16) -> Result<&mut Self> {
        if length == 0 {
            return Ok(self);
        }
        let line = "─".repeat(length as usize);
        queue!(self.writer, MoveTo(x, y), Print(line))?;
        Ok(self)
    }

    /// Draws a vertical line on the terminal at a specified starting position.
    ///
    /// # Arguments
    ///
    /// * `x` - The horizontal position (column) where the vertical line starts.
    /// * `y` - The vertical position (row) where the vertical line starts.
    /// * `length` - The number of characters (rows) the vertical line should span.
    ///
    /// # Returns
    ///
    /// Returns `Ok(&mut Self)` if the vertical line is successfully drawn, allowing for method chaining.
    /// Returns an error if there is a problem queuing the commands to the terminal writer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the terminal queue operations (such as `MoveTo` or `Print`) fail.
    ///
    /// The above example will draw a vertical line starting at column 5, row 2, and spanning 10 rows downward.
    ///
    /// # Notes
    ///
    /// This function assumes the use of `crossterm` for terminal manipulation. It queues terminal commands
    /// to move the cursor and print the vertical line character (`│`). Make sure to properly flush the terminal
    /// writer to render the changes.
    pub fn vertical_line(&mut self, x: u16, y: u16, length: u16) -> Result<&mut Self> {
        for i in 0..length {
            queue!(self.writer, MoveTo(x, y + i), Print("│"))?;
        }
        Ok(self)
    }

    /// Draws a rectangular border using box-drawing characters at the specified position and size.
    ///
    /// # Parameters
    /// - `x`: The x-coordinate of the top-left corner of the rectangle.
    /// - `y`: The y-coordinate of the top-left corner of the rectangle.
    /// - `width`: The width of the rectangle. Must be at least 2 to form a valid border.
    /// - `height`: The height of the rectangle. Must be at least 2 to form a valid border.
    ///
    /// # Returns
    /// - `Result<&mut Self>`: Returns a mutable reference to `Self` wrapped in a `Result`.
    ///   If the width or height is less than 2, it exits early and returns `self` without modifying the writer.
    ///
    /// # Description
    /// - The function first validates that the rectangle has a minimum width and height of 2.
    /// - It constructs the rectangle using Unicode box-drawing characters:
    ///   - `┌` and `┐` for the top corners.
    ///   - `└` and `┘` for the bottom corners.
    ///   - `─` for the horizontal edges.
    ///   - `│` for the vertical edges.
    /// - It queues rendering commands to draw:
    ///   1. The top border row.
    ///   2. The bottom border row.
    ///   3. The vertical edges for all rows in between.
    /// - The drawing is done using the `queue!` macro to enqueue terminal commands for efficient batch rendering.
    ///
    /// # Errors
    /// - If any terminal writing operation fails, the function returns an error wrapped in `Result`.
    ///
    /// # Notes
    /// - This function requires that the user has a valid `writer` object capable of handling terminal commands.
    /// - The rectangle will not render properly if the coordinates or dimensions exceed terminal bounds.
    ///
    /// # Constraints
    /// - `width` and `height` must both be greater than or equal to 2 to draw a valid rectangle.
    pub fn rect(
        &mut self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        border_type: &[&str],
    ) -> Result<&mut Self> {
        if width < 2 || height < 2 {
            return Ok(self);
        }

        let inner_width = width - 2;
        let h_line = "─".repeat(inner_width as usize);

        queue!(
            self.writer,
            MoveTo(x, y),
            Print(format!("{}{}{}", border_type[0], h_line, border_type[1]))
        )?;

        queue!(
            self.writer,
            MoveTo(x, y + height - 1),
            Print(format!("{}{}{}", border_type[2], h_line, border_type[3]))
        )?;

        for i in 1..(height - 1) {
            queue!(self.writer, MoveTo(x, y + i), Print(border_type[4]))?;
            queue!(
                self.writer,
                MoveTo(x + width - 1, y + i),
                Print(border_type[4])
            )?;
        }

        Ok(self)
    }

    /// Moves the cursor to the specified position (`x`, `y`) in the terminal.
    ///
    /// # Arguments
    ///
    /// * `x` - The horizontal position (column) to move the cursor to. Starts at 0.
    /// * `y` - The vertical position (row) to move the cursor to. Starts at 0.
    ///
    /// # Returns
    ///
    /// * `Result<&mut Self>` - Returns a mutable reference to `Self` wrapped in a `Result`,
    ///   or an error if the underlying operation fails.
    ///
    /// # Errors
    ///
    /// This function will return an error if the `execute!` macro fails during the
    /// execution of the `MoveTo` action.
    ///
    /// The example above moves the cursor to position `(10, 5)` in the terminal.
    pub fn to(&mut self, x: u16, y: u16) -> Result<&mut Self> {
        execute!(self.writer, MoveTo(x, y))?;
        Ok(self)
    }

    /// Moves the cursor to the specified coordinates `(x, y)` in the terminal
    /// and prints the given `symbol` at that location.
    ///
    /// This function uses the `crossterm` crate to manage terminal manipulations.
    /// It queues a `MoveTo` command to position the cursor and a `Print` command
    /// to print the symbol at the specified coordinates. The function returns
    /// a mutable reference to `Self` on success.
    ///
    /// # Parameters
    /// - `x`: The horizontal position (column) to move the cursor to, as a `u16`.
    /// - `y`: The vertical position (row) to move the cursor to, as a `u16`.
    /// - `symbol`: A string slice representing the symbol to print at the specified
    ///   coordinates.
    ///
    /// # Returns
    /// - `Ok(&mut Self)`: The mutable reference to the current instance if the
    ///   operation is successful.
    /// - `Err(...)`: Error if there is an issue queuing the commands or performing
    ///   the terminal operations.
    ///
    /// # Errors
    /// Returns an error if the `queue!` macro, which processes the terminal
    /// commands, fails to execute.
    ///
    pub fn intersection(&mut self, x: u16, y: u16, symbol: &str) -> Result<&mut Self> {
        queue!(self.writer, MoveTo(x, y), Print(symbol))?;
        Ok(self)
    }

    /// Flushes the underlying writer to ensure that all buffered data is written.
    ///
    /// This method forces the writer to flush its buffer, ensuring all pending
    /// writes are sent to the underlying output. It is particularly useful to
    /// guarantee that data has been fully written when performing buffered I/O
    /// operations.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: If the flush operation completes successfully.
    /// - `Err(e)`: If an error occurs while flushing the writer.
    ///
    /// # Errors
    /// This function propagates any I/O errors that occur while flushing the
    /// underlying writer.
    ///
    /// # Notes
    /// It's a good practice to call this method when you're finished writing
    /// to ensure all data is written, especially for buffered writers.
    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()
    }
    /// Clears the terminal screen or a specified section of it based on the given mode.
    ///
    /// # Parameters
    /// - `mode`: The `ClearType` enum variant that specifies the area to be cleared.
    ///   It could represent clearing the entire screen, from the cursor to the end of the screen, etc.
    ///
    /// # Returns
    /// Get a mutable reference to `Self` wrapped in a `Result`. This allows for chaining
    /// method calls on the same object if the operation succeeds.
    ///
    /// # Errors
    /// If queuing the clearing command fails, an error is returned.

    ///
    /// This call clears the entire screen and returns a mutable reference to the terminal
    /// instance for further operations.
    ///
    /// # Dependencies
    /// This function relies on the `queue!` macro to send the `Clear` command to the
    /// terminal via the writer object.
    pub fn clear(&mut self, mode: ClearType) -> Result<&mut Self> {
        queue!(self.writer, Clear(mode))?;
        Ok(self)
    }
    /// Clears the entire terminal screen.
    ///
    /// This function sends a command to clear all content from the terminal screen,
    /// using the `ClearType::All` variant to specify clearing the entire display.
    ///
    /// # Returns
    ///
    /// Returns a mutable reference to `self` if the operation is successful.
    ///
    /// # Errors
    ///
    /// If writing to the terminal fails, this function will return an error.
    ///
    /// # Dependencies
    ///
    /// This function relies on the `queue!` macro to send commands to the terminal
    ///  writer and on the `Clear` enum provided by the relevant terminal backend.
    ///
    /// # Notes
    ///
    /// Ensure that `self.writer` is properly initialized and configured to interact
    /// with the terminal before invoking this function, or it may fail.
    pub fn clear_screen(&mut self) -> Result<&mut Self> {
        queue!(self.writer, Clear(ClearType::All))?;
        Ok(self)
    }
}
