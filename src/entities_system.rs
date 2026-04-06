mod app_global_state;
mod producer;

use std::sync::Arc;

use crate::{
    constants::INITIAL_URLS,
    entities_system::{app_global_state::GlobalState, producer::Producer},
};

pub struct Entities {
    global_state: GlobalState,
    producer: Producer,
}

impl Entities {
    pub fn new() -> Self {
        println!("Initializing the multi-threaded web crawler and indexer...");

        let global_state = GlobalState::new();
        let global_state_receiver = Arc::clone(&global_state.quarded_global_state_rx);
        let producer = Producer::new(global_state_receiver);

        Entities {
            global_state,
            producer,
        }
    }

    pub fn run(&mut self) {
        println!("Main running");

        loop {
            let url_visited = {
                self.global_state.url_visited.lock().unwrap().clone()
            };
            if url_visited >= INITIAL_URLS.len() as isize {
                if self.producer.check_threads_finished() {
                    println!("All threads finished, breaking the loop");
                    self.producer.join_threads();
                    break;
                }else {
                    println!("Waiting for threads to finish...");
                }
            }

            let unvisited_url = self.global_state.urls_data.iter().find(|item| {
                let lock_item = item.1.lock().unwrap();
                !lock_item.visited && !lock_item.in_processing
            });

            match unvisited_url {
                Some(value) => {
                    let send_value = value.1.clone();

                    send_value.lock().unwrap().in_processing = true;
                    let url_visited = self.global_state.url_visited.clone();
                    let send_data = (send_value, url_visited);
                    // println!("Main thread sending data: {:?}", send_value);
                    self.global_state
                        .global_state_tx
                        .send(Some(send_data))
                        .unwrap();
                }
                None => {
                    self.global_state.global_state_tx.send(None).unwrap();
                }
            }
        }

        println!("Main ended")
    }
}
