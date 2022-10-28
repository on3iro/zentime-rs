use crate::config::Config;
use crate::events::AppAction;
use crate::events::InputEvent;
use crate::events::TerminalEvent;
use crate::events::ViewState;
use crate::input::handle_input;
use crate::util::seconds_to_time;
use crossterm::event::Event;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::time::Instant;

// NOTE: I tried to use the typestate approach, like it's described here:
// https://cliffle.com/blog/rust-typestate/

pub trait TimerState {}

pub struct Paused {
    remaining_time: Duration,
}
struct Running {
    target_time: Instant,
}

impl TimerState for Paused {}
impl TimerState for Running {}

struct ActualTimerState {
    round: u64,
    is_break: bool,
}

pub struct PomodoroTimer<S: TimerState> {
    config: Config,
    shared_state: Box<ActualTimerState>,
    internal_state: S,
    input_receiver: Receiver<InputEvent<Event>>,
    view_sender: Sender<TerminalEvent>,
}

impl<S: TimerState> PomodoroTimer<S> {
    fn next(self) {
        let is_major_break = self.shared_state.round % self.config.timers.intervals == 0;

        let new_timer = if self.shared_state.is_break {
            self.new_timer()
        } else {
            self.new_break_timer(is_major_break)
        };

        new_timer.init();
    }

    fn new_break_timer(self, is_major_break: bool) -> PomodoroTimer<Paused> {
        let break_length = if is_major_break {
            self.config.timers.major_break
        } else {
            self.config.timers.minor_break
        };

        PomodoroTimer {
            input_receiver: self.input_receiver,
            view_sender: self.view_sender,
            config: self.config,
            shared_state: Box::new(ActualTimerState {
                round: self.shared_state.round,
                is_break: true,
            }),
            internal_state: Paused {
                remaining_time: Duration::from_secs(break_length),
            },
        }
    }

    fn new_timer(self) -> PomodoroTimer<Paused> {
        let remaining_time = Duration::from_secs(self.config.timers.timer);

        PomodoroTimer {
            input_receiver: self.input_receiver,
            view_sender: self.view_sender,
            config: self.config,
            shared_state: Box::new(ActualTimerState {
                round: self.shared_state.round + 1,
                is_break: false,
            }),
            internal_state: Paused { remaining_time },
        }
    }
}

impl PomodoroTimer<Paused> {
    /// Creates a new paused timer
    pub fn new(
        input_receiver: Receiver<InputEvent<Event>>,
        view_sender: Sender<TerminalEvent>,
        config: Config,
    ) -> Self {
        let remaining_time = Duration::from_secs(config.timers.timer);

        Self {
            config,
            input_receiver,
            view_sender,
            shared_state: Box::new(ActualTimerState {
                round: 1,
                is_break: false,
            }),
            internal_state: Paused { remaining_time },
        }
    }

    /// Puts the paused timer into a waiting state waiting for input.
    pub fn init(self) {
        loop {
            let time = self.internal_state.remaining_time.as_secs();
            self.view_sender
                .send(TerminalEvent::View(ViewState {
                    is_break: self.shared_state.is_break,
                    round: self.shared_state.round,
                    time: seconds_to_time(time),
                }))
                .unwrap();

            let action = match self.input_receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(event) => handle_input(event),
                Err(RecvTimeoutError::Disconnected) => AppAction::Quit,
                _ => AppAction::None,
            };

            match action {
                AppAction::Quit => {
                    self.view_sender.send(TerminalEvent::Quit).unwrap();
                }
                AppAction::PlayPause => {
                    self.unpause();
                    break;
                }
                AppAction::None => {}
            }
        }
    }

    /// Transitions the paused timer into a running timer
    fn unpause(self) {
        PomodoroTimer {
            input_receiver: self.input_receiver,
            view_sender: self.view_sender,
            config: self.config,
            shared_state: self.shared_state,
            internal_state: Running {
                target_time: Instant::now() + self.internal_state.remaining_time,
            },
        }
        .start();
    }
}

impl PomodoroTimer<Running> {
    /// Runs the timer and awaits input
    fn start(self) {
        while self.internal_state.target_time > Instant::now() {
            let time = (self.internal_state.target_time - Instant::now()).as_secs();
            self.view_sender
                .send(TerminalEvent::View(ViewState {
                    is_break: self.shared_state.is_break,
                    round: self.shared_state.round,
                    time: seconds_to_time(time),
                }))
                .unwrap();

            let action = match self.input_receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(event) => handle_input(event),
                Err(RecvTimeoutError::Disconnected) => AppAction::Quit,
                _ => AppAction::None,
            };

            match action {
                AppAction::Quit => {
                    self.view_sender.send(TerminalEvent::Quit).unwrap();
                }
                AppAction::PlayPause => {
                    return self.pause();
                }
                AppAction::None => {}
            }
        }

        self.next();
    }

    /// Transitions the running timer into a paused timer state
    fn pause(self) {
        PomodoroTimer {
            input_receiver: self.input_receiver,
            view_sender: self.view_sender,
            config: self.config,
            shared_state: self.shared_state,
            internal_state: Paused {
                remaining_time: self.internal_state.target_time - Instant::now(),
            },
        }
        .init();
    }
}
