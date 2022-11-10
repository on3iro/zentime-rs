use crate::events::AppAction;
use crossterm::{event::DisableMouseCapture, execute, terminal::disable_raw_mode};
use std::{io::Stdout, process};
use tui::{backend::CrosstermBackend, Terminal};

pub fn seconds_to_time(duration: u64) -> String {
    let min = duration / 60;
    let sec = duration % 60;
    format!("{:02}:{:02}", min, sec)
}

/// Quit by gracefully terminating
pub fn quit(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    msg: Option<&str>,
    is_error: bool,
) -> AppAction {
    disable_raw_mode().expect("Could not disable raw mode");
    terminal.show_cursor().expect("Could not show cursor");
    terminal.clear().expect("Could not clear terminal");
    execute!(std::io::stdout(), DisableMouseCapture).expect("Could not disable mouse capture");

    println!("\n\n\n\n\n{}", msg.unwrap_or(""));

    process::exit(if is_error { 1 } else { 0 })
}
