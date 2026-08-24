use crate::apps::App;
use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use git2::build::RepoBuilder;
use git2::{FetchOptions, RemoteCallbacks};
use indicatif::{ProgressBar, ProgressStyle};
use std::env::{current_dir, set_current_dir};
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
                    .required(false)
                    .value_parser(value_parser!(String))
                    .action(ArgAction::Set)
                    .default_value("."),
            ),
        )
        .subcommand(
            Command::new("clone")
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

fn clone_and_open(sub: &ArgMatches) -> io::Result<()> {
    let url = sub.get_one::<String>("url").expect("url is required");
    let destination = sub
        .get_one::<String>("destination")
        .expect("destination is required");

    let dest = Path::new(destination.as_str());
    let x = current_dir()?;
    let p = x.join(destination);

    if p.is_dir() {
        set_current_dir(p.as_path())?;
        return App::new(p.as_path())?.run();
    }

    // 1. Initialisation avec un template adapté aux "objets" et non aux octets
    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::default_bar()
            // {pos}/{len} comptera les objets ou les deltas, selon la phase
            .template(
                "{spinner:.white} [{elapsed_precise}] [{bar:40.white}] {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("██  "),
    );

    let mut callbacks = RemoteCallbacks::new();
    let pb_clone = pb.clone();

    // 2. Traitement des deux phases du clone Git
    callbacks.transfer_progress(move |stats| {
        if stats.received_objects() < stats.total_objects() {
            // PHASE 1 : Réception depuis le serveur
            pb_clone.set_length(stats.total_objects() as u64);
            pb_clone.set_position(stats.received_objects() as u64);

            // On affiche quand même les Mo téléchargés dans le message texte
            let mb_received = stats.received_bytes() as f64 / 1_048_576.0;
            pb_clone.set_message(format!("Receiving objects... ({:.2} MiB)", mb_received));
        } else {
            // PHASE 2 : Résolution locale des deltas
            pb_clone.set_length(stats.total_deltas() as u64);
            pb_clone.set_position(stats.indexed_deltas() as u64);
            pb_clone.set_message("Resolving deltas...");
        }
        true
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let mut builder = RepoBuilder::new();
    builder.fetch_options(fetch_options);

    pb.set_message(format!("Connecting to {}...", url));

    // 4. Clonage avec notre constructeur personnalisé
    if builder.clone(url, dest).is_ok() {
        pb.finish_with_message("Clone and checkout completed.");
        set_current_dir(p.as_path())?;
        App::new(p.as_path())?.run()
    } else {
        pb.abandon_with_message("Clone failed.");
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
            } else {
                app.clone().print_help()?;
                Ok(())
            }
        }
        Some(("clone", sub)) => clone_and_open(sub),
        _ => {
            app.clone().print_long_help()?;
            Ok(())
        }
    }
}
