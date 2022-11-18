#![warn(missing_docs)]

//! Zentime cli

use crate::start_timer::start_timer;
use clap::{Parser, Subcommand};

pub mod config;
mod input;
mod notification;
mod sound;
mod start_timer;
mod subcommands;
mod terminal_event;
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

        None => start_timer(&cli.config),
    }
}
