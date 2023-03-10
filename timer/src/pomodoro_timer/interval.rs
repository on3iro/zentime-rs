use std::marker::PhantomData;

use crate::{config::PomodoroTimerConfig, timer::TimerTickHandler, Timer, TimerAction};

use super::{
    long_break::LongBreak,
    on_end_handler::{OnEndHandler, OnTimerEnd},
    on_tick_handler::{OnTick, PomodoroActionHandler},
    short_break::ShortBreak,
    state::{Callbacks, PomodoroState, PomodoroTimer, PomodoroTimerState, ViewState},
};

/// Pomodoro timer state designating a focus interval
#[derive(Debug, Copy, Clone)]
pub struct Interval {}

impl PomodoroState for Interval {}

struct IntervalTickHandler {
    pomodoro_timer: PomodoroTimer<Interval>,
}

impl PomodoroActionHandler<Interval> for IntervalTickHandler {
    fn get_timer(&self) -> PomodoroTimer<Interval> {
        self.pomodoro_timer.clone()
    }
}

impl TimerTickHandler for IntervalTickHandler {
    fn call(&mut self, current_time: crate::timer::CurrentTime) -> Option<TimerAction> {
        let callbacks = self.pomodoro_timer.callbacks.clone();
        let state = self.pomodoro_timer.shared_state;

        let result = (callbacks.on_tick)(ViewState {
            is_break: false,
            is_postponed: false,
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

impl PomodoroTimer<Interval> {
    /// Creates a new pomodoro timer with the initial pomodoro state and returns it
    /// To actually run the timer and listen for input etc., [self.init] has to be called
    /// on the returned timer.
    pub fn new(config: PomodoroTimerConfig, on_timer_end: OnTimerEnd, on_tick: OnTick) -> Self {
        let shared_state = PomodoroTimerState {
            round: 1,
            postponed_count: 0,
        };

        Self {
            shared_state,
            config,
            callbacks: Callbacks {
                on_timer_end,
                on_tick,
            },
            marker: PhantomData,
        }
    }

    /// Runs the timer so that the inner timer loop is started and the [OnTick]
    /// closure is being called continously.
    /// NOTE: This does not mean that the timer starts counting.
    /// The internal [Timer] will be initialized in a paused state, waiting for
    /// a [TimerAction:PlayPause]-action (triggered in turn by a [PomodoroTimerAction::PlayPause])
    pub fn init(self) {
        let is_major_break = self.shared_state.round % self.config.intervals == 0;

        Timer::new(
            self.config.timer,
            Some(OnEndHandler {
                on_timer_end: self.callbacks.on_timer_end.clone(),
                state: self.shared_state,
                notification: "Good job, take a break!",
            }),
            Some(IntervalTickHandler {
                pomodoro_timer: self.clone(),
            }),
        )
        .init();

        Self::next(
            self.config,
            self.callbacks,
            self.shared_state,
            is_major_break,
        )
    }

    fn next(
        config: PomodoroTimerConfig,
        callbacks: Callbacks,
        shared_state: PomodoroTimerState,
        is_major_break: bool,
    ) {
        let state = PomodoroTimerState {
            postponed_count: 0,
            ..shared_state
        };

        if is_major_break {
            PomodoroTimer {
                shared_state: state,
                config,
                callbacks,
                marker: PhantomData::<LongBreak>,
            }
            .init()
        } else {
            PomodoroTimer {
                shared_state: state,
                config,
                callbacks,
                marker: PhantomData::<ShortBreak>,
            }
            .init()
        }
    }
}
