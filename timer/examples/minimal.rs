use std::rc::Rc;
use std::thread;
use zentime_rs_timer::pomodoro_timer::PomodoroTimer;

fn main() {
    // Run timer in its own thread so it does not block the current one
    thread::spawn(move || {
        PomodoroTimer::new(
            Default::default(),
            Rc::new(move |state, msg, _| {
                println!("{} {}", state.round, msg.unwrap());
            }),
            Rc::new(move |view_state| {
                println!("{:?}", view_state);
                None
            }),
        )
        .init()
    });
}
