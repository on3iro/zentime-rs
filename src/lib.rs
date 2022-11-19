#![warn(missing_docs)]

//! Zentime cli

use crate::default_cmd::default_cmd;
use clap::{Parser, Subcommand};

mod client;
pub mod config;
mod default_cmd;
mod server;

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

        None => default_cmd(&cli.config),
    }
}
