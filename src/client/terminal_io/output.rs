use crate::client::view::timer_view;
use anyhow::Context;
use crossterm::cursor::Hide;
use crossterm::style::Stylize;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor::Show, event::DisableMouseCapture, execute, terminal::disable_raw_mode};
use std::io::Write;
use std::{io::Stdout, process};
use tui::{backend::CrosstermBackend, Terminal as TuiTerminal};
use zentime_rs_timer::timer::ViewState;

pub trait TerminalOut {
    fn render(&mut self, state: ViewState) -> anyhow::Result<()>;
    fn quit(&mut self, msg: Option<String>, is_error: bool);
}

pub struct DefaultInterface {
    tui_terminal: TuiTerminal<CrosstermBackend<Stdout>>,
}

impl DefaultInterface {
    pub fn new() -> anyhow::Result<Self> {
        let backend = CrosstermBackend::new(std::io::stdout());
        execute!(std::io::stdout(), EnterAlternateScreen)
            .context("Can't execute crossterm macros")?;
        let mut terminal =
            TuiTerminal::new(backend).context("Tui-Terminal could not be created")?;
        enable_raw_mode().context("Can't run in raw mode")?;
        terminal.clear().context("Terminal could not be cleared")?;
        terminal.hide_cursor().context("Could not hide cursor")?;

        Ok(Self {
            tui_terminal: terminal,
        })
    }
}

impl TerminalOut for DefaultInterface {
    fn render(&mut self, state: ViewState) -> anyhow::Result<()> {
        timer_view(&mut self.tui_terminal, state)
    }

    fn quit(&mut self, msg: Option<String>, is_error: bool) {
        disable_raw_mode().expect("Could not disable raw mode");
        self.tui_terminal
            .show_cursor()
            .expect("Could not show cursor");
        self.tui_terminal.clear().expect("Could not clear terminal");
        execute!(std::io::stdout(), DisableMouseCapture, LeaveAlternateScreen)
            .expect("Could not execute crossterm macros");

        println!("\n{}", msg.unwrap_or_else(|| String::from("")));

        process::exit(i32::from(is_error))
    }
}

pub struct MinimalInterface {}

impl MinimalInterface {
    pub fn new() -> anyhow::Result<Self> {
        enable_raw_mode().context("Can't run in raw mode")?;

        execute!(std::io::stdout(), Hide).context("Could not execute crossterm macros")?;
        Ok(Self {})
    }
}

impl TerminalOut for MinimalInterface {
    fn render(&mut self, state: ViewState) -> anyhow::Result<()> {
        let timer = format!(" {} ", state.time.white());
        let round = format!("Round: {}", state.round);
        print!(
            "\r{} {} {}",
            timer.on_dark_red(),
            round.green(),
            if state.is_break { "Break" } else { "Focus" }
        );

        Ok(std::io::stdout().flush()?)
    }

    fn quit(&mut self, msg: Option<String>, is_error: bool) {
        disable_raw_mode().expect("Could not disable raw mode");
        execute!(std::io::stdout(), Show, DisableMouseCapture)
            .expect("Could not execute crossterm macros");

        println!("\r\n{}", msg.unwrap_or_else(|| String::from("")));

        process::exit(i32::from(is_error))
    }
}
