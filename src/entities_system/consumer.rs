mod consumer_task;

use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use crossbeam::channel::Receiver;
use thiserror::Error;

use crate::entities_system::{
    app_global_state::GlobalState, consumer::consumer_task::ConsumerTask,
    producer::ProducerChannelData,
};

pub struct Consumer {
    handler: JoinHandle<()>,
}

#[derive(Error,Debug)]
pub enum ConsumerErr {
    #[error("Error in creating the handler")]
    HandlerError,
}

impl Consumer {
    pub fn new(
        guarded_global_state: Arc<GlobalState>,
        producer_rx: Receiver<ProducerChannelData>,
    ) -> Result<Self,ConsumerErr> {
        let producer_rx_clone = producer_rx.clone();
        let guarded_global_state_clone = guarded_global_state.clone();
        let task = move || match ConsumerTask::new(guarded_global_state_clone, producer_rx_clone) {
            Ok(consumer_task) => {
                consumer_task.run();
            }
            Err(err) => {
                eprintln!("error: {}", err);
            }
        };

        let handler = thread::Builder::new()
            .name("consumer thread".to_string())
            .spawn(task);

        match handler {
            Err(err) => {
                println!("Error in creating the handler: {}", err);
                return Err(ConsumerErr::HandlerError)
            }
            Ok(value) => Ok(Consumer { handler: value }),
        }
    }

    pub fn check_threads_finished(&self) -> bool {
        self.handler.is_finished()
    }

    pub fn join_thread(self) {
        match self.handler.join() {
            Err(err) => {
                println!("Error during consumer join. error: {:?}",err)
            }
            _ => {}
        }
    }
}
