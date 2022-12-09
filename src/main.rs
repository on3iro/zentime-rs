use crate::default_cmd::default_cmd;
use clap::{Parser, Subcommand};
use env_logger::Env;

mod default_cmd;
mod subcommands;
use subcommands::server::{start_daemonized, status, stop};

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

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .target(env_logger::Target::Stdout)
        .init();
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Server { command }) => match command {
            ServerCommands::Start { config } => start_daemonized(config),
            ServerCommands::Stop => stop(),
            ServerCommands::Status => status(),
        },

        None => default_cmd(&cli.config),
    }
}
