use crate::client::view::timer_view;
use crossterm::cursor::Hide;
use crossterm::style::Stylize;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor::Show, event::DisableMouseCapture, execute, terminal::disable_raw_mode};
use std::io::Write;
use std::{io::Stdout, process, sync::mpsc::Receiver};
use tui::{backend::CrosstermBackend, Terminal as TuiTerminal};

use super::terminal_event::TerminalEvent;

pub trait Terminal {
    fn new() -> Self;
    fn render(self, terminal_rx: Receiver<TerminalEvent>);
    fn quit(self, msg: Option<&str>, is_error: bool);
}

pub struct DefaultTerminal {
    tui_terminal: TuiTerminal<CrosstermBackend<Stdout>>,
}

impl Terminal for DefaultTerminal {
    fn new() -> Self {
        let backend = CrosstermBackend::new(std::io::stdout());
        execute!(std::io::stdout(), EnterAlternateScreen).expect("Can't execute crossterm macros");
        let mut terminal = TuiTerminal::new(backend).expect("Terminal could be created");
        enable_raw_mode().expect("Can run in raw mode");
        terminal.clear().expect("Terminal could be cleared");
        terminal.hide_cursor().expect("Could not hide cursor");

        Self {
            tui_terminal: terminal,
        }
    }

    fn render(mut self, terminal_rx: Receiver<TerminalEvent>) {
        loop {
            match terminal_rx.recv() {
                Ok(TerminalEvent::View(state)) => {
                    if let Err(err) = timer_view(&mut self.tui_terminal, state) {
                        return self.quit(Some(&format!("ERROR: {}", err)), true);
                    };
                }
                Ok(TerminalEvent::Quit) => {
                    return self.quit(Some("Cya!"), false);
                }
                _ => {}
            }
        }
    }

    fn quit(mut self, msg: Option<&str>, is_error: bool) {
        disable_raw_mode().expect("Could not disable raw mode");
        self.tui_terminal
            .show_cursor()
            .expect("Could not show cursor");
        self.tui_terminal.clear().expect("Could not clear terminal");
        execute!(std::io::stdout(), DisableMouseCapture, LeaveAlternateScreen)
            .expect("Could not execute crossterm macros");

        println!("\n{}", msg.unwrap_or(""));

        process::exit(i32::from(is_error))
    }
}

pub struct MinimalTerminal {}

impl Terminal for MinimalTerminal {
    fn new() -> Self {
        enable_raw_mode().expect("Can run in raw mode");

        execute!(std::io::stdout(), Hide).expect("Could not execute crossterm macros");
        Self {}
    }

    fn render(self, terminal_rx: Receiver<TerminalEvent>) {
        loop {
            match terminal_rx.recv() {
                Ok(TerminalEvent::View(state)) => {
                    let timer = format!(" {} ", state.time.white());
                    let round = format!("Round: {}", state.round);
                    print!(
                        "\r{} {} {}",
                        timer.on_dark_red(),
                        round.green(),
                        if state.is_break { "Break" } else { "Focus" }
                    );

                    if let Err(err) = std::io::stdout().flush() {
                        return self.quit(Some(&format!("ERROR: {}", err)), true);
                    };
                }

                Ok(TerminalEvent::Quit) => {
                    return self.quit(Some("Cya!"), false);
                }
                _ => {}
            }
        }
    }

    fn quit(self, msg: Option<&str>, is_error: bool) {
        disable_raw_mode().expect("Could not disable raw mode");
        execute!(std::io::stdout(), Show, DisableMouseCapture)
            .expect("Could not execute crossterm macros");

        println!("\r\n{}", msg.unwrap_or(""));

        process::exit(i32::from(is_error))
    }
}
