use crate::terminal::core::QwxEvent;
use crate::terminal::echo::Echo;
use crossterm::terminal::WindowSize;
use std::io::{Result, Write};

pub trait App {
    fn on_event(&mut self, event: &QwxEvent);

    fn render<W: Write>(&self, w: &mut W, echo: &Echo, window: &WindowSize) -> Result<()>;
}
