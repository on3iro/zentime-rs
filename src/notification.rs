use notify_rust::{Notification, NotificationHandle};

pub fn send(message: &str) -> anyhow::Result<NotificationHandle> {
    let handle = Notification::new()
        .summary("\u{25EF} zentime")
        .body(message)
        .show()?;
    Ok(handle)
}
