use crate::terminal::core::QwxEvent;
use crate::terminal::echo::Echo;
use std::io::{Result, Write};

pub enum EventResult {
    Consumed,
    Ignored,
}

pub trait Component {
    fn on_mount(&mut self) {}

    fn on_event(&mut self, _event: &QwxEvent) -> EventResult {
        EventResult::Ignored
    }

    // Le changement magique est ici : on remplace <W: Write>(w: &mut W)
    // par directement w: &mut dyn Write
    fn render(
        &self,
        w: &mut dyn Write,
        echo: &Echo,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) -> Result<()>;
}
