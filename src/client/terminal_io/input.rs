//! Code related to async client terminal input handling

use crossterm::event::{EventStream, KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;
use tokio::task::yield_now;
use tokio::{spawn, task::JoinHandle};
use tokio_stream::StreamExt;

use crossterm::event::Event;

/// Actions triggered by user terminal input on a client
#[derive(Copy, Clone, Debug)]
pub enum ClientInputAction {
    /// Quit Timer and terminate server
    Quit,

    /// Detach current client without terminating server
    Detach,

    /// NoOp
    None,

    /// Either start or pause the current timer
    PlayPause,

    /// Skip to the next timer (break or focus)
    Skip,
}

/// Tokio task handling terminal input events
#[derive(Copy, Clone, Debug)]
pub struct TerminalInputTask {}

impl TerminalInputTask {
    /// Spanws the task and converts incoming terminal input events into [ClientInputAction]s and
    /// sends them to the client.
    pub async fn spawn(input_worker_tx: UnboundedSender<ClientInputAction>) -> JoinHandle<()> {
        spawn(async move {
            let mut stream = EventStream::new();

            loop {
                let result = stream.next().await;
                if let Some(Ok(event)) = result {
                    if let Err(error) = input_worker_tx.send(handle_input(event)) {
                        // TODO: handle this more gracefully
                        panic!("Could not send ClientInputAction: {}", error)
                    };
                }

                yield_now().await;
            }
        })
    }
}

/// Keymap from terminal input events to [ClientInputAction]
fn handle_input(event: Event) -> ClientInputAction {
    if let Event::Key(key_event) = event {
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
                return ClientInputAction::Quit;
            }

            KeyEvent {
                code: KeyCode::Char('d'),
                ..
            } => {
                return ClientInputAction::Detach;
            }

            KeyEvent {
                code: KeyCode::Char(' '),
                ..
            } => {
                return ClientInputAction::PlayPause;
            }

            KeyEvent {
                code: KeyCode::Char('s'),
                ..
            } => {
                return ClientInputAction::Skip;
            }

            _ => {}
        }
    }

    ClientInputAction::None
}
