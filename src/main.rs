use crate::default_cmd::default_cmd;
use clap::{Parser, Subcommand};
use env_logger::Env;

mod default_cmd;
mod subcommands;
use serde::{Deserialize, Serialize};
use subcommands::{
    query_server_once::query_server_once,
    reset_timer::reset_timer,
    server::{start_daemonized, status, stop},
    skip_timer::skip_timer,
    toggle_timer::toggle_timer,
};

#[derive(clap::Args)]
pub struct CommonArgs {
    /// Sets a custom config file
    #[arg(short, long, default_value = "~/.config/zentime/zentime.toml")]
    config: String,

    #[command(flatten)]
    server_config: ServerConfig,
}

/// This should match [Config::NotificationConfig], but makes fields optional, so that they are not
/// required by clap. If no value is provided and therefore the `Option` is `None`, we skip
/// serializing the value.
#[derive(clap::Args, Serialize, Deserialize, Clone, Debug)]
#[serde(rename(serialize = "NotificationConfig"))]
struct ClapNotificationConfig {
    /// Enable/Disable bell
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub enable_bell: Option<bool>,

    /// Path to soundfile which is played back on each interval end
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub sound_file: Option<String>,

    /// Notification bell volume
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub volume: Option<f32>,

    /// Show OS-notification
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub show_notification: Option<bool>,
}

/// This should match [zentime-rs-timer::config::TimerConfig], but makes fields optional, so that they are not
/// required by clap. If no value is provided and therefore the `Option` is `None`, we skip
/// serializing the value.
#[derive(clap::Args, Serialize, Deserialize, Copy, Clone, Debug)]
#[serde(rename(serialize = "TimerConfig"))]
struct ClapTimerConfig {
    /// Timer in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub timer: Option<u64>,

    /// Minor break time in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub minor_break: Option<u64>,

    /// Major break time in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub major_break: Option<u64>,

    /// Intervals before major break
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub intervals: Option<u64>,

    /// Determines how often a break may be postponed.
    /// A value of 0 denotes, that postponing breaks is not allowed and the feature is
    /// disabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub postpone_limit: Option<u16>,

    /// Determines how long each postpone timer runs (in seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub postpone_timer: Option<u64>,
}

#[derive(clap::Args, Serialize, Deserialize, Clone, Debug)]
#[serde(rename(serialize = "Config"))]
pub struct ServerConfig {
    #[command(flatten)]
    timers: ClapTimerConfig,

    #[command(flatten)]
    notifications: ClapNotificationConfig,
}

/// This should match [Config::ViewConfig], but makes fields optional, so that they are not
/// required by clap. If no value is provided and therefore the `Option` is `None`, we skip
/// serializing the value.
#[derive(clap::Args, Serialize, Deserialize, Clone, Debug)]
#[serde(rename(serialize = "ViewConfig"))]
struct ClapViewConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[arg(long, short = 'i', long_help = include_str!("./ViewConfig.md"), verbatim_doc_comment)]
    pub interface: Option<String>,
}

#[derive(clap::Args, Serialize, Deserialize, Clone, Debug)]
#[serde(rename(serialize = "Config"))]
pub struct ClientConfig {
    #[command(flatten)]
    view: ClapViewConfig,
}

/// Starts the timer or attaches to an already running timer
#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = "When run without command: starts the zentime server if necessary and attaches a client to it."
)]
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
    /// Runs a single [ViewState]-query against the server and
    /// terminates the connection afterwards.
    /// This is useful for integration with other tools such as tmux, to integrate
    /// zentime into a status bar etc.
    Once,

    /// Toggles between timer play/pause
    ToggleTimer,

    /// Skips to next timer interval
    Skip,

    /// Resets the timer to the first interval
    Reset,

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
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .target(env_logger::Target::Stdout)
        .init();
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Server { command }) => match command {
            ServerCommands::Start { common_args } => start_daemonized(common_args),
            ServerCommands::Stop => stop(),
            ServerCommands::Status => status(),
        },

        Some(Commands::Once) => {
            query_server_once();
        }

        Some(Commands::ToggleTimer) => {
            toggle_timer();
        }

        Some(Commands::Skip) => {
            skip_timer();
        }

        Some(Commands::Reset) => {
            reset_timer();
        }

        None => default_cmd(&cli.common_args, &cli.client_config),
    }
}
