use notify_rust::{Notification, NotificationHandle};

pub fn send(message: &str) -> NotificationHandle {
    Notification::new()
        .summary("\u{25EF} zentime")
        .body(message)
        .show()
        .unwrap()
}
