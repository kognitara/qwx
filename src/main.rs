use crate::apps::App;
use std::{io::Result, path::Path};

mod apps;
mod editor;
mod finder;

fn main() -> Result<()> {
    App::new(Path::new("."))?.run()
}
