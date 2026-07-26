use crate::apps::App;
use std::io::Result;
mod apps;

fn main() -> Result<()> {
    let mut app = App::new()?;
    app.run()
}
