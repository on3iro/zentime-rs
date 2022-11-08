use clap::{Parser, Subcommand};
use std::path::PathBuf;
use subcommands::run::run;

mod config;
mod events;
mod input;
mod notification;
mod sound;
mod subcommands;
mod timer;
mod util;
mod view;

/// Starts the timer or attaches to an already running timer
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    detached: bool,

    /// Sets a custom config file
    #[arg(short, long, default_value = "~/.config/zentime/zentime.toml")]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
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
        // TODO
        Some(_Commands) => {}

        None => run(&cli.config),
    }
}
