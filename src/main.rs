use crate::apps::App;
use clap::{Arg, Command, value_parser};
use std::io;
use std::path::Path;

mod apps;
mod editor;
mod finder;
const HELP_CONTENT: &str = include_str!("../help.txt");
fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .long_about(HELP_CONTENT)
        .subcommand(
            Command::new("open").about("Open a directory").arg(
                Arg::new("path")
                    .short('p')
                    .required(false)
                    .value_parser(value_parser!(String))
                    .default_value("."),
            ),
        )
}

fn main() -> io::Result<()> {
    let app = cli();
    let matches = app.clone().get_matches();
    match matches.subcommand() {
        Some(("open", sub)) => {
            let p = sub.get_one::<String>("path").expect("required");
            if Path::new(p.as_str()).is_dir() {
                App::new(Path::new(p.as_str()))?.run()
            } else if dirs::home_dir().ne(&Some(Path::new(".").to_path_buf())) {
                App::new(Path::new("."))?.run()
            } else {
                app.clone().print_long_help()?;
                Ok(())
            }
        }
        _ => {
            app.clone().print_long_help()?;
            Ok(())
        }
    }
}
