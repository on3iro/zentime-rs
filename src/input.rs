use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::thread::JoinHandle;
use std::{sync::mpsc::Sender, thread, time::Duration};
use zentime_rs_timer::TimerAction;

use crossterm::event::{self, Event};

pub enum InputEvent<I> {
    Input(I),
}

pub struct TerminalInputThread {}

impl TerminalInputThread {
    pub fn spawn(input_worker_tx: Sender<TimerAction>) -> JoinHandle<()> {
        thread::spawn(move || loop {
            if event::poll(Duration::from_millis(200)).expect("poll works") {
                let crossterm_event = event::read().expect("can read events");
                input_worker_tx
                    .send(handle_input(InputEvent::Input(crossterm_event)))
                    .expect("can send events");
            }
        })
    }
}

fn handle_input(event: InputEvent<Event>) -> TimerAction {
    if let InputEvent::Input(Event::Key(key_event)) = event {
        match key_event {
            KeyEvent {
                code: KeyCode::Char('q'),
                ..
            }
            | KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                return TimerAction::Quit;
            }

            KeyEvent {
                code: KeyCode::Char(' '),
                ..
            } => {
                return TimerAction::PlayPause;
            }

            KeyEvent {
                code: KeyCode::Char('s'),
                ..
            } => {
                return TimerAction::Skip;
            }

            _ => {}
        }
    }

    TimerAction::None
}
