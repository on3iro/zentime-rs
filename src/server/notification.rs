//! OS-Notification and sound playback related functions.

use super::sound::{play, AudioPlaybackError};
use crate::config::NotificationConfig;
use anyhow::bail;
use log::error;
use notify_rust::{Notification, NotificationHandle};
use rand::{seq::SliceRandom, thread_rng};
use std::fmt::Write;
use thiserror::Error;

/// Something went wrong during notification dispatch
#[derive(Debug, Error)]
pub enum NotificationDispatchError {
    /// Denotes that the given [SoundFile] could not be decoded
    #[error("Could not play sound file")]
    SoundPlayback(#[from] AudioPlaybackError),

    /// Denotes that something went wrong while zentime tried to send
    /// a system notification.
    /// NOTE: This case should currently not happen, because the underlying
    /// call to the [notify_rust] library will always return with `Ok`
    #[error("Could not send OS notification")]
    OperatingSystemNotification(#[from] anyhow::Error),
}

/// Play a sound file and send an OS-notification.
pub fn dispatch_notification(
    config: NotificationConfig,
    notification_string: Option<&str>,
    should_show_suggestion: bool,
) -> Result<(), NotificationDispatchError> {
    if config.enable_bell {
        play(config.sound_file, config.volume)?;
    }

    if !config.show_notification || notification_string.is_none() {
        return Ok(());
    };

    let mut notification = notification_string.unwrap().to_string();

    if !should_show_suggestion {
        send(&notification)?;
        return Ok(());
    }

    let suggestions = match config.break_suggestions {
        Some(suggestions) => suggestions,
        None => vec![],
    };

    let random_suggestion = suggestions.choose(&mut thread_rng());

    if let Some(suggestion) = random_suggestion {
        if let Err(error) = write!(notification, "\n\n{}", suggestion) {
            error!("Could not concatenate random suggestion to notification message");
            return Err(NotificationDispatchError::OperatingSystemNotification(
                anyhow::Error::new(error),
            ));
        };
    }

    send(&notification)?;

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
