pub const INITIAL_URLS: [&str; 5] = [
    "https://www.example.com",
    "https://www.rust-lang.org",
    "https://www.wikipedia.org",
    "https://www.github.com",
    "https://www.stackoverflow.com",
];

#[derive(Clone)]
pub struct NewUnReadUrl {
    pub url: &'static str,
    pub took: bool,
}

impl NewUnReadUrl {
    pub const fn new(url: &'static str) -> Self {
        NewUnReadUrl { url, took: false }
    }
}

pub const NEW_URLS: [&str; 15] = [
    "https://www.example2.com",
    "https://www.rust-lang2.org",
    "https://www.wikipedia2.org",
    "https://www.github2.com",
    "https://www.stackoverflow2.com",

    "https://www.example3.com",
    "https://www.rust-lang3.org",
    "https://www.wikipedia3.org",
    "https://www.github3.com",
    "https://www.stackoverflow3.com",

    "https://www.example4.com",
    "https://www.rust-lang4.org",
    "https://www.wikipedia4.org",
    "https://www.github4.com",
    "https://www.stackoverflow4.com",
];

pub const THREAD_COUNT: isize = 4;
// pub const THREAD_COUNT: isize = 1;
pub const SLEEP_DURATION: u64 = 1000; // Simulated network latency in milliseconds