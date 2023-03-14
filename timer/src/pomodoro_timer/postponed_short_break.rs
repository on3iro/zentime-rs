use std::marker::PhantomData;

use crate::{
    config::PomodoroTimerConfig,
    timer::{Running, TimerTickHandler},
    Timer, TimerAction,
};

use super::{
    on_end_handler::OnEndHandler,
    on_tick_handler::PomodoroActionHandler,
    short_break::ShortBreak,
    state::{Callbacks, PomodoroState, PomodoroTimer, PomodoroTimerState, ViewState},
};

/// Pomodoro timer state designating a postponed short break
#[derive(Debug, Copy, Clone)]
pub struct PostponedShortBreak {}

impl PomodoroState for PostponedShortBreak {}

struct PostponeShortBreakTickHandler {
    pomodoro_timer: PomodoroTimer<PostponedShortBreak>,
}

impl PomodoroActionHandler<PostponedShortBreak> for PostponeShortBreakTickHandler {
    fn get_timer(&self) -> PomodoroTimer<PostponedShortBreak> {
        self.pomodoro_timer.clone()
    }
}

impl TimerTickHandler for PostponeShortBreakTickHandler {
    fn call(&mut self, current_time: crate::timer::CurrentTime) -> Option<TimerAction> {
        let callbacks = self.pomodoro_timer.callbacks.clone();
        let state = self.pomodoro_timer.shared_state;

        let result = (callbacks.on_tick)(ViewState {
            is_break: false,
            is_postponed: true,
            postpone_count: state.postponed_count,
            round: state.round,
            time: current_time.to_string(),
        });

        if let Some(action) = result {
            self.handle_action(action)
        } else {
            None
        }
    }
}

impl PomodoroTimer<PostponedShortBreak> {
    pub(crate) fn init(self) {
        Timer::<Running>::new(
            self.config.postpone_timer,
            Some(OnEndHandler {
                on_timer_end: self.callbacks.on_timer_end.clone(),
                state: self.shared_state,
                notification: "Postpone done - back to break",
            }),
            Some(PostponeShortBreakTickHandler {
                pomodoro_timer: self.clone(),
            }),
        )
        .init();

        Self::next(self.config, self.callbacks, self.shared_state)
    }

    fn next(config: PomodoroTimerConfig, callbacks: Callbacks, shared_state: PomodoroTimerState) {
        PomodoroTimer {
            shared_state,
            config,
            callbacks,
            marker: PhantomData::<ShortBreak>,
        }
        .init();
    }
}
