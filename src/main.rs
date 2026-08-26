use clap::{Arg, ArgAction, ArgMatches, Command, value_parser};
use clap_complete::{Shell, generate};
use git2::build::RepoBuilder;
use git2::{FetchOptions, RemoteCallbacks};
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Text;
use qwx::editor::{Mode, Qwx};
use qwx::player::{SpotifyClient, SpotifyCredentials};
use std::env::{current_dir, set_current_dir};
use std::io;
use std::path::Path;

const HELP_CONTENT: &str = include_str!("../help.txt");

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .version(env!("CARGO_PKG_VERSION"))
        .long_about(HELP_CONTENT)
        .subcommand(Command::new("open").about("Open a directory or file"))
        .subcommand(
            Command::new("gen")
                .about("Gen auto completion for shell")
                .arg(
                    Arg::new("shell")
                        .help("Generate the auto completion script")
                        .value_parser(["bash", "zsh", "fish", "powershell", "elvish"]),
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
        .subcommand(
            Command::new("spotify")
                .visible_alias("spotify-config")
                .about("Configure Spotify credentials interactively or update access token")
                .subcommand(
                    Command::new("update-token")
                        .visible_alias("update-token")
                        .about("Automatically request and update Spotify access token using Client ID & Secret"),
                ),
        )
}

fn configure_spotify() -> io::Result<()> {
    let existing = SpotifyCredentials::load_from_config();

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║              QWX - Spotify Web API Configuration               ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!("Press Enter to keep the current value or leave empty.\n");

    let mut client_id_prompt = Text::new("Spotify Client ID:")
        .with_help_message("Client ID from the Spotify Developer Dashboard");
    if let Some(ref val) = existing.client_id {
        client_id_prompt = client_id_prompt.with_default(val);
    }
    let client_id = client_id_prompt
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut client_secret_prompt = Text::new("Spotify Client Secret:")
        .with_help_message("Client Secret from the Spotify Developer Dashboard");
    if let Some(ref val) = existing.client_secret {
        client_secret_prompt = client_secret_prompt.with_default(val);
    }
    let client_secret = client_secret_prompt
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut access_token_prompt = Text::new("Spotify Access Token (Bearer):")
        .with_help_message("OAuth Bearer access token (optional)");
    if let Some(ref val) = existing.access_token {
        access_token_prompt = access_token_prompt.with_default(val);
    }
    let access_token = access_token_prompt
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let mut refresh_token_prompt =
        Text::new("Spotify Refresh Token:").with_help_message("OAuth refresh token (optional)");
    if let Some(ref val) = existing.refresh_token {
        refresh_token_prompt = refresh_token_prompt.with_default(val);
    }
    let refresh_token = refresh_token_prompt
        .prompt()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    let to_option = |s: String| -> Option<String> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    };

    let raw_json = serde_json::json!({
        "client_id": to_option(client_id),
        "client_secret": to_option(client_secret),
        "access_token": to_option(access_token),
        "refresh_token": to_option(refresh_token),
    });

    let credentials: SpotifyCredentials = serde_json::from_value(raw_json).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Deserialization error: {e}"),
        )
    })?;

    credentials.save_to_config()?;

    if let Some(path) = SpotifyCredentials::config_file_path() {
        println!(
            "\n✓ Spotify configuration successfully saved to {}",
            path.display()
        );
    } else {
        println!("\n✓ Spotify configuration successfully saved.");
    }

    Ok(())
}

fn update_spotify_token() -> io::Result<()> {
    println!("Requesting new Spotify Access Token using configured Client ID & Secret...");
    let mut client = SpotifyClient::new();
    match client.request_client_credentials_token() {
        Ok(token) => {
            println!("\n✓ Spotify Access Token successfully retrieved and saved to configuration!");
            let preview = if token.len() > 16 {
                format!("{}...", &token[..16])
            } else {
                token
            };
            println!("  Token: {}", preview);
            if let Some(path) = SpotifyCredentials::config_file_path() {
                println!("  Saved to: {}", path.display());
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("\n✗ Failed to update Spotify token: {}", e);
            eprintln!(
                "  Tip: Make sure Client ID & Client Secret are configured via `qwx spotify` or environment variables."
            );
            Err(io::Error::new(io::ErrorKind::Other, e))
        }
    }
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
        return Qwx::new(p.as_path(), Mode::Normal)?.run();
    }

    let pb = ProgressBar::new(0);
    pb.set_style(
        ProgressStyle::default_bar()
            // {pos}/{len} will count objects or deltas depending on the phase
            .template(
                "{spinner:.white} [{elapsed_precise}] [{bar:40.white}] {pos:>7}/{len:7} {msg}",
            )
            .unwrap()
            .progress_chars("██  "),
    );

    let mut callbacks = RemoteCallbacks::new();
    let pb_clone = pb.clone();

    callbacks.transfer_progress(move |stats| {
        if stats.received_objects() < stats.total_objects() {
            // PHASE 1: Receiving from server
            pb_clone.set_length(stats.total_objects() as u64);
            pb_clone.set_position(stats.received_objects() as u64);

            // Display downloaded MiB in message text
            let mb_received = stats.received_bytes() as f64 / 1_048_576.0;
            pb_clone.set_message(format!("Receiving objects... ({:.2} MiB)", mb_received));
        } else {
            // PHASE 2: Resolving deltas locally
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

    if let Err(e) = builder.clone(url, dest) {
        pb.abandon_with_message(format!("Clone failed: {e}"));
        return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
    }
    pb.finish_with_message("Clone and checkout completed.");
    set_current_dir(p.as_path())?;
    Qwx::new(p.as_path(), Mode::Normal)?.run()
}
fn main() -> io::Result<()> {
    let mut app = cli();
    let matches = app.clone().get_matches();
    match matches.subcommand() {
        Some(("open", sub)) => {
            let p = sub
                .get_one::<String>("path")
                .map(|s| s.as_str())
                .unwrap_or(".");
            let path = Path::new(p);
            Qwx::new(path, Mode::Normal)?.run()
        }
        Some(("gen", sub)) => {
            let shell = sub.get_one::<String>("shell").expect("shell is required");
            let shell = match shell.as_str() {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                "powershell" => Shell::PowerShell,
                "elvish" => Shell::Elvish,
                _ => unreachable!(),
            };
            generate(shell, &mut app, "qwx", &mut io::stdout());
            Ok(())
        }
        Some(("clone", sub)) => clone_and_open(sub),
        Some(("spotify", sub)) | Some(("spotify-config", sub)) => {
            if let Some(("update-token", _)) = sub.subcommand() {
                update_spotify_token()
            } else {
                configure_spotify()
            }
        }
        None => {
            if let Some(p) = matches.get_one::<String>("path") {
                let path = Path::new(p);
                Qwx::new(path, Mode::Normal)?.run()
            } else {
                Qwx::new(Path::new("."), Mode::Normal)?.run()
            }
        }
        _ => {
            app.clone().print_help()?;
            Ok(())
        }
    }
}
