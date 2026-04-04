use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread::{self, JoinHandle},
};

const INITIAL_URLS: [&str; 5] = [
    "https://www.example.com",
    "https://www.rust-lang.org",
    "https://www.wikipedia.org",
    "https://www.github.com",
    "https://www.stackoverflow.com",
];

const THREAD_COUNT: isize = 4;
const sleep_duration: u64 = 1000; // Simulated network latency in milliseconds

#[derive(Debug)]
struct UrlData {
    url: String,
    content: String,
    visited: bool,
}

#[derive(Debug)]
struct GlobalState {
    stop: bool,
    url_visited: isize,
    // The Coordinator (Shared State): A central record that keeps track of which URLs have already been visited
    // so you don't crawl the same page twice.
    urls_data: HashMap<String, UrlData>,
}

impl GlobalState {
    fn new() -> Self {
        let mut urls_data: HashMap<String, UrlData> = HashMap::new();
        let url_visited = 0;
        let stop = false;
        INITIAL_URLS.iter().for_each(|url| {
            urls_data.insert(
                url.to_string(),
                UrlData {
                    url: url.to_string(),
                    content: String::new(),
                    visited: false,
                },
            );
        });
        GlobalState {
            urls_data,
            url_visited,
            stop,
        }
    }
}

struct Producer {
    // The Producers (Crawlers): Multiple threads that "visit" a URL, wait (simulate network latency),
    // and find "new" URLs.
    thread_count: isize,
    stop: bool,
    tx: Sender<UrlData>,
}

impl Producer {
    fn new(tx: Sender<UrlData>) -> Self {
        Producer {
            thread_count: THREAD_COUNT,
            stop: false,
            tx,
        }
    }
}

struct consumer {
    // The Consumer (Indexer): A single background thread that receives discovered data and "indexes" it
    // (updates a global word count).
}

pub fn run() {
    // let (tx, rx) = mpsc::channel::<UrlData>();

    // let global_state = GlobalState::new();
    // let producer = Producer::new(tx);
    // println!("{:?}", global_state);
    let (tx, rx) = mpsc::channel::<usize>();

    let mut handlers: Vec<JoinHandle<()>> = Vec::new();

    let rx_ref = Arc::new(Mutex::new(rx));
    for i in 0..4 {
        let rx_clone = Arc::clone(&rx_ref);
        let handler = thread::spawn(move || {
            loop {
                let data = rx_clone.lock().unwrap().recv();

                match data {
                    Ok(data) => {
                        println!("thread: {} ,Received: {:?}", i, data);
                    }
                    Err(_) => {
                        println!("thread: {} ,Channel closed", i);
                        break;
                    }
                }
            }
        });
        handlers.push(handler);
    }

    for i in 0..8 {
        tx.send(i).unwrap();
    }

    drop(tx); // Close the channel
    handlers.into_iter().for_each(|h| h.join().unwrap());
}
