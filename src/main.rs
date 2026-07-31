use crate::apps::App;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use git2::Repository;
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
        .subcommand(
            Command::new("get")
                .about("Get a git repository in a directory and open it in qwx")
                .arg(
                    Arg::new("url")
                        .required(true)
                        .action(ArgAction::Set)
                        .value_parser(value_parser!(String)),
                )
                .arg(
                    Arg::new("destination")
                        .required(true)
                        .action(ArgAction::Set)
                        .value_parser(value_parser!(String)),
                ),
        )
}
fn mount_and_open(sub: &ArgMatches) -> io::Result<()> {
    let url = sub.get_one::<String>("url").expect("url is required");
    let destination = sub
        .get_one::<String>("destination")
        .expect("destination is required");
    let dest = Path::new(destination.as_str());
    if Repository::clone(url, dest).is_ok() {
        App::new(dest)?.run()
    } else {
        Ok(())
    }
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
        Some(("mount", sub)) => mount_and_open(sub),
        _ => {
            app.clone().print_long_help()?;
            Ok(())
        }
    }
}
