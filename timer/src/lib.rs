#![warn(
    missing_docs,
    missing_copy_implementations,
    missing_debug_implementations
)]

//! Pomodoro/Productivity timer that can transition between various states ([Paused]/[Running]),
//! tracks intervals and can be configured.
//!
//! ## Example
//!
//! ```
//! use std::sync::mpsc::{self, RecvTimeoutError};
//! use std::sync::mpsc::{Receiver, Sender};
//! use std::thread;
//! use std::rc::Rc;
//! use std::time::Duration;
//! use zentime_rs_timer::config::PomodoroTimerConfig;
//! use zentime_rs_timer::pomodoro_timer_action::PomodoroTimerAction;
//! use zentime_rs_timer::pomodoro_timer::{ PomodoroTimer, TimerKind, ViewState };
//!
//!     let (terminal_input_sender, terminal_input_receiver): (Sender<PomodoroTimerAction>, Receiver<PomodoroTimerAction>) =
//!         mpsc::channel();
//!     let (view_sender, view_receiver): (Sender<ViewState>, Receiver<ViewState>) =
//!         mpsc::channel();
//!
//!     let config = PomodoroTimerConfig::default();
//!
//!     // Run timer in its own thread so it does not block the current one
//!     thread::spawn(move || {
//!         let timer = PomodoroTimer::new(
//!             config,
//!             Rc::new(move |state, msg, _| {
//!                 println!("{} {}", state.round, msg.unwrap());
//!             }),
//!             Rc::new(move |view_state| -> Option<PomodoroTimerAction> {
//!                 view_sender.send(view_state).unwrap();
//!
//!                 let input = terminal_input_receiver.recv_timeout(Duration::from_millis(100));
//!
//!                 match input {
//!                     Ok(action) => Some(action),
//!                     Err(RecvTimeoutError::Disconnected) => std::process::exit(0),
//!                     _ => None,
//!                 }
//!             }),
//!         );
//!
//!         timer.init();
//!     });
//!
//!     let action_jh = thread::spawn(move || {
//!         // Start the timer
//!         terminal_input_sender.send(PomodoroTimerAction::PlayPause).unwrap();
//!
//!         // Render current timer state three seconds in a row
//!         for _ in 0..3 {
//!             thread::sleep(Duration::from_millis(100));
//!             if let Ok(state) = view_receiver.recv() {
//!                 println!("{}", state.time)
//!             }
//!         }
//!
//!         # std::process::exit(0);
//!     });
//!
//!     action_jh.join().unwrap();
//! ```

pub use timer::Timer;
pub use timer_action::TimerAction;

pub mod config;
pub mod pomodoro_timer;
pub mod pomodoro_timer_action;
pub mod timer;
pub mod timer_action;
pub mod util;
