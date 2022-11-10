use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;
use zentime_rs::config::create_config;
use zentime_rs::timer::Timer;
use zentime_rs::{AppAction, TerminalEvent};

fn main() {
    let config = create_config("")
        .extract()
        .expect("Could not create config");

    let (terminal_input_sender, terminal_input_receiver): (Sender<AppAction>, Receiver<AppAction>) =
        mpsc::channel();
    let (view_sender, view_receiver): (Sender<TerminalEvent>, Receiver<TerminalEvent>) =
        mpsc::channel();

    // Run timer in its own thread so it does not block the current one
    thread::spawn(move || {
        Timer::new(terminal_input_receiver, view_sender, config)
            .init()
            .expect("ERROR: Timer hung up");
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
