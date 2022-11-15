use crate::config::{create_config, Config};
use crate::input::TerminalInputThread;
use crate::notification::dispatch_notification;
use crate::view::TerminalRenderer;
use zentime_rs_timer::Timer;

use std::sync::mpsc;

// self.view_sender
// .send(TerminalEvent::View(ViewState {
// is_break: self.shared_state.is_break,
// round: self.shared_state.round,
// time: seconds_to_time(time),
// }))
// .context("View sender could not send")?;

// let action = match self
// .app_action_receiver
// .recv_timeout(Duration::from_secs(1))
// {
// Ok(action) => action,
// Err(RecvTimeoutError::Disconnected) => AppAction::Quit,
// _ => AppAction::None,
// };

// self.view_sender
// .send(TerminalEvent::View(ViewState {
// is_break: self.shared_state.is_break,
// round: self.shared_state.round,
// time: seconds_to_time(time),
// }))
// .context("View sender could not send")?;

// let action = match self
// .app_action_receiver
// .recv_timeout(Duration::from_secs(1))
// {
// Ok(action) => action,
// Err(RecvTimeoutError::Disconnected) => AppAction::Quit,
// _ => AppAction::None,
// };

pub fn start_timer(config_path: &str) {
    let config: Config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    let (terminal_input_sender, terminal_input_receiver) = mpsc::channel();
    let (view_sender, view_receiver) = mpsc::channel();

    TerminalInputThread::spawn(terminal_input_sender);
    let render_thread_handle = TerminalRenderer::spawn(view_receiver, config.view);

    Timer::new(
        terminal_input_receiver,
        view_sender,
        config.timers,
        Box::new(move |_, msg| {
            // We simply discard errors here for now...
            dispatch_notification(config.notifications, msg).ok();
        }),
    )
    .init()
    .expect("Could not initialize timer");

    render_thread_handle.join().expect("Could not join threads");
}
