//! Terminal event handled by a client

use zentime_rs_timer::pomodoro_timer::ViewState;

/// Describes a message passed from a connection to the [TerminalOutputTask]
#[derive(Debug)]
pub enum TerminalEvent {
    /// Rendering information with a [ViewState]
    View(ViewState),

    /// The timer received an [AppAction::Quit] and forwards
    /// this information to the view
    Quit {
        /// Optinal message to display on quit
        msg: Option<String>,

        /// Determines if the quit should happen with an error display
        error: bool,
    },
}
