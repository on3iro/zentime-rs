use clap::{Parser, Subcommand};
use spin_sleep::sleep;
use std::path::PathBuf;
use std::time;

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

    // TODO
    // 1. Add crossterm
    // 2. Split into rendering and input threads
    // 3. Refactor
    // 4. Add play/pause
    // 5. Add configuration parsing

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

            // 25minutes
            let mut remaining_time = 60;

            while remaining_time > 0 {
                remaining_time -= 1;
                let time = seconds_to_time(remaining_time);
                println!("{}", time);
                sleep(time::Duration::new(1, 0));
            }
        }

        Some(_Commands) => {}

        None => {}
    }
}

fn seconds_to_time(duration: u16) -> String {
    let min = duration / 60;
    let sec = duration % 60;
    format!("{:02}:{:02}", min, sec)
}
