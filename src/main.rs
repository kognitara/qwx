use crate::apps::App;
use std::{io::Result, path::Path};

mod apps;
mod finder;

fn main() -> Result<()> {
    let mut app = App::new(Path::new("."))?;
    app.run()
}
