//! Code related to client async terminal output handling

use crate::client::terminal_io::default_interface::render;
use anyhow::Context;
use crossterm::cursor::Hide;
use crossterm::style::Stylize;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{cursor::Show, event::DisableMouseCapture, execute, terminal::disable_raw_mode};
use futures::lock::Mutex;
use zentime_rs_timer::pomodoro_timer::ViewState;
use std::io::Write;
use std::sync::Arc;
use std::{io::Stdout, process};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::{spawn, JoinHandle};
use tui::{backend::CrosstermBackend, Terminal as TuiTerminal};

use super::terminal_event::TerminalEvent;

/// Tokio task which continouusly renders the current view state to the terminal output.
#[derive(Copy, Clone, Debug)]
pub struct TerminalOutputTask {}

impl TerminalOutputTask {
    /// Spawns a tokio task which continously handles terminal output
    pub async fn spawn(
        terminal_out: Arc<Mutex<Box<dyn TerminalOut + Send>>>,
        mut out_rx: UnboundedReceiver<TerminalEvent>,
    ) -> JoinHandle<()> {
        spawn(async move {
            loop {
                match out_rx.recv().await {
                    Some(TerminalEvent::View(state)) => {
                        if let Err(error) = terminal_out.lock().await.render(state) {
                            return terminal_out
                                .lock()
                                .await
                                .quit(Some(format!("ERROR: {}", error)), true);
                        }
                    }
                    Some(TerminalEvent::Quit { msg, error }) => {
                        return terminal_out.lock().await.quit(msg, error);
                    }
                    None => continue,
                }
            }
        })
    }
}

/// Trait representing a terminal output
pub trait TerminalOut {
    /// Renders the current [ViewState]
    fn render(&mut self, state: ViewState) -> anyhow::Result<()>;

    /// Gracefully quits the [Self] so that raw-mode, alternate screens etc.
    /// are restored to their default.
    fn quit(&mut self, msg: Option<String>, is_error: bool);
}

/// Implementation of a [TerminalOut]
/// Uses a [TuiTerminal] with a [CrosstermBackend] to render.
#[allow(missing_debug_implementations)]
#[derive()]
pub struct DefaultInterface {
    tui_terminal: TuiTerminal<CrosstermBackend<Stdout>>,
}

impl DefaultInterface {
    /// Creates a new default interface
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
        render(&mut self.tui_terminal, state)
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

/// Minimal interface which uses a [Crossterm] to display colors, hide the cursor and enable raw mode.
/// The actual rendering happens with simple `print!`-macro-calls.
#[derive(Debug, Copy, Clone)]
pub struct MinimalInterface {}

impl MinimalInterface {
    /// Creates a new minimal interface and also enables raw mode and hides the cursor.
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
        let timer_kind = (if state.is_break {
                "Break"
            } else if state.is_postponed {
                "Postpone"
            } else {
                "Focus"
            }).to_string();

        let postponed_count = if state.is_postponed {
            format!(" ({})", state.postpone_count)
        } else {
            "".to_string()
        };

        let ansi_erase_line_escape = "\x1B[2K";
        let ansi_move_cursor_to_start_of_line_escape = "\r";

        print!(
            "{}{}{} {} {}{}",
            ansi_move_cursor_to_start_of_line_escape,
            ansi_erase_line_escape,
            if state.is_paused { timer.on_dark_green() } else { timer.on_dark_red() },
            round.green(),
            timer_kind,
            postponed_count
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
