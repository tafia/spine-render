extern crate time;

use std::thread;
use std::time::Duration;

pub enum Action {
    Stop,
    Continue,
}

pub fn start_loop<F>(duration_ns: u64, mut callback: F) where F: FnMut() -> Action {
    let mut accumulator = 0;
    let mut previous_clock = time::precise_time_ns();

    loop {
        match callback() {
            Action::Stop => break,
            Action::Continue => ()
        };

        let now = time::precise_time_ns();
        accumulator += now - previous_clock;
        previous_clock = now;

        while accumulator >= duration_ns {
            accumulator -= duration_ns;

            // if you have a game, update the state here
        }

        thread::sleep(Duration::new(0, (duration_ns - accumulator) as u32));
    }
}
