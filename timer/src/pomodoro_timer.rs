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
mod interval;
mod long_break;
mod on_end_handler;
mod on_tick_handler;
mod postponed_long_break;
mod postponed_short_break;
mod short_break;
mod state;

pub use state::{PomodoroTimer, ViewState};
