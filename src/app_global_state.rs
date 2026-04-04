use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

use crate::{constants::INITIAL_URLS, producer::Producer};

#[derive(Debug)]
pub struct UrlData {
    url: String,
    content: String,
    visited: bool,
}

#[derive(Debug)]
pub struct GlobalState {
    url_visited: isize,
    // The Coordinator (Shared State): A central record that keeps track of which URLs have already been visited
    // so you don't crawl the same page twice.
    urls_data: HashMap<String, UrlData>,
    global_state_tx: Sender<Option<(isize)>>,
    global_state_receiver: Arc<Mutex<Receiver<Option<(isize)>>>>,
    producer: Producer,
}

impl GlobalState {
    pub fn new() -> Self {
        let (global_state_tx, global_state_rx) = channel::<Option<(isize)>>();

        let mut urls_data: HashMap<String, UrlData> = HashMap::new();
        let url_visited = 0;
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
        let global_state_receiver = Arc::new(Mutex::new(global_state_rx));
        let producer = Producer::new(&global_state_receiver);
        GlobalState {
            urls_data,
            url_visited,
            global_state_tx,
            global_state_receiver,
            producer,
        }
    }

    pub fn run(&mut self) {
        loop {
            if self.url_visited >= INITIAL_URLS.len() as isize {
                self.global_state_tx.send(None).unwrap();
                println!("All URLs have been visited.");

                let all_thread_finished = self.producer.check_threads_finished();

                if all_thread_finished {
                    println!("All producer threads have finished. Joining threads...");
                    self.producer.join_threads();
                    break;
                } else {
                    println!("Some producer threads are still running. Skipping join.");
                }

            }
            let producer_receiver = self.producer.producer_rx.recv().unwrap();

            match producer_receiver {
                Some(value) => {
                    println!("GlobalState received: {:?}", value);
                    self.global_state_tx.send(Some(value)).unwrap();
                    self.url_visited += 1;
                }
                None => {
                    println!("GlobalState received signal to stop processing data. Exiting...");
                    self.global_state_tx.send(None).unwrap();
                }
            }
        }
    }
}
