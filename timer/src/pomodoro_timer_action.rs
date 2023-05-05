//! Action enum that can be passed to the timer on each tick to interact with it

/// Various control actions to transition into new states
#[derive(Debug, Copy, Clone)]
pub enum PomodoroTimerAction {
    /// NoOp
    None,

    /// Either start or pause the current timer
    PlayPause,

    /// Skip to the next timer (break or focus)
    Skip,

    /// Reset timer
    ResetTimer,

    /// Postpone a break
    PostponeBreak,

    /// Set current timer to a specific time in seconds
    SetTimer(u64),
}
