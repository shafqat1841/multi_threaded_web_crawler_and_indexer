use std::{collections::HashMap, sync::mpsc::{Receiver, Sender}};

use crate::constants::INITIAL_URLS;

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
    global_state_tx: Sender<Option<()>>,
    producer_rx: Receiver<Option<()>>,
}

impl GlobalState {
    pub fn new(global_state_tx: Sender<Option<()>>, producer_rx: Receiver<Option<()>>) -> Self {
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
        GlobalState {
            urls_data,
            url_visited,
            global_state_tx,
            producer_rx,
        }
    }

    pub fn run(&mut self){
        loop {
            println!("GlobalState is waiting for data from producer...");
            let received_data = self.producer_rx.recv();
            match received_data {
                Ok(data) => {
                    match data {
                        Some(_) => {
                            println!("GlobalState received: {:?}", data);
                            self.global_state_tx.send(Some(())).expect("Failed to send signal to producer thread");
                        },
                        None => {
                            println!("GlobalState received signal to stop processing data. Exiting...");
                            break;
                        }
                    }
                },
                Err(_) => {
                    println!("GlobalState channel closed");
                    break;
                },
            }
        }
    }
}
