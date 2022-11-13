use crate::config::NotificationConfig;
use crate::notification;
use crate::sound::{play, SoundFile};
use notify_rust::{Notification, NotificationHandle};

pub fn dispatch_notification(
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

fn send(message: &str) -> anyhow::Result<NotificationHandle> {
    let handle = Notification::new()
        .summary("\u{25EF} zentime")
        .body(message)
        .show()?;
    Ok(handle)
}
