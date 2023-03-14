use super::{interval::Interval, on_end_handler::OnTimerEnd, on_tick_handler::OnTick};
use crate::config::PomodoroTimerConfig;
use serde::{Deserialize, Serialize};
use std::{fmt::Debug, marker::PhantomData};

/// General trait describing the various states a pomodoro timer can be in
pub trait PomodoroState {}

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

    /// Denotes if the timer is currently paused
    pub is_paused: bool,
}

#[derive(Clone)]
pub struct Callbacks {
    pub on_timer_end: OnTimerEnd,
    pub on_tick: OnTick,
}

impl Debug for Callbacks {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PomodoroTimer")
            .field("on_timer_end", &"callback closure")
            .field("on_tick", &"callback closure")
            .finish()
    }
}

/// A Pomodoro-timer instance
#[derive(Clone)]
pub struct PomodoroTimer<S: PomodoroState> {
    /// User configuration of the pomodoro timer
    pub config: PomodoroTimerConfig,

    /// Callback handlers
    pub callbacks: Callbacks,

    /// State which is shared between PomodoroTimer-transitions
    pub shared_state: PomodoroTimerState,

    /// Marker designating in which typestate we are currently in
    pub marker: PhantomData<S>,
}

impl<S: PomodoroState + Debug> Debug for PomodoroTimer<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PomodoroTimer")
            .field("config", &self.config)
            .field("on_timer_end", &"[closure] without context")
            .field("on_tick", &"[closure] without context")
            .finish()
    }
}

impl<S: PomodoroState> PomodoroTimer<S> {
    /// Resets the pomodoro timer to the very first interval
    pub fn reset(config: PomodoroTimerConfig, callbacks: Callbacks) -> PomodoroTimer<Interval> {
        PomodoroTimer::new(config, callbacks.on_timer_end, callbacks.on_tick)
    }
}
