mod app_global_state;
mod consumer;
mod producer;

use thiserror::Error;

use std::sync::{Arc, Mutex};

use crate::entities_system::{
    app_global_state::GlobalState,
    consumer::Consumer,
    producer::{Producer, ProducerErr},
};

pub struct Entities {
    global_state: Arc<Mutex<GlobalState>>,
    producer: Producer,
    consumer: Consumer,
}

#[derive(Error, Debug)]
pub enum EntitiesErr {
    #[error("An error occured during locking of global state receiver in producer")]
    ProducerErr(#[from] ProducerErr),
}


#[derive(Error, Debug)]
pub enum EntitiesRunErr {
    #[error(
        "An error occured during locking of global state receiver when the process was running"
    )]
    MutexPoisonErr,
}

impl Entities {
    pub fn new() -> Result<Self, EntitiesErr> {
        println!("Initializing the multi-threaded web crawler and indexer...");

        let guarded_global_state = Arc::new(Mutex::new(GlobalState::new()));

        let producer = Producer::new(guarded_global_state.clone())?;

        let consumer = Consumer::new(
            guarded_global_state.clone(),
            producer.producer_rx.clone(),
        );

        Ok(Entities {
            global_state: guarded_global_state,
            producer,
            consumer: consumer,
        })
    }

    pub fn run(self) -> Result<(), EntitiesRunErr> {
        let global_state: Arc<Mutex<GlobalState>> = self.global_state;
        let mut producer = self.producer;
        let mut consumer = Some(self.consumer);

        println!("Main run started");
        loop {
            {
                let lock = global_state.lock().map_err(|_| EntitiesRunErr::MutexPoisonErr)?;
                lock.send_data_to_producer();

                if lock.is_all_urls_visiting_done() {
                    if let Err(_) = lock.send_end_process_signal() {
                        // continue;
                        break;
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

        println!("Main ended");
        Ok(())
    }
}
