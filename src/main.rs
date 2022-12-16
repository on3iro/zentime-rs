use crate::default_cmd::default_cmd;
use clap::{Parser, Subcommand};
use env_logger::Env;

mod default_cmd;
mod subcommands;
use subcommands::server::{start_daemonized, status, stop};

#[derive(clap::Args)]
pub struct CommonArgs {
    /// Sets a custom config file
    #[arg(short, long, default_value = "~/.config/zentime/zentime.toml")]
    config: String,

    #[command(flatten)]
    server_config: ServerConfig,
}

#[derive(clap::Args, Clone, Debug)]
pub struct ServerConfig {
    /// Enable/Disable bell
    #[arg(long)]
    pub enable_bell: Option<bool>,

    /// Notification bell volume
    #[arg(long)]
    pub volume: Option<f32>,

    /// Show OS-notification
    #[arg(long)]
    pub show_notification: Option<bool>,

    /// Timer in seconds
    #[arg(long)]
    pub timer: Option<u64>,

    /// Minor break time in seconds
    #[arg(long)]
    pub minor_break: Option<u64>,

    /// Major break time in seconds
    #[arg(long)]
    pub major_break: Option<u64>,

    /// Intervals before major break
    #[arg(long)]
    pub intervals: Option<u64>,
}

#[derive(clap::Args, Clone, Debug)]
pub struct ClientConfig {
    #[arg(long, short = 'i', long_help = include_str!("./ViewConfig.md"), verbatim_doc_comment)]
    pub interface: Option<String>,
}

/// Starts the timer or attaches to an already running timer
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    common_args: CommonArgs,

    #[command(flatten)]
    client_config: ClientConfig,

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
        #[command(flatten)]
        common_args: CommonArgs,
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
            ServerCommands::Start { common_args } => start_daemonized(common_args),
            ServerCommands::Stop => stop(),
            ServerCommands::Status => status(),
        },

        None => default_cmd(&cli.common_args, &cli.client_config),
    }
}
