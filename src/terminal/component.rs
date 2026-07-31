use crate::terminal::echo::Echo;
use std::io::{Result, Write};

pub trait Component {
    fn render<W: Write>(
        &self,
        w: &mut W,
        echo: &Echo,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) -> Result<()>;
}
