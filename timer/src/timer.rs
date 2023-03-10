//! Timer implementation
//! The timer hast the ability to toggle playback and end.
//! It will call a given `on_tick` closure on every tick update and
//! an `on_timer_end`-closure, when it's done (either by receiving a [TimerAction::End]) or
//! when the internal timer is down to 0

use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display};

use crate::timer_action::TimerAction;
use crate::util::seconds_to_time;
use std::time::{Duration, Instant};

// NOTE: I tried to use the typestate approach, like it's described here:
// https://cliffle.com/blog/rust-typestate/

/// Information that will be handed to the [on_tick] closure continously
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentTime(String);

impl Display for CurrentTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

/// Empty trait implemented by structs (e.g. Paused, Running)
pub trait TimerState {}

/// State specific to a paused timer
#[derive(Clone, Copy, Debug)]
pub struct Paused {
    remaining_time: Duration,
}

/// State specific to a running timer
#[derive(Clone, Copy, Debug)]
pub struct Running {
    target_time: Instant,
}

impl TimerState for Paused {}
impl TimerState for Running {}

/// Handler which is called whenever a timer ends by running out
pub trait TimerEndHandler {
    /// Callback
    fn call(&mut self);
}

/// Handler which is called on each timer tick
pub trait TimerTickHandler {
    /// Callback
    fn call(&mut self, current_time: CurrentTime) -> Option<TimerAction>;
}

/// Timer which can either be in a paused state or a running state.
/// To instantiate the timer run `Timer::new()`.
/// To actually start it call `Timer::init()`
///
/// ## Example
///
/// ```
/// use zentime_rs_timer::timer::{Timer, TimerEndHandler, TimerTickHandler, CurrentTime};
/// use zentime_rs_timer::TimerAction;
/// use std::thread;
///
/// // Handler which is passed to our timer implementation
/// pub struct OnEndHandler { }
///
/// impl TimerEndHandler for OnEndHandler {
///     fn call(&mut self) {
///         println!("Hi from timer");
///     }
/// }
///
/// pub struct OnTickHandler {}
///
/// impl TimerTickHandler for OnTickHandler {
///     fn call(&mut self, current_time: CurrentTime) -> Option<TimerAction> {
///         println!("{}", current_time);
///         None
///     }
/// }
///
/// // Run timer in its own thread so it does not block the current one
/// thread::spawn(move || {
///     Timer::new(
///         10,
///         Some(OnEndHandler {}),
///         Some(OnTickHandler {})
///     )
///     .init();
/// });
/// ```
pub struct Timer<S: TimerState> {
    time: u64,

    /// Callback closure which is called at the end of each timer
    on_timer_end: Option<Box<dyn TimerEndHandler>>,

    /// Callback closure which is being run on each tick
    on_tick: Option<Box<dyn TimerTickHandler>>,

    /// Internal state data associated with a certain timer state (e.g. [Paused] or [Running])
    internal_state: S,
}

impl<S: TimerState + std::fmt::Debug> Debug for Timer<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Timer")
            .field("time", &self.time)
            .field("on_timer_end", &"[closure] without context")
            .field("internal_state", &self.internal_state)
            .field("on_tick", &"[closure] without context")
            .finish()
    }
}

impl<S: TimerState> Timer<S> {}

/// Implementation of the [Paused] state for [Timer]
impl Timer<Paused> {
    /// Creates a new timer in paused state.
    /// You have to call [Self::init()] to start the timer
    pub fn new<E, T>(time: u64, on_timer_end: Option<E>, on_tick: Option<T>) -> Self
    where
        E: TimerEndHandler + 'static,
        T: TimerTickHandler + 'static,
    {
        let remaining_time = Duration::from_secs(time);

        Self {
            time,
            on_timer_end: on_timer_end.map(|x| Box::new(x) as Box<dyn TimerEndHandler>),
            on_tick: on_tick.map(|x| Box::new(x) as Box<dyn TimerTickHandler>),
            internal_state: Paused { remaining_time },
        }
    }

    /// Puts the paused timer into a waiting state waiting for input (e.g. to unpause the timer
    /// and transition it into a running state).
    pub fn init(mut self) {
        loop {
            let time = self.internal_state.remaining_time.as_secs();

            let Some(ref mut callback) = self.on_tick else { continue };
            if let Some(action) = callback.call(CurrentTime(seconds_to_time(time))) {
                match action {
                    TimerAction::PlayPause => {
                        self.unpause();
                        break;
                    }

                    // Returns from the blocking loop, so that the calling code
                    // can resume execution
                    TimerAction::End => return,
                }
            }
        }
    }

    /// Transitions the paused timer into a running timer
    fn unpause(self) {
        Timer {
            on_timer_end: self.on_timer_end,
            on_tick: self.on_tick,
            time: self.time,
            internal_state: Running {
                target_time: Instant::now() + self.internal_state.remaining_time,
            },
        }
        .init()
    }
}

impl Timer<Running> {
    /// Transitions the running timer into a paused timer state and calls `init()` on_interval_end
    /// it, so that the new timer is ready to receive an [TimerInputAction]
    fn pause(self) {
        Timer {
            time: self.time,
            on_tick: self.on_tick,
            on_timer_end: self.on_timer_end,
            internal_state: Paused {
                remaining_time: self.internal_state.target_time - Instant::now(),
            },
        }
        .init();
    }

    /// Runs the timer and awaits input.
    /// Depending on the input [TimerInputAction] the timer might transition into a paused state or skip to the next interval.
    fn init(mut self) {
        while self.internal_state.target_time > Instant::now() {
            let time = (self.internal_state.target_time - Instant::now()).as_secs();

            let Some(ref mut callback) = self.on_tick else { continue };
            if let Some(action) = callback.call(CurrentTime(seconds_to_time(time))) {
                match action {
                    TimerAction::PlayPause => {
                        return self.pause();
                    }

                    // Returns from the blocking loop, so that the calling code
                    // can resume execution
                    TimerAction::End => return,
                }
            }
        }

        if let Some(mut on_timer_end) = self.on_timer_end {
            on_timer_end.call()
        }
    }
}
