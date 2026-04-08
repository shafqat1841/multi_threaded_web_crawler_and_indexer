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
    // consumer: Consumer,
}

impl Entities {
    pub fn new() -> Self {
        println!("Initializing the multi-threaded web crawler and indexer...");

        let global_state = GlobalState::new();

        let producer = Producer::new(global_state.quarded_global_state_rx.clone());

        // let consumer = Consumer::new(producer.garded_producer_rx.clone());

        Entities {
            global_state,
            producer,
            // consumer: consumer,
        }
    }

    pub fn run(self) {
        let mut global_state = self.global_state;
        let mut producer = self.producer;
        // let mut consumer = Some(self.consumer);

        println!("Main run started");
        loop {

            global_state.send_data_to_producer();


            // let mut url: Option<String> = None;
            // match consumer.as_ref() {
            //     Some(consumer) => match consumer.consumer_rx.try_recv() {
            //         Ok(value) => {
            //             url = value;
            //         }
            //         Err(err) => {
            //             eprintln!("Main thread failed to receive url from consumer: {:?}", err);
            //         }
            //     },
            //     None => {
            //         println!("Main thread received None from consumer");
            //     }
            // }

            // match url {
            //     Some(url) => {
            //         let arc_url = Arc::new(url);
            //         println!("arc_url: {}", arc_url);
            //         let url_data = UrlData {
            //             url: arc_url.clone().to_string(),
            //             content: String::new(),
            //             visited: false,
            //             in_processing: false,
            //         };
            //         let url_data_quarded = Arc::new(Mutex::new(url_data));
            //         global_state
            //             .urls_data
            //             .insert(arc_url.clone().to_string(), url_data_quarded);
            //     }
            //     None => {
            //         println!("Main thread received None url data");
            //     }
            // }

            let url_visited = { global_state.url_visited.lock().unwrap().clone() };

            // println!("url_visited: {}", url_visited);

            if url_visited >= MAX_URLS_TO_PROCESS {
                let producer_threads_finished = producer.check_threads_finished();
                // let consumer_thread_finished = consumer.as_ref().unwrap().check_threads_finished();
                // let consumer_thread_finished = consumer.as_ref().unwrap().check_threads_finished();

                if producer_threads_finished {
                    println!("All producer threads finished, breaking the loop");
                    producer.join_threads();
                    break;
                } else {
                    println!("Waiting for threads to finish...");
                }

                // if consumer_thread_finished {
                //     println!("All consumer threads finished, breaking the loop");
                //     consumer.take().unwrap().join_thread();
                // }

                // if producer_threads_finished && consumer_thread_finished {
                // if producer_threads_finished {
                //     break;
                // } else {
                //     println!("Waiting for threads to finish...");
                // }
            }
        }
        println!("Main run ended");

        global_state.urls_data.iter().for_each(|item| {
            println!("{:?}", item.1.lock().unwrap());
        });

        println!("Main ended")
    }
}
