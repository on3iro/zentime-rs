use serde::{Deserialize, Serialize};
use zentime_rs_timer::timer::ViewState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerOutputAction {
    Timer(ViewState),
}
