#![warn(missing_docs)]
//! Pomodoro/Productivity timer that can transition between various states ([Paused]/[Running]),
//! tracks intervals and can be configured.
//!
//! ## Example
//!
//! ```
//! use std::sync::mpsc::{self, RecvTimeoutError};
//! use std::sync::mpsc::{Receiver, Sender};
//! use std::thread;
//! use std::time::Duration;
//! use zentime_rs_timer::config::TimerConfig;
//! use zentime_rs_timer::events::AppAction;
//! use zentime_rs_timer::events::TerminalEvent;
//! use zentime_rs_timer::timer::Timer;
//!
//!     let (terminal_input_sender, terminal_input_receiver): (Sender<AppAction>, Receiver<AppAction>) =
//!         mpsc::channel();
//!     let (view_sender, view_receiver): (Sender<TerminalEvent>, Receiver<TerminalEvent>) =
//!         mpsc::channel();
//!
//!     let config = TimerConfig::default();
//!
//!     // Run timer in its own thread so it does not block the current one
//!     thread::spawn(move || {
//!         let timer = Timer::new(
//!             config,
//!             Box::new(move |state, msg| {
//!                 println!("{} {}", state.round, msg);
//!             }),
//!             Box::new(move |view_state| -> Option<AppAction> {
//!                 view_sender.send(TerminalEvent::View(view_state)).unwrap();
//!
//!                 let input = terminal_input_receiver.recv_timeout(Duration::from_secs(1));
//!
//!                 match input {
//!                     Ok(action) => Some(action),
//!                     Err(RecvTimeoutError::Disconnected) => Some(AppAction::Quit),
//!                     _ => None,
//!                 }
//!             }),
//!         );
//!
//!         if timer.init().is_err() {
//!             // Do nothing
//!         };
//!     });
//!
//!     let action_jh = thread::spawn(move || {
//!         // Start the timer
//!         terminal_input_sender.send(AppAction::PlayPause).unwrap();
//!
//!         // Render current timer state three seconds in a row
//!         for _ in 0..3 {
//!             thread::sleep(Duration::from_secs(1));
//!             if let Ok(TerminalEvent::View(state)) = view_receiver.recv() {
//!                 println!("{}", state.time)
//!             }
//!         }
//!
//!         // Terminate timer
//!         terminal_input_sender
//!             .send(AppAction::Quit)
//!             .expect("Could not send quit action");
//!     });
//!
//!     action_jh.join().unwrap();
//! ```

pub use timer::Timer;

pub mod config;
pub mod events;
pub mod timer;
pub mod util;
