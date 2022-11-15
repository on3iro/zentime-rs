//! Implementation of the actual timer logic

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

/// State which is shared between timers when they transition from one timer state to the
/// next. Some of its information is also being shared with the `view_sender`.
#[derive(Clone, Copy)]
pub struct TimerStateData {
    /// State that denotes if the current timer is a break timer.
    /// If set to `false` the current timer is a regular focus interval timer.
    pub is_break: bool,

    /// The current pomodoro round.
    /// This is incremented after each break.
    pub round: u64,
}

type OnTimerEnd = Box<dyn Fn(TimerStateData, &str)>;

/// Timer which can either be in a paused state or a running state.
/// To instantiate the timer run `Timer::new()`.
/// To actually start it call `Timer::init()`
/// This puts the timer into a paused state waiting for [AppAction](AppAction)s to be sent down
/// the input channel. For example an [AppAction::PlayPause](AppAction::PlayPause) starts the timer.
pub struct Timer<S: TimerState> {
    /// Config describin how long intervals are etc.
    config: TimerConfig,

    // TODO make this optional
    /// Callback closure which is called at the end of each timer
    on_interval_end: OnTimerEnd,

    /// State shared between timers when they transition into each other
    shared_state: Box<TimerStateData>,

    /// Internal state data associated with a certain timer state (e.g. [Paused] or [Running])
    internal_state: S,

    /// Channel-receiver of [AppAction]
    app_action_receiver: Receiver<AppAction>,

    /// Channel-sender of [TerminalEvent]
    view_sender: Sender<TerminalEvent>,
}

impl<S: TimerState> Timer<S> {
    /// Transitions a timer from one timer to the next.
    /// The next timer will either be a break timer or a regular timer.
    /// Either way it will be initalized in a paused state.
    /// If [is_timer_end] is set to true, the [Timer::on_interval_end] will be called.
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

    /// Creates a new break timer whose length will either be that of a
    /// [TimerConfig::minor_break] or [TimerConfig::major_break]
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

    /// Creates a new regular interval timer
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

/// Implementation of the [Paused] state for [Timer]
impl Timer<Paused> {
    /// Creates a new timer in paused state.
    /// You have to call [Self::init()] to make the timer listen for inputs on its
    /// `input_receiver` so that it can actually be started.
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
    /// Runs the timer and awaits input.
    /// Depending on the input [AppAction] the timer might, Quit (and inform [Self::view_sender] about this),
    /// transition into a paused state or jump to the next interval.
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

    /// Transitions the running timer into a paused timer state and calls `init()` on_interval_end
    /// it, so that the new timer is ready to receive an [AppAction]
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
