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

pub struct IdleTimer {
    remaining_time: Duration,
}
pub struct RunningTimer {
    target_time: Instant,
}

impl TimerState for IdleTimer {}
impl TimerState for RunningTimer {}

struct ActualTimerState {
    input_receiver: Receiver<InputEvent<Event>>,
    view_sender: Sender<TerminalEvent>,
}

pub struct PomodoroTimer<S: TimerState> {
    state: Box<ActualTimerState>,
    extra: S,
}

impl PomodoroTimer<IdleTimer> {
    pub fn new(
        input_receiver: Receiver<InputEvent<Event>>,
        view_sender: Sender<TerminalEvent>,
        duration: Duration,
    ) -> Self {
        Self {
            state: Box::new(ActualTimerState {
                input_receiver,
                view_sender,
            }),
            extra: IdleTimer {
                remaining_time: duration,
            },
        }
    }

    pub fn run(self) {
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

    fn unpause(self) {
        PomodoroTimer {
            state: self.state,
            extra: RunningTimer {
                target_time: Instant::now() + self.extra.remaining_time,
            },
        }
        .start();
    }
}

impl PomodoroTimer<RunningTimer> {
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

    fn pause(self) {
        PomodoroTimer {
            state: self.state,
            extra: IdleTimer {
                remaining_time: self.extra.target_time - Instant::now(),
            },
        }
        .run();
    }
}
