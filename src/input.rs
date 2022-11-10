use crate::events::AppAction;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use std::thread::JoinHandle;
use std::{sync::mpsc::Sender, thread, time::Duration};

use crossterm::event::{self, Event};

use crate::events::InputEvent;

pub struct TerminalInputThread {}

impl TerminalInputThread {
    pub fn spawn(input_worker_tx: Sender<AppAction>) -> JoinHandle<()> {
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

fn handle_input(event: InputEvent<Event>) -> AppAction {
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
                return AppAction::Quit;
            }

            KeyEvent {
                code: KeyCode::Char(' '),
                ..
            } => {
                return AppAction::PlayPause;
            }

            KeyEvent {
                code: KeyCode::Char('s'),
                ..
            } => {
                return AppAction::Skip;
            }

            _ => {}
        }
    }

    AppAction::None
}
