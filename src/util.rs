use crate::events::AppAction;
use crossterm::{event::DisableMouseCapture, execute, terminal::disable_raw_mode};
use std::io::Stdout;
use std::process;
use tui::backend::CrosstermBackend;
use tui::Terminal;

pub fn seconds_to_time(duration: u64) -> String {
    let min = duration / 60;
    let sec = duration % 60;
    format!("{:02}:{:02}", min, sec)
}

/// Quit by gracefully terminating
pub fn quit(terminal: &mut Terminal<CrosstermBackend<Stdout>>, msg: Option<&str>) -> AppAction {
    disable_raw_mode().unwrap();
    terminal.show_cursor().unwrap();
    execute!(std::io::stdout(), DisableMouseCapture).unwrap();

    println!("\n{}", msg.unwrap_or(""));

    process::exit(0)
}
