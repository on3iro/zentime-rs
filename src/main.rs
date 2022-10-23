use clap::{Parser, Subcommand};
use crossterm::terminal::enable_raw_mode;
use spin_sleep::sleep;
use std::io::Error;
use std::io::Stdout;
use std::path::PathBuf;
use std::time;
use tui::backend::CrosstermBackend;
use tui::layout::Alignment;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::Color;
use tui::style::Style;
use tui::widgets::Block;
use tui::widgets::Borders;
use tui::widgets::Paragraph;
use tui::Terminal;

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
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
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

/// Base layout of the program
pub fn layout(rect: Rect) -> Vec<Rect> {
    Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(2),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(rect)
}

fn render(terminal: &mut Terminal<CrosstermBackend<Stdout>>, timer: &str) -> Result<(), Error> {
    terminal.draw(|frame| {
        let rect = frame.size();
        let layout = layout(rect);
        let timer = Paragraph::new(timer)
            .block(Block::default().title("zentime").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(timer, layout[0])
    })?;

    Ok(())
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

            enable_raw_mode().expect("Can run in raw mode");
            let stdout = std::io::stdout();
            let backend = CrosstermBackend::new(stdout);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal.clear().unwrap();

            // 25minutes
            let mut remaining_time = 60;

            while remaining_time > 0 {
                remaining_time -= 1;
                let time = seconds_to_time(remaining_time);
                render(&mut terminal, &time).expect("Can render");
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
