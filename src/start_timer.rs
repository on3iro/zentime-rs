use crate::config::{create_config, Config};
use crate::input::TerminalInputThread;
use crate::notification::dispatch_notification;
use crate::view::TerminalRenderer;
use crate::TerminalEvent;
use std::time::Duration;
use zentime_rs_timer::events::AppAction;
use zentime_rs_timer::Timer;

use std::sync::mpsc::{self, RecvTimeoutError};

pub fn start_timer(config_path: &str) {
    let config: Config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    let (terminal_input_sender, terminal_input_receiver) = mpsc::channel();
    let (view_sender, view_receiver) = mpsc::channel();

    TerminalInputThread::spawn(terminal_input_sender);
    let render_thread_handle = TerminalRenderer::spawn(view_receiver, config.view);

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
                Some(AppAction::Quit)
            };

            // Handle app actions and hand them to the timer caller
            match terminal_input_receiver.recv_timeout(Duration::from_secs(1)) {
                Ok(AppAction::Quit) => handle_quit(),
                Ok(action) => Some(action),
                Err(RecvTimeoutError::Disconnected) => handle_quit(),
                _ => None,
            }
        }),
    )
    .init()
    .expect("Could not initialize timer");

    render_thread_handle.join().expect("Could not join threads");
}
