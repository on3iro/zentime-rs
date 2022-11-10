#![warn(missing_docs)]

//! Zentime cli

use clap::{Parser, Subcommand};
use subcommands::run::run;
pub use zentime_rs_timer::events::{AppAction, TerminalEvent, ViewState};

pub mod config;
mod input;
mod subcommands;
mod util;
mod view;

/// Starts the timer or attaches to an already running timer
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, default_value = "~/.config/zentime/zentime.toml")]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

/// Available cli sub commands
#[derive(Subcommand)]
enum Commands {}

/// Runs the specified zentime cli command
pub fn run_cli() {
    let cli = Cli::parse();

    match &cli.command {
        // TODO
        Some(_commands) => {}

        None => run(&cli.config),
    }
}
