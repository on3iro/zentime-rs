//! Pomodoro timer implementation.
//! When instantiated this runs instances of [Timer] internally and allows the transitioning
//! between various states like [Interval], [ShortBreak] or [LongBreak].
//!
//! To communicate with "the outside world" two distinct closures are used:
//!
//! [OnTimerEnd] will be called whenever the internal timer has ended by reaching 0.
//! This closure won't run on other occasions, like [PomodoroTimerAction::Skip], though.
//!
//! [OnTick] will be called on every tick of a running internal timer and can be used to
//! reveive the current timer state and to send [PomodoroTimerActions].
//!
use std::{fmt::Debug, marker::PhantomData, rc::Rc};

use serde::{Deserialize, Serialize};

use crate::{
    config::PomodoroTimerConfig, pomodoro_timer_action::PomodoroTimerAction, Timer, TimerAction,
};

/// General trait describing the various states a pomodoro timer can be in
pub trait PomodoroState {}

/// Pomodoro timer state designating a short break
#[derive(Debug, Copy, Clone)]
pub struct ShortBreak {}

/// Pomodoro timer state designating a long break
#[derive(Debug, Copy, Clone)]
pub struct LongBreak {}

/// Pomodoro timer state designating a focus interval
#[derive(Debug, Copy, Clone)]
pub struct Interval {}

/// Pomodoro timer state designating a postponed long break
#[derive(Debug, Copy, Clone)]
pub struct PostponedLongBreak {}

/// Pomodoro timer state designating a postponed short break
#[derive(Debug, Copy, Clone)]
pub struct PostponedShortBreak {}

impl PomodoroState for ShortBreak {}
impl PomodoroState for LongBreak {}
impl PomodoroState for Interval {}
impl PomodoroState for PostponedLongBreak {}
impl PomodoroState for PostponedShortBreak {}

/// State which is shared between timers when they transition from one timer state to the
/// next. Some of its information is also being shared with the [OnTick] closure.
#[derive(Clone, Copy, Debug)]
pub struct PomodoroTimerState {
    /// The current pomodoro round.
    /// This is incremented after each break.
    pub round: u64,

    /// Times the current break has been postponed - if a limit for postponing
    /// has been set this will be used to determine if postponing is possible or not.
    pub postponed_count: u16,
}

/// Information that will be handed to the [on_tick] closure continously
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewState {
    /// Denotes if the current timer is a break timer
    pub is_break: bool,

    /// Denotes if the timer is currently in a postponed state or not
    pub is_postponed: bool,

    /// Denotes how often the current timer has already been postponed
    pub postpone_count: u16,

    /// Denotes the current interval round
    pub round: u64,

    /// Denotes the current time of the timer
    pub time: String,
}

type OnTimerEnd = Rc<dyn Fn(PomodoroTimerState, &str)>;
type OnTick = Rc<dyn Fn(ViewState) -> Option<PomodoroTimerAction>>;

#[derive(Clone)]
struct Callbacks {
    on_timer_end: OnTimerEnd,
    on_tick: OnTick,
}

/// A Pomodoro-timer instance
pub struct PomodoroTimer<S: PomodoroState> {
    config: PomodoroTimerConfig,
    callbacks: Callbacks,
    shared_state: PomodoroTimerState,
    marker: PhantomData<S>,
}

impl<S: PomodoroState + std::fmt::Debug> Debug for PomodoroTimer<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PomodoroTimer")
            .field("config", &self.config)
            .field("on_timer_end", &"[closure] without context")
            .field("on_tick", &"[closure] without context")
            .finish()
    }
}

impl<S: PomodoroState> PomodoroTimer<S> {
    fn reset(config: PomodoroTimerConfig, callbacks: Callbacks) -> PomodoroTimer<Interval> {
        PomodoroTimer::new(config, callbacks.on_timer_end, callbacks.on_tick)
    }

    fn can_postpone(config: PomodoroTimerConfig, state: PomodoroTimerState) -> bool {
        config.postpone_limit > 0 && state.postponed_count < config.postpone_limit
    }

    fn handle_action(
        action: PomodoroTimerAction,
        config: PomodoroTimerConfig,
        callbacks: Callbacks,
    ) -> Option<TimerAction> {
        match action {
            PomodoroTimerAction::PlayPause => Some(TimerAction::PlayPause),
            PomodoroTimerAction::Skip => Some(TimerAction::End),

            PomodoroTimerAction::ResetTimer => {
                PomodoroTimer::<Interval>::reset(config, callbacks).init();
                None
            }

            _ => None,
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
        let on_tick_callbacks = self.callbacks.clone();
        let on_timer_end = self.callbacks.on_timer_end.clone();
        let is_major_break = &self.shared_state.round % self.config.intervals == 0;

        Timer::new(
            self.config.timer,
            Box::new(move || {
                let state = self.shared_state.clone();
                (on_timer_end)(state, "Good job, take a break!");
            }),
            Box::new(move |current_time| {
                if let Some(action) = (on_tick_callbacks.on_tick)(ViewState {
                    is_break: false,
                    is_postponed: false,
                    postpone_count: self.shared_state.postponed_count,
                    round: self.shared_state.round,
                    time: current_time.to_string(),
                }) {
                    Self::handle_action(action, self.config, on_tick_callbacks.clone())
                } else {
                    None
                }
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

impl PomodoroTimer<ShortBreak> {
    fn init(self) {
        let on_tick_callbacks = self.callbacks.clone();
        let on_timer_end = self.callbacks.on_timer_end.clone();

        let next_shared_state = PomodoroTimerState {
            round: self.shared_state.round + 1,
            postponed_count: self.shared_state.postponed_count,
        };

        Timer::new(
            self.config.minor_break,
            Box::new(move || {
                let state = self.shared_state.clone();
                (on_timer_end)(state, "Break is over");
            }),
            Box::new(move |current_time| {
                if let Some(action) = (on_tick_callbacks.on_tick)(ViewState {
                    is_break: true,
                    is_postponed: false,
                    postpone_count: self.shared_state.postponed_count,
                    round: self.shared_state.round,
                    time: current_time.to_string(),
                }) {
                    if let PomodoroTimerAction::PostponeBreak = action {
                        if !Self::can_postpone(self.config, self.shared_state) {
                            return None;
                        }

                        let state = PomodoroTimerState {
                            postponed_count: self.shared_state.postponed_count + 1,
                            ..self.shared_state
                        };

                        Self::postpone(self.config, on_tick_callbacks.clone(), state);

                        return None;
                    }

                    Self::handle_action(action, self.config, on_tick_callbacks.clone())
                } else {
                    None
                }
            }),
        )
        .init();

        Self::next(self.config, self.callbacks, next_shared_state)
    }

    fn postpone(
        config: PomodoroTimerConfig,
        callbacks: Callbacks,
        shared_state: PomodoroTimerState,
    ) {
        PomodoroTimer {
            shared_state,
            config,
            callbacks,
            marker: PhantomData::<PostponedShortBreak>,
        }
        .init();
    }

    fn next(config: PomodoroTimerConfig, callbacks: Callbacks, shared_state: PomodoroTimerState) {
        PomodoroTimer {
            shared_state,
            config,
            callbacks,
            marker: PhantomData::<Interval>,
        }
        .init();
    }
}

impl PomodoroTimer<LongBreak> {
    fn init(self) {
        let on_tick_callbacks = self.callbacks.clone();
        let on_timer_end = self.callbacks.on_timer_end.clone();

        let next_shared_state = PomodoroTimerState {
            round: self.shared_state.round + 1,
            postponed_count: self.shared_state.postponed_count,
        };

        Timer::new(
            self.config.major_break,
            Box::new(move || {
                let state = self.shared_state.clone();
                (on_timer_end)(state, "Break is over");
            }),
            Box::new(move |current_time| {
                if let Some(action) = (on_tick_callbacks.on_tick)(ViewState {
                    is_break: true,
                    is_postponed: false,
                    postpone_count: self.shared_state.postponed_count,
                    round: self.shared_state.round,
                    time: current_time.to_string(),
                }) {
                    if let PomodoroTimerAction::PostponeBreak = action {
                        if !Self::can_postpone(self.config, self.shared_state) {
                            return None;
                        }

                        let state = PomodoroTimerState {
                            postponed_count: self.shared_state.postponed_count + 1,
                            ..self.shared_state
                        };

                        Self::postpone(self.config, on_tick_callbacks.clone(), state);

                        return None;
                    }

                    Self::handle_action(action, self.config, on_tick_callbacks.clone())
                } else {
                    None
                }
            }),
        )
        .init();

        Self::next(self.config, self.callbacks, next_shared_state)
    }

    fn postpone(
        config: PomodoroTimerConfig,
        callbacks: Callbacks,
        shared_state: PomodoroTimerState,
    ) {
        PomodoroTimer {
            shared_state,
            config,
            callbacks,
            marker: PhantomData::<PostponedLongBreak>,
        }
        .init();
    }

    fn next(config: PomodoroTimerConfig, callbacks: Callbacks, shared_state: PomodoroTimerState) {
        PomodoroTimer {
            shared_state,
            config,
            callbacks,
            marker: PhantomData::<Interval>,
        }
        .init();
    }
}

impl PomodoroTimer<PostponedLongBreak> {
    fn init(self) {
        let on_tick_callbacks = self.callbacks.clone();
        let on_end = self.callbacks.on_timer_end.clone();

        Timer::new(
            self.config.postpone_timer,
            Box::new(move || (on_end)(self.shared_state, "Postpone done - back to break")),
            Box::new(move |current_time| {
                if let Some(action) = (on_tick_callbacks.on_tick)(ViewState {
                    is_break: false,
                    is_postponed: true,
                    postpone_count: self.shared_state.postponed_count,
                    round: self.shared_state.round,
                    time: current_time.to_string(),
                }) {
                    Self::handle_action(action, self.config, on_tick_callbacks.clone())
                } else {
                    None
                }
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

impl PomodoroTimer<PostponedShortBreak> {
    fn init(self) {
        let on_tick_callbacks = self.callbacks.clone();
        let on_end = self.callbacks.on_timer_end.clone();

        Timer::new(
            self.config.postpone_timer,
            Box::new(move || (on_end)(self.shared_state, "Postpone done - back to break")),
            Box::new(move |current_time| {
                if let Some(action) = (on_tick_callbacks.on_tick)(ViewState {
                    is_break: false,
                    is_postponed: true,
                    postpone_count: self.shared_state.postponed_count,
                    round: self.shared_state.round,
                    time: current_time.to_string(),
                }) {
                    Self::handle_action(action, self.config, on_tick_callbacks.clone())
                } else {
                    None
                }
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
