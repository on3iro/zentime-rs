use crossterm::{cursor::Show, event::DisableMouseCapture, execute, terminal::disable_raw_mode};
use std::{io::Stdout, process};
use tui::{backend::CrosstermBackend, Terminal};
use zentime_rs_timer::TimerAction;

/// Quit by gracefully terminating
pub fn quit(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    msg: Option<&str>,
    is_error: bool,
) -> TimerAction {
    disable_raw_mode().expect("Could not disable raw mode");
    terminal.show_cursor().expect("Could not show cursor");
    terminal.clear().expect("Could not clear terminal");
    execute!(std::io::stdout(), Show, DisableMouseCapture)
        .expect("Could not disable mouse capture");

    println!("\n\n\n\n\n{}", msg.unwrap_or(""));

    process::exit(i32::from(is_error))
}
