use zentime_rs_timer::timer::ViewState;

/// Describes a message passed to the [Timer::view_sender]
#[derive(Debug)]
pub enum TerminalEvent {
    /// Rendering information with a [ViewState]
    View(ViewState),

    /// The timer received an [AppAction::Quit] and forwards
    /// this information to the view
    Quit,
}
