use crossterm::{
    cursor::MoveTo,
    queue,
    style::{Color, Print, SetForegroundColor},
};
use std::io::{Result, Write};

pub struct QwxPainter<'a, W: Write> {
    writer: &'a mut W,
}

impl<'a, W: Write> QwxPainter<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }

    /// Change la couleur du pinceau
    pub fn set_color(&mut self, color: Color) -> Result<&mut Self> {
        queue!(self.writer, SetForegroundColor(color))?;
        Ok(self)
    }

    /// Trace une ligne horizontale
    pub fn hline(&mut self, x: u16, y: u16, length: u16) -> Result<&mut Self> {
        if length == 0 {
            return Ok(self);
        }
        let line = "─".repeat(length as usize);
        queue!(self.writer, MoveTo(x, y), Print(line))?;
        Ok(self)
    }

    /// Trace une ligne verticale
    pub fn vline(&mut self, x: u16, y: u16, length: u16) -> Result<&mut Self> {
        for i in 0..length {
            queue!(self.writer, MoveTo(x, y + i), Print("│"))?;
        }
        Ok(self)
    }

    /// Dessine un carré ou rectangle complet (bordures uniquement)
    pub fn rect(&mut self, x: u16, y: u16, width: u16, height: u16) -> Result<&mut Self> {
        if width < 2 || height < 2 {
            return Ok(self);
        }

        let inner_width = width - 2;
        let h_line = "─".repeat(inner_width as usize);

        // Bordure haute
        queue!(self.writer, MoveTo(x, y), Print(format!("┌{}┐", h_line)))?;

        // Bordure basse
        queue!(
            self.writer,
            MoveTo(x, y + height - 1),
            Print(format!("└{}┘", h_line))
        )?;

        // Bordures latérales
        for i in 1..(height - 1) {
            queue!(self.writer, MoveTo(x, y + i), Print("│"))?;
            queue!(self.writer, MoveTo(x + width - 1, y + i), Print("│"))?;
        }

        Ok(self)
    }

    /// Place un point d'intersection spécifique (ex: ├, ┤, ┬, ┴, ┼)
    pub fn intersection(&mut self, x: u16, y: u16, symbol: &str) -> Result<&mut Self> {
        queue!(self.writer, MoveTo(x, y), Print(symbol))?;
        Ok(self)
    }

    pub fn flush(&mut self) -> Result<()> {
        self.writer.flush()
    }
}
