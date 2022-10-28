use crate::config::Config;
use crate::events::AppAction;
use crate::events::InputEvent;
use crate::events::TerminalEvent;
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
    input_receiver: Receiver<InputEvent<Event>>,
    view_sender: Sender<TerminalEvent>,
    round: u16,
    is_break: bool,
}

pub struct PomodoroTimer<S: TimerState> {
    config: Config,
    state: Box<ActualTimerState>,
    extra: S,
}

impl<S: TimerState> PomodoroTimer<S> {
    fn next(&self) {}
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
            state: Box::new(ActualTimerState {
                input_receiver,
                view_sender,
                round: 1,
                is_break: false,
            }),
            extra: Paused { remaining_time },
        }
    }

    /// Puts the paused timer into a waiting state waiting for input.
    pub fn init(self) {
        loop {
            let time = self.extra.remaining_time.as_secs();
            self.state
                .view_sender
                .send(TerminalEvent::View(seconds_to_time(time)))
                .unwrap();

            let action = match self
                .state
                .input_receiver
                .recv_timeout(Duration::from_secs(1))
            {
                Ok(event) => handle_input(event),
                Err(RecvTimeoutError::Disconnected) => AppAction::Quit,
                _ => AppAction::None,
            };

            match action {
                AppAction::Quit => {
                    self.state.view_sender.send(TerminalEvent::Quit).unwrap();
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
            config: self.config,
            state: self.state,
            extra: Running {
                target_time: Instant::now() + self.extra.remaining_time,
            },
        }
        .start();
    }
}

impl PomodoroTimer<Running> {
    /// Runs the timer and awaits input
    fn start(self) {
        while self.extra.target_time > Instant::now() {
            let time = (self.extra.target_time - Instant::now()).as_secs();
            self.state
                .view_sender
                .send(TerminalEvent::View(seconds_to_time(time)))
                .unwrap();

            let action = match self
                .state
                .input_receiver
                .recv_timeout(Duration::from_secs(1))
            {
                Ok(event) => handle_input(event),
                Err(RecvTimeoutError::Disconnected) => AppAction::Quit,
                _ => AppAction::None,
            };

            match action {
                AppAction::Quit => {
                    self.state.view_sender.send(TerminalEvent::Quit).unwrap();
                }
                AppAction::PlayPause => {
                    return self.pause();
                }
                AppAction::None => {}
            }
        }

        self.state.view_sender.send(TerminalEvent::Quit).unwrap();
    }

    /// Transitions the running timer into a paused timer state
    fn pause(self) {
        PomodoroTimer {
            config: self.config,
            state: self.state,
            extra: Paused {
                remaining_time: self.extra.target_time - Instant::now(),
            },
        }
        .init();
    }
}
