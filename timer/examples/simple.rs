use std::rc::Rc;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use zentime_rs_timer::config::PomodoroTimerConfig;
use zentime_rs_timer::pomodoro_timer::{PomodoroTimer, ViewState};
use zentime_rs_timer::pomodoro_timer_action::PomodoroTimerAction;

fn main() {
    let (terminal_input_sender, terminal_input_receiver): (
        Sender<PomodoroTimerAction>,
        Receiver<PomodoroTimerAction>,
    ) = mpsc::channel();
    let (view_sender, view_receiver): (Sender<ViewState>, Receiver<ViewState>) = mpsc::channel();

    let config = PomodoroTimerConfig::default();

    // Run timer in its own thread so it does not block the current one
    thread::spawn(move || {
        let timer = PomodoroTimer::new(
            config,
            Rc::new(move |state, msg| {
                println!("{} {}", state.round, msg);
            }),
            Rc::new(move |state| -> Option<PomodoroTimerAction> {
                view_sender.send(state).unwrap();

                let input = terminal_input_receiver.recv_timeout(Duration::from_secs(1));

                match input {
                    Ok(action) => Some(action),
                    Err(RecvTimeoutError::Disconnected) => None, // Handle error in a real scenario
                    _ => None,
                }
            }),
        );

        timer.init()
    });

    let action_jh = thread::spawn(move || {
        // Start the timer
        terminal_input_sender
            .send(PomodoroTimerAction::PlayPause)
            .unwrap();

        // Render current timer state three seconds in a row
        for _ in 0..3 {
            thread::sleep(Duration::from_secs(1));
            if let Ok(state) = view_receiver.recv() {
                println!("{}", state.time)
            }
        }

        // Pause the timer
        terminal_input_sender
            .send(PomodoroTimerAction::PlayPause)
            .unwrap();
        let state = view_receiver.recv().unwrap();

        // NOTE:
        // The received messages after pausing can be a bit irritating,
        // depending on how long you pause the timer this is because our thread
        // is sleeping while the paused timer thread is still sending messages submitting its
        // (state which ofcourse will always be the same, as long as the timer is paused).
        // Each recv() below is basically just catching up with the timer.
        // For example if we would wait 3 seconds instead of one, we would
        // see 24:27 three times in a row, because these messages have already been queued.
        println!(
            "Paused at {}, waiting 1 seconds before resuming",
            state.time
        );

        thread::sleep(Duration::from_secs(1));

        // Start the timer again
        terminal_input_sender
            .send(PomodoroTimerAction::PlayPause)
            .unwrap();

        // Render current timer state three seconds in a row
        for _ in 0..3 {
            thread::sleep(Duration::from_secs(1));
            if let Ok(state) = view_receiver.recv() {
                println!("{}", state.time)
            }
        }
    });

    action_jh.join().unwrap();
}
