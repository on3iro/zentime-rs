use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None, arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts the timer or attaches to an already running timer
    Run {
        #[arg(short, long)]
        detached: bool,

        /// Sets a custom config file
        #[arg(short, long, value_name = "~/.config/zentime/zentime.toml")]
        config: Option<PathBuf>,
    },

    /// Restarts an already running timer and applies the current configuration.
    /// If no timer exists it simply starts a new timer.
    Restart {
        /// Sets a custom config file
        #[arg(short, long, value_name = "~/.config/zentime/zentime.toml")]
        config: Option<PathBuf>,
    },

    /// Opens the configuration file inside the default terminal editor
    Configure {},
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Run { detached, config }) => {
            if *detached {
                // TODO
                println!("Started timer in detached mode...");
            }

            if config.is_some() {
                // TODO
                println!("Read custom config file...");
            }
        }

        Some(_Commands) => {}

        None => {}
    }
}
