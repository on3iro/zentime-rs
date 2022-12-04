use super::sound::{play, SoundFile};
use crate::config::NotificationConfig;
use notify_rust::{Notification, NotificationHandle};

pub fn dispatch_notification(
    config: NotificationConfig,
    notification_string: &str,
) -> anyhow::Result<()> {
    if config.enable_bell {
        play(SoundFile::Bell, config.volume);
    }

    if config.show_notification {
        send(notification_string)?;
    }
    Ok(())
}

fn send(message: &str) -> anyhow::Result<NotificationHandle> {
    let handle = Notification::new()
        .summary("\u{25EF} zentime")
        .body(message)
        .show()?;
    Ok(handle)
}
