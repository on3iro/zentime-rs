//! OS-Notification and sound playback related functions.

use super::sound::{play, SoundFile};
use crate::config::NotificationConfig;
use anyhow::bail;
use log::error;
use notify_rust::{Notification, NotificationHandle};

/// Play a sound file and send an OS-notification.
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

/// Send a OS-notificaion
fn send(message: &str) -> anyhow::Result<NotificationHandle> {
    match Notification::new()
        .summary("\u{25EF} zentime")
        .body(message)
        .show()
    {
        Ok(handle) => Ok(handle),
        Err(error) => {
            // Currently show() will always return ok() (as per the definition of)
            // notify_rust. However if they API changes one day an we are indeed able to receive
            // errors, we wan't it to be logged in some way.
            error!("Error on notification: {:?}", error);
            bail!(error)
        }
    }
}
