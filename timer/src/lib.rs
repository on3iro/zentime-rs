#![warn(missing_docs)]
//! Pomodoro/Productivity timer that can transition between various states ([Paused]/[Running]),
//! tracks intervals and can be configured.
//!
//! ## Example
//!
//! ```
//! use zentime_rs_timer::config::{TimerConfig};
//! use zentime_rs_timer::events::{TerminalEvent, AppAction};
//! use zentime_rs_timer::timer::{Timer};
//! use std::time::Duration;
//! use std::sync::mpsc;
//! use std::thread;
//! use std::sync::mpsc::{Sender, Receiver};
//!
//! let (terminal_input_sender, terminal_input_receiver): (Sender<AppAction>, Receiver<AppAction>) =
//!     mpsc::channel();
//! let (view_sender, view_receiver): (Sender<TerminalEvent>, Receiver<TerminalEvent>) =
//!     mpsc::channel();
//!
//! // ...Do something with the terminal_input_sender and view_receiver
//! // e.g. have a thread handle keyboard input and send AppActions and
//! // have another thread render a TUI (terminal user interface) receiving
//! // the view state of the timer)
//! // ...
//!
//! // Run timer in its own thread so it does not block the current one
//! thread::spawn(move || {
//!     Timer::new(terminal_input_receiver, view_sender, TimerConfig::default(), Box::new(move |state, msg| {
//!         println!("{} {}", state.round, msg);
//!     }))
//!         .init()
//!         .expect("Could not initialize timer");
//! });
//! ```

pub use timer::Timer;

pub mod config;
pub mod events;
pub mod timer;
pub mod util;
