mod app_global_state;
mod consumer;
mod producer;

use std::{fmt::Display, sync::{Arc, Mutex}};

use crate::entities_system::{
    app_global_state::GlobalState, consumer::Consumer, producer::Producer,
};

pub struct Entities {
    global_state: Arc<Mutex<GlobalState>>,
    producer: Producer,
    consumer: Consumer,
}

pub enum EntitiesErr {
   GlobalStateRxNoneErr,
}

impl Display for EntitiesErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
         write!(f, "Error in entities creation")
    }
}

impl Entities {
    pub fn new() -> Result<Self, EntitiesErr> {
        println!("Initializing the multi-threaded web crawler and indexer...");

        let guarded_global_state = Arc::new(Mutex::new(GlobalState::new()));

        let global_state_rx = match guarded_global_state.clone().lock() {
            Err(_) => {
                None
            },
            Ok(lock) => Some(lock.quarded_global_state_rx.clone()),
        };

        if let None = global_state_rx {
            return Err(EntitiesErr::GlobalStateRxNoneErr);
        }

        let producer = Producer::new(global_state_rx.unwrap());

        let consumer = Consumer::new(
            guarded_global_state.clone(),
            producer.garded_producer_rx.clone(),
        );

        Ok(Entities {
            global_state: guarded_global_state,
            producer,
            consumer: consumer,
        })
    }

    pub fn run(self) {
        let global_state = self.global_state;
        let mut producer = self.producer;
        let mut consumer = Some(self.consumer);

        println!("Main run started");
        loop {
            {
                match global_state.lock() {
                    Err(err) => {
                        println!("global_state lock error: {}", err)
                    }
                    Ok(lock) => {
                        lock.send_data_to_producer();
                    }
                };
            }

            {
                match global_state.lock() {
                    Err(err) => {
                        println!("global_state lock error: {}", err);
                    }
                    Ok(lock) => {
                        if lock.is_all_urls_visiting_done() {
                            if let Err(_) = lock.send_end_process_signal() {
                                continue;
                            }

                            let producer_threads_finished = producer.check_threads_finished();
                            let consumer_thread_finished = match consumer.as_ref() {
                                Some(value) => value.check_threads_finished(),
                                _ => true,
                            };

                            if producer_threads_finished {
                                println!("All producer threads finished, breaking the loop");
                                producer.join_threads();
                            }

                            if consumer_thread_finished {
                                if let Some(val) = consumer.take() {
                                    println!("All consumer threads finished, breaking the loop");
                                    val.join_thread()
                                }
                            }

                            if producer_threads_finished && consumer_thread_finished {
                                break;
                            }
                        }
                    }
                };
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
