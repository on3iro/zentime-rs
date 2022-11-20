//! Action enum that can be passed to the timer on each tick to interact with it

/// Various control actions to transition into new states
#[derive(Debug, Copy, Clone)]
pub enum TimerInputAction {
    /// NoOp
    None,

    /// Either start or pause the current timer
    PlayPause,

    /// Skip to the next timer (break or focus)
    Skip,
}
