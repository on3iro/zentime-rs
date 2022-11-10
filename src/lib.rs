#![warn(missing_docs)]

//! Zentime cli

use clap::{Parser, Subcommand};
pub use events::{AppAction, TerminalEvent, ViewState};
use std::path::PathBuf;
use subcommands::run::run;

mod config;
mod events;
mod input;
mod notification;
mod sound;
mod subcommands;
mod util;
mod view;

pub mod timer;

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
pub enum Commands {}

/// Runs the specified zentime cli command
pub fn run_cli() {
    let cli = Cli::parse();

    match &cli.command {
        // TODO
        Some(_commands) => {}

        None => run(&cli.config),
    }
}
