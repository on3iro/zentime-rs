//! Implementation of the actual timer logic

use crate::config::TimerConfig;
use crate::timer_action::TimerAction;
use crate::util::seconds_to_time;
use std::time::{Duration, Instant};

// NOTE: I tried to use the typestate approach, like it's described here:
// https://cliffle.com/blog/rust-typestate/

// TODO
// use thiserror crate

/// Information that can be shared  with the [Timer::view_sender]
#[derive(Debug)]
pub struct ViewState {
    /// Denotes if the current timer is a break timer
    pub is_break: bool,

    /// Denotes the current interval round
    pub round: u64,

    /// Denotes the current time of the timer
    pub time: String,
}

/// Empty trait implemented by structs (e.g. Paused, Running)
pub trait TimerState {}

/// State specific to a paused timer
pub struct Paused {
    remaining_time: Duration,
}

/// State specific to a running timer
pub struct Running {
    target_time: Instant,
}

impl TimerState for Paused {}
impl TimerState for Running {}

/// State which is shared between timers when they transition from one timer state to the
/// next. Some of its information is also being shared with the `view_sender`.
#[derive(Clone, Copy)]
pub struct TimerStateData {
    /// State that denotes if the current timer is a break timer.
    /// If set to `false` the current timer is a regular focus interval timer.
    pub is_break: bool,

    /// The current pomodoro round.
    /// This is incremented after each break.
    pub round: u64,
}

// TODO return results on these (and use thiserror instead of anyhow)
type OnTimerEnd = Box<dyn Fn(TimerStateData, &str)>;
type OnTick = Box<dyn FnMut(ViewState) -> Option<TimerAction>>;

/// Timer which can either be in a paused state or a running state.
/// To instantiate the timer run `Timer::new()`.
/// To actually start it call `Timer::init()`
/// This puts the timer into a paused state waiting for [TimerAction](TimerAction)s to be sent down
/// the input channel. For example an [TimerAction::PlayPause](TimerAction::PlayPause) starts the timer.
///
/// ## Example
///
/// ```
/// use zentime_rs_timer::config::{TimerConfig};
/// use zentime_rs_timer::timer::{Timer};
/// use std::thread;
///
/// // Run timer in its own thread so it does not block the current one
/// thread::spawn(move || {
///     Timer::new(
///         TimerConfig::default(),
///         Box::new(move |state, msg| {
///             println!("{} {}", state.round, msg);
///         }),
///         Box::new(move |view_state| {
///             println!("{:?}", view_state);
///             None
///         }),
///     )
///     .init()
///     .expect("Could not initialize timer");
/// });
/// ```
pub struct Timer<S: TimerState> {
    /// Config describin how long intervals are etc.
    config: TimerConfig,

    // TODO make this optional
    /// Callback closure which is called at the end of each timer
    on_interval_end: OnTimerEnd,

    /// State shared between timers when they transition into each other
    shared_state: Box<TimerStateData>,

    /// Internal state data associated with a certain timer state (e.g. [Paused] or [Running])
    internal_state: S,

    /// Callback closure which is being run on each tick
    on_tick: OnTick,
}

impl<S: TimerState> Timer<S> {
    /// Transitions a timer from one timer to the next.
    /// The next timer will either be a break timer or a regular timer.
    /// Either way it will be initalized in a paused state.
    /// If [is_timer_end] is set to true, the [Timer::on_interval_end] will be called.
    fn next(self, is_timer_end: bool) -> anyhow::Result<()> {
        let is_major_break = self.shared_state.round % self.config.intervals == 0;
        let shared_state = self.shared_state.clone();

        let (new_timer, notification_string) = if self.shared_state.is_break {
            (self.new_timer(), "Break is over")
        } else {
            (
                self.new_break_timer(is_major_break),
                "Good job! Take a break",
            )
        };

        if is_timer_end {
            (new_timer.on_interval_end)(*shared_state, notification_string);
        }

        new_timer.init()
    }

    /// Creates a new break timer whose length will either be that of a
    /// [TimerConfig::minor_break] or [TimerConfig::major_break]
    fn new_break_timer(self, is_major_break: bool) -> Timer<Paused> {
        let break_length = if is_major_break {
            self.config.major_break
        } else {
            self.config.minor_break
        };

        Timer {
            on_interval_end: self.on_interval_end,
            on_tick: self.on_tick,
            config: self.config,
            shared_state: Box::new(TimerStateData {
                round: self.shared_state.round,
                is_break: true,
            }),
            internal_state: Paused {
                remaining_time: Duration::from_secs(break_length),
            },
        }
    }

    /// Creates a new regular interval timer
    fn new_timer(self) -> Timer<Paused> {
        let remaining_time = Duration::from_secs(self.config.timer);

        Timer {
            on_interval_end: self.on_interval_end,
            on_tick: self.on_tick,
            config: self.config,
            shared_state: Box::new(TimerStateData {
                round: self.shared_state.round + 1,
                is_break: false,
            }),
            internal_state: Paused { remaining_time },
        }
    }
}

/// Implementation of the [Paused] state for [Timer]
impl Timer<Paused> {
    /// Creates a new timer in paused state.
    /// You have to call [Self::init()] to make the timer listen for inputs on its
    /// `input_receiver` so that it can actually be started.
    pub fn new(config: TimerConfig, on_interval_end: OnTimerEnd, on_tick: OnTick) -> Self {
        let remaining_time = Duration::from_secs(config.timer);

        Self {
            config,
            on_interval_end,
            on_tick,
            shared_state: Box::new(TimerStateData {
                round: 1,
                is_break: false,
            }),
            internal_state: Paused { remaining_time },
        }
    }

    /// Puts the paused timer into a waiting state waiting for input (e.g. to unpause the timer
    /// and transition it into a running state).
    pub fn init(mut self) -> anyhow::Result<()> {
        loop {
            let time = self.internal_state.remaining_time.as_secs();

            if let Some(action) = (self.on_tick)(ViewState {
                is_break: self.shared_state.is_break,
                round: self.shared_state.round,
                time: seconds_to_time(time),
            }) {
                match action {
                    TimerAction::Quit => {
                        return Ok(());
                    }
                    TimerAction::PlayPause => {
                        self.unpause()?;
                        break;
                    }
                    TimerAction::Skip => {
                        return self.next(false);
                    }

                    TimerAction::None => {}
                }
            }
        }

        Ok(())
    }

    /// Transitions the paused timer into a running timer
    fn unpause(self) -> anyhow::Result<()> {
        Timer {
            on_interval_end: self.on_interval_end,
            on_tick: self.on_tick,
            config: self.config,
            shared_state: self.shared_state,
            internal_state: Running {
                target_time: Instant::now() + self.internal_state.remaining_time,
            },
        }
        .start()
    }
}

impl Timer<Running> {
    /// Transitions the running timer into a paused timer state and calls `init()` on_interval_end
    /// it, so that the new timer is ready to receive an [TimerAction]
    fn pause(self) -> anyhow::Result<()> {
        Timer {
            config: self.config,
            on_tick: self.on_tick,
            on_interval_end: self.on_interval_end,
            shared_state: self.shared_state,
            internal_state: Paused {
                remaining_time: self.internal_state.target_time - Instant::now(),
            },
        }
        .init()
    }

    /// Runs the timer and awaits input.
    /// Depending on the input [TimerAction] the timer might, Quit (and inform [Self::view_sender] about this),
    /// transition into a paused state or jump to the next interval.
    fn start(mut self) -> anyhow::Result<()> {
        while self.internal_state.target_time > Instant::now() {
            let time = (self.internal_state.target_time - Instant::now()).as_secs();

            if let Some(action) = (self.on_tick)(ViewState {
                is_break: self.shared_state.is_break,
                round: self.shared_state.round,
                time: seconds_to_time(time),
            }) {
                match action {
                    TimerAction::Quit => {
                        return Ok(());
                    }
                    TimerAction::PlayPause => {
                        return self.pause();
                    }
                    TimerAction::Skip => {
                        return self.next(false);
                    }
                    TimerAction::None => {}
                }
            }
        }

        self.next(true)
    }
}
