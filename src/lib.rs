#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations
)]

//! Zentime cli

use crate::default_cmd::default_cmd;
use clap::{Parser, Subcommand};
use subcommands::server::{start, status, stop};

mod client;
pub mod config;
mod default_cmd;
mod ipc;
mod server;
mod subcommands;

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
enum Commands {
    /// Interact with the zentime server
    Server {
        #[command(subcommand)]
        command: ServerCommands,
    },
}

#[derive(Subcommand)]
enum ServerCommands {
    /// Start the zentime server
    Start {
        /// Sets a custom config file
        #[arg(short, long, default_value = "~/.config/zentime/zentime.toml")]
        config: String,
    },

    /// Stop the zentime server and close all client connections
    Stop,

    /// Check if the zentime server is currently running
    Status,
}

/// Runs the specified zentime cli command
pub fn run_cli() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Server { command }) => match command {
            ServerCommands::Start { config } => start(config),
            ServerCommands::Stop => stop(),
            ServerCommands::Status => status(),
        },

        None => default_cmd(&cli.config),
    }
}
