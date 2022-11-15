use std::thread;
use zentime_rs_timer::config::TimerConfig;
use zentime_rs_timer::timer::Timer;

fn main() {
    // Run timer in its own thread so it does not block the current one
    thread::spawn(move || {
        Timer::new(
            TimerConfig::default(),
            Box::new(move |state, msg| {
                println!("{} {}", state.round, msg);
            }),
            Box::new(move |view_state| {
                println!("{:?}", view_state);
                None
            }),
        )
        .init()
        .expect("Could not initialize timer");
    });
}
