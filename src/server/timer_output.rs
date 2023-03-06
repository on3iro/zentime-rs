use serde::{Deserialize, Serialize};
use zentime_rs_timer::pomodoro_timer::ViewState;

/// Carries the timer state as view state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerOutputAction {
    Timer(ViewState),
}
