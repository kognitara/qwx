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
        .long_about(HELP_CONTENT)
        .subcommand(
            Command::new("open").arg(
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
            App::new(Path::new(p.as_str()))?.run()
        }
        _ => {
            app.clone().print_long_help()?;
            Ok(())
        }
    }
}
