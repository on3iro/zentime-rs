use crate::config::NotificationConfig;
use crate::config::{create_config, Config};
use crate::input::TerminalInputThread;
use crate::notification;
use crate::sound::{play, SoundFile};
use crate::view::TerminalRenderer;
use zentime_rs_timer::Timer;

use std::sync::mpsc;

fn dispatch_notification(
    config: NotificationConfig,
    notification_string: &str,
) -> anyhow::Result<()> {
    if config.enable_bell {
        play(SoundFile::Bell, config.volume);
    }

    if config.show_notification {
        notification::send(notification_string)?;
    }
    Ok(())
}

pub fn start_timer(config_path: &str) {
    let config: Config = create_config(config_path)
        .extract()
        .expect("Could not create config");

    let (terminal_input_sender, terminal_input_receiver) = mpsc::channel();
    let (view_sender, view_receiver) = mpsc::channel();

    TerminalInputThread::spawn(terminal_input_sender);
    let render_thread_handle = TerminalRenderer::spawn(view_receiver);

    Timer::new(
        terminal_input_receiver,
        view_sender,
        config.timers,
        |_, msg| {
            // We simply discard errors here for now...
            dispatch_notification(config.notifications, msg).ok();
        },
    )
    .init()
    .expect("Could not initialize timer");

    render_thread_handle.join().expect("Could not join threads");
}
