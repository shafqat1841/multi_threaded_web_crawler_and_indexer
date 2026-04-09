mod app_global_state;
mod consumer;
mod producer;

use std::sync::{Arc, Mutex};

use crate::entities_system::{
    app_global_state::GlobalState, consumer::Consumer, producer::Producer,
};

pub struct Entities {
    global_state: Arc<Mutex<GlobalState>>,
    producer: Producer,
    consumer: Consumer,
}

impl Entities {
    pub fn new() -> Self {
        println!("Initializing the multi-threaded web crawler and indexer...");

        // let mut global_state = GlobalState::new();

        let guarded_global_state = Arc::new(Mutex::new(GlobalState::new()));
        let producer = Producer::new(
            guarded_global_state
                .lock()
                .unwrap()
                .quarded_global_state_rx
                .clone(),
        );

        let consumer = Consumer::new(
            guarded_global_state.clone(),
            producer.garded_producer_rx.clone(),
        );

        Entities {
            global_state: guarded_global_state,
            producer,
            consumer: consumer,
        }
    }

    pub fn run(self) {
        let global_state = self.global_state;
        let mut producer = self.producer;
        let mut consumer = Some(self.consumer);

        println!("Main run started");
        loop {
            global_state.lock().unwrap().send_data_to_producer();

            if global_state.lock().unwrap().is_all_urls_visiting_done() {
                let _ = global_state.lock().unwrap().send_end_process_signal();

                let producer_threads_finished = producer.check_threads_finished();
                // let consumer_thread_finished = consumer.check_threads_finished();
                let consumer_thread_finished = match consumer.as_ref() {
                    Some(value) => value.check_threads_finished(),
                    _ => true,
                };

                if producer_threads_finished {
                    println!("All producer threads finished, breaking the loop");
                    producer.join_threads();
                    // break;
                }

                if consumer_thread_finished {
                    match consumer.take() {
                        Some(val) => {
                            println!("All consumer threads finished, breaking the loop");
                            val.join_thread()
                        }
                        None => {}
                    };
                }

                if producer_threads_finished && consumer_thread_finished {
                    break;
                }
            }
        }
        println!("Main run ended");

        {
            println!("length: {}", global_state.lock().unwrap().urls_data.len());
            global_state
                .lock()
                .unwrap()
                .urls_data
                .iter()
                .for_each(|item| {
                    println!("{:?}", item.1.lock().unwrap());
                });
        }

        println!("Main ended")
    }
}
