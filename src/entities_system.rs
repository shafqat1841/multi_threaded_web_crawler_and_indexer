mod app_global_state;
mod consumer;
mod producer;

use std::sync::{Arc, Mutex};

use crate::{
    constants::{INITIAL_URLS, MAX_URLS_TO_PROCESS},
    entities_system::{
        app_global_state::{GlobalState, UrlData},
        consumer::Consumer,
        producer::Producer,
    },
};

pub struct Entities {
    global_state: GlobalState,
    producer: Producer,
    consumer: Consumer,
}

impl Entities {
    pub fn new() -> Self {
        println!("Initializing the multi-threaded web crawler and indexer...");

        let global_state = GlobalState::new();

        let producer = Producer::new(global_state.quarded_global_state_rx.clone());

        let consumer = Consumer::new(producer.garded_producer_rx.clone());

        Entities {
            global_state,
            producer,
            consumer: consumer,
        }
    }

    pub fn run(self) {
        let mut global_state = self.global_state;
        let mut producer = self.producer;
        let mut consumer = Some(self.consumer);

        loop {
            println!("Main loop started");

            global_state.urls_data.iter().for_each(|item| {
                println!("Checking url: {:?}", item.0);
                let lock_item = { item.1.lock() };
                match lock_item {
                    Ok(value) => {
                        println!(
                            "global_state.urls_data item.0: {:?}, in_processing: {}, visited: {}",
                            item.0,
                            value.in_processing,
                            value.visited
                        );
                    },
                    Err(_) => {
                        eprintln!("Failed to lock url data for url: {:?}", item.0);
                        // return;
                    }
                };
            });

            let unvisited_url = global_state.urls_data.iter().find(|item| {
                println!("11111");
                let lock_item = { item.1.lock() };
                let url_info = match lock_item {
                    Ok(value) => Some(value),
                    Err(_) => {
                        eprintln!("Failed to lock url data for url: {:?}", item.0);
                        None
                    }
                };
                match url_info {
                    Some(value) => !value.visited && !value.in_processing,
                    None => false,
                }
            });

            match unvisited_url {
                Some(value) => {
                    let send_value = value.1.clone();

                    send_value.lock().unwrap().in_processing = true;
                    let url_visited = global_state.url_visited.clone();
                    let send_data = (send_value, url_visited);
                    // println!("Main thread sending data: {:?}", send_value);
                    global_state.global_state_tx.send(Some(send_data)).unwrap();
                }
                None => {
                    global_state.global_state_tx.send(None).unwrap();
                }
            }

            let mut url: Option<String> = None;
            match consumer.as_ref() {
                Some(consumer) => match consumer.consumer_rx.recv() {
                    Ok(value) => {
                        url = value;
                    }
                    Err(_) => {
                        println!("Main thread failed to receive url from consumer");
                    }
                },
                None => {
                    println!("Main thread received None from consumer");
                }
            }

            match url {
                Some(url) => {
                    let arc_url = Arc::new(url);
                    let url_data = UrlData {
                        url: arc_url.clone().to_string(),
                        content: String::new(),
                        visited: false,
                        in_processing: false,
                    };
                    let url_data_quarded = Arc::new(Mutex::new(url_data));
                    global_state
                        .urls_data
                        .insert(arc_url.clone().to_string(), url_data_quarded);
                }
                None => {
                    println!("Main thread received None url data");
                }
            }

            let url_visited = { global_state.url_visited.lock().unwrap().clone() };

            if url_visited >= MAX_URLS_TO_PROCESS {
                let producer_threads_finished = producer.check_threads_finished();
                // let consumer_thread_finished = consumer.as_ref().unwrap().check_threads_finished();
                let consumer_thread_finished = consumer.as_ref().unwrap().check_threads_finished();

                if producer_threads_finished {
                    println!("All producer threads finished, breaking the loop");
                    producer.join_threads();
                }

                if consumer_thread_finished {
                    println!("All consumer threads finished, breaking the loop");
                    consumer.take().unwrap().join_thread();
                }

                if producer_threads_finished && consumer_thread_finished {
                    break;
                } else {
                    println!("Waiting for threads to finish...");
                }
            }
            println!("Main loop ended");
        }

        println!("Main ended")
    }
}
