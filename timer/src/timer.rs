use anyhow::Context;

use crate::config::TimerConfig;
use crate::events::{AppAction, TerminalEvent, ViewState};
use crate::util::seconds_to_time;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::time::{Duration, Instant};

// NOTE: I tried to use the typestate approach, like it's described here:
// https://cliffle.com/blog/rust-typestate/

// TODO
// use thiserror crate

/// Empty trait implemented by structs (e.g. Paused, Running)
pub trait TimerState {}

/// State specific to a paused timer
pub struct Paused {
    remaining_time: Duration,
}

/// State specific to a running timer
pub struct Running {
    target_time: Instant,
}

impl TimerState for Paused {}
impl TimerState for Running {}

#[derive(Clone, Copy)]
pub struct TimerStateData {
    pub round: u64,
    pub is_break: bool,
}

type OnTimerEnd = Box<dyn Fn(TimerStateData, &str)>;

/// Timer which can either be in a paused state or a running state.
/// To instantiate the timer run `Timer::new()`.
/// To actually start it call `Timer::init()`
/// This puts the timer into a paused state waiting for [AppAction](AppAction)s to be sent down
/// the input channel. For example an [AppAction::PlayPause](AppAction::PlayPause) starts the timer.
///
/// ## Example
///
/// ```
/// use zentime_rs_timer::config::{TimerConfig};
/// use zentime_rs_timer::events::{TerminalEvent, AppAction};
/// use zentime_rs_timer::timer::{Timer};
/// use std::time::Duration;
/// use std::sync::mpsc;
/// use std::thread;
/// use std::sync::mpsc::{Sender, Receiver};
///
/// let (terminal_input_sender, terminal_input_receiver): (Sender<AppAction>, Receiver<AppAction>) =
///     mpsc::channel();
/// let (view_sender, view_receiver): (Sender<TerminalEvent>, Receiver<TerminalEvent>) =
///     mpsc::channel();
///
/// // ...Do something with the terminal_input_sender and view_receiver
/// // e.g. have a thread handle keyboard input and send AppActions and
/// // have another thread render a TUI (terminal user interface) receiving
/// // the view state of the timer)
/// // ...
///
/// // Run timer in its own thread so it does not block the current one
/// thread::spawn(move || {
///     Timer::new(terminal_input_receiver, view_sender, TimerConfig::default(), Box::new(move |state, msg| {
///         println!("{} {}", state.round, msg);
///     }))
///         .init()
///         .expect("Could not initialize timer");
/// });
/// ```
pub struct Timer<S: TimerState> {
    config: TimerConfig,
    on_interval_end: OnTimerEnd,
    shared_state: Box<TimerStateData>,
    internal_state: S,
    app_action_receiver: Receiver<AppAction>,
    view_sender: Sender<TerminalEvent>,
}

impl<S: TimerState> Timer<S> {
    fn next(self, is_timer_end: bool) -> anyhow::Result<()> {
        let is_major_break = self.shared_state.round % self.config.intervals == 0;
        let shared_state = self.shared_state.clone();

        let (new_timer, notification_string) = if self.shared_state.is_break {
            (self.new_timer(), "Break is over")
        } else {
            (
                self.new_break_timer(is_major_break),
                "Good job! Take a break",
            )
        };

        if is_timer_end {
            (new_timer.on_interval_end)(*shared_state, notification_string);
        }

        new_timer.init()
    }

    fn new_break_timer(self, is_major_break: bool) -> Timer<Paused> {
        let break_length = if is_major_break {
            self.config.major_break
        } else {
            self.config.minor_break
        };

        Timer {
            app_action_receiver: self.app_action_receiver,
            on_interval_end: self.on_interval_end,
            view_sender: self.view_sender,
            config: self.config,
            shared_state: Box::new(TimerStateData {
                round: self.shared_state.round,
                is_break: true,
            }),
            internal_state: Paused {
                remaining_time: Duration::from_secs(break_length),
            },
        }
    }

    fn new_timer(self) -> Timer<Paused> {
        let remaining_time = Duration::from_secs(self.config.timer);

        Timer {
            app_action_receiver: self.app_action_receiver,
            on_interval_end: self.on_interval_end,
            view_sender: self.view_sender,
            config: self.config,
            shared_state: Box::new(TimerStateData {
                round: self.shared_state.round + 1,
                is_break: false,
            }),
            internal_state: Paused { remaining_time },
        }
    }
}

impl Timer<Paused> {
    /// Creates a new timer in paused state
    pub fn new(
        input_receiver: Receiver<AppAction>,
        view_sender: Sender<TerminalEvent>,
        config: TimerConfig,
        on_interval_end: OnTimerEnd,
    ) -> Self {
        let remaining_time = Duration::from_secs(config.timer);

        Self {
            config,
            on_interval_end,
            app_action_receiver: input_receiver,
            view_sender,
            shared_state: Box::new(TimerStateData {
                round: 1,
                is_break: false,
            }),
            internal_state: Paused { remaining_time },
        }
    }

    /// Puts the paused timer into a waiting state waiting for input (e.g. to unpause the timer
    /// and transition it into a running state).
    pub fn init(self) -> anyhow::Result<()> {
        loop {
            let time = self.internal_state.remaining_time.as_secs();
            self.view_sender
                .send(TerminalEvent::View(ViewState {
                    is_break: self.shared_state.is_break,
                    round: self.shared_state.round,
                    time: seconds_to_time(time),
                }))
                .context("View sender could not send")?;

            let action = match self
                .app_action_receiver
                .recv_timeout(Duration::from_secs(1))
            {
                Ok(action) => action,
                Err(RecvTimeoutError::Disconnected) => AppAction::Quit,
                _ => AppAction::None,
            };

            match action {
                AppAction::Quit => {
                    self.view_sender.send(TerminalEvent::Quit)?;
                    return Ok(());
                }
                AppAction::PlayPause => {
                    self.unpause()?;
                    break;
                }
                AppAction::Skip => {
                    return self.next(false);
                }

                AppAction::None => {}
            }
        }

        Ok(())
    }

    /// Transitions the paused timer into a running timer
    fn unpause(self) -> anyhow::Result<()> {
        Timer {
            app_action_receiver: self.app_action_receiver,
            on_interval_end: self.on_interval_end,
            view_sender: self.view_sender,
            config: self.config,
            shared_state: self.shared_state,
            internal_state: Running {
                target_time: Instant::now() + self.internal_state.remaining_time,
            },
        }
        .start()
    }
}

impl Timer<Running> {
    /// Runs the timer and awaits input
    fn start(self) -> anyhow::Result<()> {
        while self.internal_state.target_time > Instant::now() {
            let time = (self.internal_state.target_time - Instant::now()).as_secs();
            self.view_sender
                .send(TerminalEvent::View(ViewState {
                    is_break: self.shared_state.is_break,
                    round: self.shared_state.round,
                    time: seconds_to_time(time),
                }))
                .context("View sender could not send")?;

            let action = match self
                .app_action_receiver
                .recv_timeout(Duration::from_secs(1))
            {
                Ok(action) => action,
                Err(RecvTimeoutError::Disconnected) => AppAction::Quit,
                _ => AppAction::None,
            };

            match action {
                AppAction::Quit => {
                    self.view_sender
                        .send(TerminalEvent::Quit)
                        .context("Could not send quit event")?;
                    return Ok(());
                }
                AppAction::PlayPause => {
                    return self.pause();
                }
                AppAction::Skip => {
                    return self.next(false);
                }
                AppAction::None => {}
            }
        }

        self.next(true)
    }

    /// Transitions the running timer into a paused timer state
    fn pause(self) -> anyhow::Result<()> {
        Timer {
            app_action_receiver: self.app_action_receiver,
            view_sender: self.view_sender,
            config: self.config,
            on_interval_end: self.on_interval_end,
            shared_state: self.shared_state,
            internal_state: Paused {
                remaining_time: self.internal_state.target_time - Instant::now(),
            },
        }
        .init()
    }
}
