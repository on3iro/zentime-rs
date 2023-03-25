use std::rc::Rc;
use std::sync::mpsc::channel;
use std::thread::{self, sleep};
use std::time::Duration;
use tokio::task;
use zentime_rs_timer::config::PomodoroTimerConfig;
use zentime_rs_timer::pomodoro_timer::PomodoroTimer;
use zentime_rs_timer::pomodoro_timer_action::PomodoroTimerAction;

#[tokio::main]
async fn main() {
    let (terminal_input_sender,  terminal_input_receiver) = channel();
    let (view_sender, mut view_receiver) = tokio::sync::mpsc::unbounded_channel();

    let config = PomodoroTimerConfig::default();

    // Run timer in its own thread so it does not block the current one
    thread::spawn(move || {
        let timer = PomodoroTimer::new(
            config,
            Rc::new(move |state, msg, _| {
                println!("{} {}", state.round, msg.unwrap());
            }),
            Rc::new(move |state| -> Option<PomodoroTimerAction> {
                view_sender.send(state).unwrap();

                sleep(Duration::from_secs(1));

                let input = terminal_input_receiver.try_recv();

                match input {
                    Ok(action) => Some(action),
                    _ => None,
                }
            }),
        );

        timer.init()
    });

    task::spawn(async move {
        // Start the timer
        terminal_input_sender
            .send(PomodoroTimerAction::PlayPause)
            .unwrap();

        // Render current timer state three seconds in a row
        for _ in 0..3 {
            let state = view_receiver.recv().await.unwrap();
            println!("{}", state.time);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        // Pause the timer
        terminal_input_sender
            .send(PomodoroTimerAction::PlayPause)
            .unwrap();
        let state = view_receiver.recv().await.unwrap();

        // NOTE:
        // The received messages after pausing can be a bit irritating,
        // depending on how long you pause the timer this is because our task
        // is sleeping while the timer thread is still sending messages.
        // Each recv() below is basically just catching up with the timer.
        // For example if we would wait 3 seconds instead of one, we would
        // see 24:27 three times in a row, because these messages have already been queued.
        println!(
            "Paused at {}, waiting 1 seconds before resuming",
            state.time
        );

        tokio::time::sleep(Duration::from_secs(1)).await;

        // Start the timer again
        terminal_input_sender
            .send(PomodoroTimerAction::PlayPause)
            .unwrap();

        // Render current timer state three seconds in a row
        for _ in 0..3 {
            let state = view_receiver.recv().await.unwrap();
            println!("{}", state.time);
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    })
    .await
    .unwrap();
}
