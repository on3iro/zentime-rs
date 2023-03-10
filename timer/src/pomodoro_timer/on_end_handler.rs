use std::rc::Rc;

use crate::timer::TimerEndHandler;

use super::state::PomodoroTimerState;

pub type OnTimerEnd = Rc<dyn Fn(PomodoroTimerState, &str)>;

/// Handler which is passed to our timer implementation
pub struct OnEndHandler {
    pub on_timer_end: OnTimerEnd,
    pub state: PomodoroTimerState,
    pub notification: &'static str,
}

impl TimerEndHandler for OnEndHandler {
    fn call(&mut self) {
        (self.on_timer_end)(self.state, self.notification);
    }
}
