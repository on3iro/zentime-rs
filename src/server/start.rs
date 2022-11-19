use crate::client::terminal_event::TerminalEvent;
use crate::{client::notification::dispatch_notification, config::Config};
use std::time::Duration;
use zentime_rs_timer::{Timer, TimerAction};

use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};

pub fn start(
    config: Config,
    view_sender: Sender<TerminalEvent>,
    terminal_input_receiver: Receiver<TimerAction>,
) {
    Timer::new(
        config.timers,
        Box::new(move |_, msg| {
            // We simply discard errors here for now...
            dispatch_notification(config.notifications, msg).ok();
        }),
        Box::new(move |view_state| {
            // Update the view
            view_sender
                .send(TerminalEvent::View(view_state))
                .expect("Could not send to view");

            // Quit handler for re-use in various match arms
            let handle_quit = || {
                view_sender
                    .send(TerminalEvent::Quit)
                    .expect("Could not send quit event");
                Some(TimerAction::Quit)
            };

            // Handle app actions and hand them to the timer caller
            match terminal_input_receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(TimerAction::Quit) => handle_quit(),
                Ok(action) => Some(action),
                Err(RecvTimeoutError::Disconnected) => handle_quit(),
                _ => None,
            }
        }),
    )
    .init()
    .expect("Could not initialize timer");
}
