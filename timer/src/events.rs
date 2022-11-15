//! Various types that denote shared state and interactions

/// Information that can be shared  with the [Timer::view_sender]
pub struct ViewState {
    /// Denotes if the current timer is a break timer
    pub is_break: bool,

    /// Denotes the current interval round
    pub round: u64,

    /// Denotes the current time of the timer
    pub time: String,
}

/// Describes a message passed to the [Timer::view_sender]
pub enum TerminalEvent {
    /// Rendering information with a [ViewState]
    View(ViewState),

    /// The timer received an [AppAction::Quit] and forwards
    /// this information to the view
    Quit,
}

/// Various control actions to transition into new states
pub enum AppAction {
    /// Command the timer to stop and be dropped
    Quit,

    /// NoOp
    None,

    /// Either start or pause the current timer
    PlayPause,

    /// Skip to the next timer (break or focus)
    Skip,
}
