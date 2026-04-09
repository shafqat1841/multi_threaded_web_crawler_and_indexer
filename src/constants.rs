
pub const INITIAL_URLS: [&str; 5] = [
    "https://www.example.com",
    "https://www.rust-lang.org",
    "https://www.wikipedia.org",
    "https://www.github.com",
    "https://www.stackoverflow.com",
];

pub const MAX_URLS_TO_PROCESS: isize = 20;

pub const THREAD_COUNT: isize = 4;
pub const SLEEP_DURATION: u64 = 1000; // Simulated network latency in milliseconds

