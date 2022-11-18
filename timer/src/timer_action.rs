//! Action enum that can be passed to the timer on each tick to interact with it

/// Various control actions to transition into new states
#[derive(Debug)]
pub enum TimerAction {
    /// Command the timer to stop and be dropped
    Quit,

    /// NoOp
    None,

    /// Either start or pause the current timer
    PlayPause,

    /// Skip to the next timer (break or focus)
    Skip,
}
