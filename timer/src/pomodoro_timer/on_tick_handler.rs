use std::rc::Rc;

use crate::{pomodoro_timer_action::PomodoroTimerAction, TimerAction};

use super::{
    interval::Interval,
    state::{PomodoroState, PomodoroTimer, ViewState},
};

pub type OnTick = Rc<dyn Fn(ViewState) -> Option<PomodoroTimerAction>>;

pub struct PostponeHandlerConfig {
    pub postpone_limit: u16,
    pub postponed_count: u16,
}

pub trait PomodoroActionHandler<S: PomodoroState> {
    fn can_postpone(postpone_config: PostponeHandlerConfig) -> bool {
        let PostponeHandlerConfig {
            postpone_limit,
            postponed_count,
        } = postpone_config;
        postpone_limit > 0 && postponed_count < postpone_limit
    }

    fn get_timer(&self) -> PomodoroTimer<S>;

    fn handle_action(&self, action: PomodoroTimerAction) -> Option<TimerAction> {
        let timer = PomodoroActionHandler::<S>::get_timer(self);

        match action {
            PomodoroTimerAction::PlayPause => Some(TimerAction::PlayPause),
            PomodoroTimerAction::Skip => Some(TimerAction::End),

            PomodoroTimerAction::ResetTimer => {
                PomodoroTimer::<Interval>::reset(timer.config, timer.callbacks).init();
                None
            }

            PomodoroTimerAction::SetTimer(time) => Some(TimerAction::SetTimer(time)),

            _ => None,
        }
    }
}
