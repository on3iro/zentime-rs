use std::rc::Rc;

use crate::timer::TimerEndHandler;

use super::state::PomodoroTimerState;

/// Describes pomodoro timer kind
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TimerKind {
    /// Always used when the current timer is not a break timer
    Interval,

    /// Only used for breaks
    Break,
}

pub type OnTimerEnd = Rc<dyn Fn(PomodoroTimerState, Option<&str>, TimerKind)>;

/// Handler which is passed to our timer implementation
pub struct OnEndHandler {
    pub on_timer_end: OnTimerEnd,
    pub state: PomodoroTimerState,
    pub notification: Option<&'static str>,
    pub kind: TimerKind,
}

impl TimerEndHandler for OnEndHandler {
    fn call(&mut self) {
        (self.on_timer_end)(self.state, self.notification, self.kind);
    }
}
