use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use zentime_rs_timer::config::TimerConfig;
use zentime_rs_timer::events::AppAction;
use zentime_rs_timer::events::TerminalEvent;
use zentime_rs_timer::timer::Timer;

fn main() {
    let (terminal_input_sender, terminal_input_receiver): (Sender<AppAction>, Receiver<AppAction>) =
        mpsc::channel();
    let (view_sender, view_receiver): (Sender<TerminalEvent>, Receiver<TerminalEvent>) =
        mpsc::channel();

    let config = TimerConfig::default();

    // Run timer in its own thread so it does not block the current one
    thread::spawn(move || {
        if Timer::new(
            terminal_input_receiver,
            view_sender,
            config,
            Box::new(move |state, msg| println!("{}: {}", state.round, msg)),
        )
        .init()
        .is_err()
        {
            // Do nothing
        };
    });

    let action_jh = thread::spawn(move || {
        // Start the timer
        terminal_input_sender.send(AppAction::PlayPause).unwrap();

        // Render current timer state three seconds in a row
        for _ in 0..3 {
            thread::sleep(Duration::from_secs(1));
            if let Ok(TerminalEvent::View(state)) = view_receiver.recv() {
                println!("{}", state.time)
            }
        }

        // Terminate timer
        terminal_input_sender
            .send(AppAction::Quit)
            .expect("Could not send quit action");
    });

    action_jh.join().unwrap();
}
