mod app_global_state;
mod consumer;
mod producer;

use thiserror::Error;

use std::sync::Arc;

use crate::entities_system::{
    app_global_state::GlobalState,
    consumer::{Consumer, ConsumerErr},
    producer::{Producer, ProducerErr},
};

pub struct Entities {
    global_state: Arc<GlobalState>,
    producer: Producer,
    consumer: Consumer,
}

#[derive(Error, Debug)]
pub enum EntitiesErr {
    #[error("An error occured during locking of global state receiver in producer")]
    ProducerErr(#[from] ProducerErr),
    #[error("An error occured during Consumer creation")]
    ConsumerErr(#[from] ConsumerErr)
}

impl Entities {
    pub fn new() -> Result<Self, EntitiesErr> {
        println!("Initializing the multi-threaded web crawler and indexer...");

        let guarded_global_state = Arc::new(GlobalState::new());

        let producer = Producer::new(guarded_global_state.clone())?;

        let consumer = Consumer::new(guarded_global_state.clone(), producer.producer_rx.clone())?;

        Ok(Entities {
            global_state: guarded_global_state,
            producer,
            consumer: consumer,
        })
    }

    pub fn run(self) {
        let global_state: Arc<GlobalState> = self.global_state;
        let mut producer = self.producer;
        let mut consumer = Some(self.consumer);

        println!("Main run started");
        if let Err(err) = global_state.send_data_to_producer() {
            println!("send data to producer error: {}", err);
            return;
        };
        loop {
            let all_urls_visiting_done = global_state.is_all_urls_visiting_done();
            if all_urls_visiting_done {
                if let Err(_) = global_state.send_end_process_signal() {
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
        println!("Main run ended");

        {
            println!("length: {}", global_state.urls_data.len());
            global_state.urls_data.iter().for_each(|item| {
                println!("url: {:?}, data: {:?}", item.key(), item.value());
            });
        }

        println!("Main ended");
    }
}
