use crate::config::Config;
use crate::config::NotificationConfig;
use crate::events::AppAction;
use crate::events::TerminalEvent;
use crate::events::ViewState;
use crate::notification;
use crate::sound;
use crate::sound::SoundFile;
use crate::util::seconds_to_time;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::time::Instant;

// NOTE: I tried to use the typestate approach, like it's described here:
// https://cliffle.com/blog/rust-typestate/

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

struct TimerStateData {
    round: u64,
    is_break: bool,
}

/// Timer which can either be in a paused state or a running state.
/// To instantiate the timer run `Timer::new()`.
/// To actually start it call `Timer::init()`
/// This puts the timer into a paused state waiting for [AppAction](AppAction)s to be sent down
/// the input channel. For example an [AppAction::PlayPause](AppAction::PlayPause) starts the timer.
///
/// ## Example
///
/// ```
/// use zentime_rs::config::{create_config};
/// use zentime_rs::{TerminalEvent, AppAction};
/// use zentime_rs::timer::{Timer};
/// use std::time::Duration;
/// use std::sync::mpsc;
/// use std::thread;
/// use std::sync::mpsc::{Sender, Receiver};
///
/// let config = create_config("")
///     .extract()
///     .expect("Could not create config");
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
///     Timer::new(terminal_input_receiver, view_sender, config)
///         .init()
///         .expect("Could not initialize timer");
/// });
/// ```
pub struct Timer<S: TimerState> {
    config: Config,
    shared_state: Box<TimerStateData>,
    internal_state: S,
    app_action_receiver: Receiver<AppAction>,
    view_sender: Sender<TerminalEvent>,
}

impl<S: TimerState> Timer<S> {
    fn dispatch_notification(
        config: NotificationConfig,
        notification_string: &str,
    ) -> anyhow::Result<()> {
        if config.enable_bell {
            sound::play(SoundFile::Bell, config.volume);
        }

        if config.show_notification {
            notification::send(notification_string)?;
        }
        Ok(())
    }

    fn next(self, notify: bool) -> anyhow::Result<()> {
        let is_major_break = self.shared_state.round % self.config.timers.intervals == 0;

        let config = self.config.notifications;

        let (new_timer, notification_string) = if self.shared_state.is_break {
            (self.new_timer(), "Break is over")
        } else {
            (
                self.new_break_timer(is_major_break),
                "Good job! Take a break",
            )
        };

        if notify {
            Timer::<S>::dispatch_notification(config, notification_string)?;
        }

        new_timer.init()
    }

    fn new_break_timer(self, is_major_break: bool) -> Timer<Paused> {
        let break_length = if is_major_break {
            self.config.timers.major_break
        } else {
            self.config.timers.minor_break
        };

        Timer {
            app_action_receiver: self.app_action_receiver,
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
        let remaining_time = Duration::from_secs(self.config.timers.timer);

        Timer {
            app_action_receiver: self.app_action_receiver,
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
        config: Config,
    ) -> Self {
        let remaining_time = Duration::from_secs(config.timers.timer);

        Self {
            config,
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
            self.view_sender.send(TerminalEvent::View(ViewState {
                is_break: self.shared_state.is_break,
                round: self.shared_state.round,
                time: seconds_to_time(time),
            }))?;

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
            self.view_sender.send(TerminalEvent::View(ViewState {
                is_break: self.shared_state.is_break,
                round: self.shared_state.round,
                time: seconds_to_time(time),
            }))?;

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
            shared_state: self.shared_state,
            internal_state: Paused {
                remaining_time: self.internal_state.target_time - Instant::now(),
            },
        }
        .init()
    }
}
