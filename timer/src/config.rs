use serde::{Deserialize, Serialize};

/// Timer configuration which determines certain aspects of the timer,
/// like the duration of `intervals` and break lengths.
#[derive(Deserialize, Serialize, Clone, Copy)]
pub struct TimerConfig {
    /// Timer in seconds
    pub timer: u64,

    /// Minor break time in seconds
    pub minor_break: u64,

    /// Major break time in seconds
    pub major_break: u64,

    /// Intervals before major break
    pub intervals: u64,
}

impl Default for TimerConfig {
    fn default() -> Self {
        TimerConfig {
            timer: 1500,
            minor_break: 300,
            major_break: 900,
            intervals: 4,
        }
    }
}
