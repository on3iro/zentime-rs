//! Configuration of a [Timer]
use serde::{Deserialize, Serialize};

/// Timer configuration which determines certain aspects of the timer,
/// like the duration of `intervals` and break lengths.
#[derive(Debug, Deserialize, Serialize, Clone, Copy)]
pub struct PomodoroTimerConfig {
    /// Timer in seconds
    pub timer: u64,

    /// Minor break time in seconds
    pub minor_break: u64,

    /// Major break time in seconds
    pub major_break: u64,

    /// Intervals before major break
    pub intervals: u64,

    /// Determines how often a break may be postponed.
    /// A value of 0 denotes, that postponing breaks is not allowed and the feature is
    /// disabled.
    pub postpone_limit: u16,

    /// Determines how long each postpone timer runs (in seconds)
    pub postpone_timer: u64,
}

impl Default for PomodoroTimerConfig {
    fn default() -> Self {
        PomodoroTimerConfig {
            timer: 1500,
            minor_break: 300,
            major_break: 900,
            intervals: 4,
            postpone_limit: 0,
            postpone_timer: 300,
        }
    }
}
