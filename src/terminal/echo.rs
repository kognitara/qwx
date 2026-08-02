use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Attribute, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
};
use std::io::{Result, Write};

use crate::terminal::style::{QwxBorders, QwxDirection, QwxStyle};

#[derive(Debug)]
#[doc = "A struct representing the Qwx terminal echo functionality"]
pub struct Echo;

impl Drop for Echo {
    fn drop(&mut self) {}
}
#[allow(clippy::too_many_arguments)]
impl Echo {
    /// Draws a rectangle with specified borders and style directly to the given writer.
    ///
    /// This method is optimized for terminal rendering. It avoids memory allocation
    /// by queuing the rendering instructions directly into the provided output buffer.
    ///
    /// Terminal state (colors and attributes) is automatically cleaned up and reset
    /// after the rectangle is drawn to prevent styling leaks across the UI.
    ///
    /// # Arguments
    ///
    /// * `w` - A mutable reference to a type implementing the `Write` trait (e.g., `stdout` or a `Vec<u8>`).
    /// * `start_x` - The 0-based column index where the rectangle begins.
    /// * `start_y` - The 0-based row index where the rectangle begins.
    /// * `width` - The width of the rectangle in characters.
    /// * `height` - The height of the rectangle in characters.
    /// * `borders` - The `QwxBorders` struct defining the characters used for the rectangle's borders.
    /// * `style` - The `QwxStyle` (foreground, background, and attributes) applied to the rectangle.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the underlying writer fails to queue the terminal instructions (e.g., broken pipe or I/O failure).
    pub fn rect<W: Write>(
        &self,
        w: &mut W,
        start_x: u16,
        start_y: u16,
        width: u16,
        height: u16,
        borders: QwxBorders,
        style: QwxStyle,
    ) -> Result<()> {
        if width < 2 || height < 2 {
            return Ok(());
        }

        if let Some(fg) = style.fg {
            queue!(w, SetForegroundColor(fg))?;
        }
        if let Some(bg) = style.bg {
            queue!(w, SetBackgroundColor(bg))?;
        }
        if let Some(attr) = style.attr {
            queue!(w, SetAttribute(attr))?;
        }

        let inner_width = width - 2;
        let inner_height = height - 2;
        let end_x = start_x + width - 1;
        let end_y = start_y + height - 1;

        self.line(
            w,
            start_x + 1,
            start_y,
            inner_width,
            QwxDirection::Horizontal,
            borders.horizontal,
            style,
        )?;
        // Bordure basse
        self.line(
            w,
            start_x + 1,
            end_y,
            inner_width,
            QwxDirection::Horizontal,
            borders.horizontal,
            style,
        )?;
        // Bordure gauche
        self.line(
            w,
            start_x,
            start_y + 1,
            inner_height,
            QwxDirection::Vertical,
            borders.vertical,
            style,
        )?;

        // Bordure droite
        self.line(
            w,
            end_x,
            start_y + 1,
            inner_height,
            QwxDirection::Vertical,
            borders.vertical,
            style,
        )?;

        if let Some(fg) = style.fg {
            queue!(w, SetForegroundColor(fg))?;
        }
        if let Some(bg) = style.bg {
            queue!(w, SetBackgroundColor(bg))?;
        }
        if let Some(attr) = style.attr {
            queue!(w, SetAttribute(attr))?;
        }

        // 4. Placer les 4 coins
        queue!(
            w,
            MoveTo(start_x, start_y),
            Print(borders.top_left),
            MoveTo(end_x, start_y),
            Print(borders.top_right),
            MoveTo(start_x, end_y),
            Print(borders.bottom_left),
            MoveTo(end_x, end_y),
            Print(borders.bottom_right),
        )?;

        // 5. Nettoyer le style
        queue!(w, ResetColor)?;
        if style.attr.is_some() {
            queue!(w, SetAttribute(Attribute::Reset))?;
        }
        Ok(())
    }
    /// Draws a straight line (either horizontal or vertical) directly to the given writer.
    ///
    /// This method is highly optimized for terminal rendering. It avoids memory allocation
    /// by bypassing string repetition (e.g., `symbol.repeat()`) and instead queues the
    /// rendering instructions directly into the provided output buffer.
    ///
    /// Terminal state (colors and attributes) is automatically cleaned up and reset
    /// after the line is drawn to prevent styling leaks across the UI.
    ///
    /// # Arguments
    ///
    /// * `w` - A mutable reference to a type implementing the `Write` trait (e.g., `stdout` or a `Vec<u8>`).
    /// * `start_x` - The 0-based column index where the line begins.
    /// * `start_y` - The 0-based row index where the line begins.
    /// * `length` - The number of characters the line spans.
    /// * `direction` - The axis of the line (`Direction::Horizontal` or `Direction::Vertical`).
    /// * `symbol` - The string slice used to draw the line (usually a single character like `"─"`).
    /// * `style` - The `Style` (foreground, background, and attributes) applied to the line.
    ///
    /// # Errors
    ///
    /// Returns an `std::io::Error` if the underlying writer fails to queue the
    /// terminal instructions (e.g., broken pipe or I/O failure).
    pub fn line<W: Write>(
        &self,
        w: &mut W,
        start_x: u16,
        start_y: u16,
        length: u16,
        direction: QwxDirection,
        symbol: &str,
        style: QwxStyle,
    ) -> Result<()> {
        // 1. Appliquer le style
        if let Some(fg) = style.fg {
            queue!(w, SetForegroundColor(fg))?;
        }
        if let Some(bg) = style.bg {
            queue!(w, SetBackgroundColor(bg))?;
        }
        if let Some(attr) = style.attr {
            queue!(w, SetAttribute(attr))?;
        }
        match direction {
            QwxDirection::Horizontal => {
                queue!(w, MoveTo(start_x, start_y))?;

                for _ in 0..length {
                    queue!(w, Print(symbol))?;
                }
            }
            QwxDirection::Vertical => {
                for i in 0..length {
                    queue!(w, MoveTo(start_x, start_y + i), Print(symbol))?;
                }
            }
        }
        queue!(w, ResetColor)?;
        if style.attr.is_some() {
            queue!(w, SetAttribute(Attribute::Reset))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*; // Importe tout ce qui est dans le fichier actuel
    use crossterm::style::Color;

    #[test]
    fn test_draw_horizontal_line() {
        // 1. Initialiser le buffer (notre "faux" terminal en mémoire)
        let mut buffer: Vec<u8> = Vec::new();
        let echo = Echo;

        let style = QwxStyle {
            fg: Some(Color::Red),
            bg: None,
            attr: None,
        };

        // 2. Appeler la fonction de rendu en lui passant le buffer
        let result = echo.line(
            &mut buffer,
            0, // start_x
            0, // start_y
            3, // length
            QwxDirection::Horizontal,
            "─",
            style,
        );

        // 3. Vérifier que la fonction n'a pas retourné d'erreur
        assert!(result.is_ok());

        // 4. Convertir les octets du buffer en chaîne de caractères lisible
        let output = String::from_utf8(buffer).expect("Le buffer ne contient pas d'UTF-8 valide");

        // 5. Assertions : On vérifie que la sortie contient bien nos 3 caractères
        // Note: l'output contiendra aussi les séquences ANSI générées par crossterm
        // (comme les codes de couleur ou les déplacements de curseur MoveTo)
        assert!(output.contains("───"));
    }
}
