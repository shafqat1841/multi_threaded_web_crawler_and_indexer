use std::{thread::sleep, time::Duration};

pub const INITIAL_URLS: [&str; 5] = [
    "https://www.example.com",
    "https://www.rust-lang.org",
    "https://www.wikipedia.org",
    "https://www.github.com",
    "https://www.stackoverflow.com",
];

pub const MAX_URLS_TO_PROCESS: isize = 15;

pub const THREAD_COUNT: isize = 4;
// pub const THREAD_COUNT: isize = 1;
pub const SLEEP_DURATION: u64 = 1000; // Simulated network latency in milliseconds

pub fn sleep_in_milisecond(milisecond: u64) {
    sleep(Duration::new(milisecond, 0));
}
