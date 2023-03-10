use std::marker::PhantomData;

use crate::{config::PomodoroTimerConfig, timer::TimerTickHandler, Timer, TimerAction};

use super::{
    long_break::LongBreak,
    on_end_handler::OnEndHandler,
    on_tick_handler::PomodoroActionHandler,
    state::{Callbacks, PomodoroState, PomodoroTimer, PomodoroTimerState, ViewState},
};

/// Pomodoro timer state designating a postponed long break
#[derive(Debug, Copy, Clone)]
pub struct PostponedLongBreak {}

impl PomodoroState for PostponedLongBreak {}

struct PostponeLongBreakTickHandler {
    pomodoro_timer: PomodoroTimer<PostponedLongBreak>,
}

impl PomodoroActionHandler<PostponedLongBreak> for PostponeLongBreakTickHandler {
    fn get_timer(&self) -> PomodoroTimer<PostponedLongBreak> {
        self.pomodoro_timer.clone()
    }
}

impl TimerTickHandler for PostponeLongBreakTickHandler {
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

impl PomodoroTimer<PostponedLongBreak> {
    /// Starts the timer loop on a `PomodoroTimer<PostponedLongBreak>`
    pub fn init(self) {
        Timer::new(
            self.config.postpone_timer,
            Some(OnEndHandler {
                on_timer_end: self.callbacks.on_timer_end.clone(),
                state: self.shared_state,
                notification: "Postpone done - back to break",
            }),
            Some(PostponeLongBreakTickHandler {
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
            marker: PhantomData::<LongBreak>,
        }
        .init();
    }
}
