use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, channel},
    },
    thread::{self, JoinHandle},
};

mod producer_task;

use crate::{
    constants::THREAD_COUNT,
    entities_system::{
        app_global_state::GuardedGlobalReceiverType, producer::producer_task::ProducerTask,
    },
};

#[derive(Debug)]
pub struct NewUrls {
    pub urls1: Option<String>,
    pub urls2: Option<String>,
}

pub enum ProducerChannelData {
    ContinueProcessing(NewUrls),
    EndProcessing,
}

#[derive(Debug)]
pub struct Producer {
    pub handlers: Vec<JoinHandle<()>>,
    pub garded_producer_rx: Arc<Mutex<Receiver<ProducerChannelData>>>,
}

impl Producer {
    pub fn new(global_state_receiver: GuardedGlobalReceiverType) -> Self {
        let mut handlers: Vec<JoinHandle<()>> = Vec::new();
        let (producer_tx, producer_rx) = channel::<ProducerChannelData>();

        for _ in 0..THREAD_COUNT {
            let producer_tx_clone = producer_tx.clone();
            let global_state_receiver = global_state_receiver.clone();
            let task = move || {
                let mut producer_task = ProducerTask::new(global_state_receiver, producer_tx_clone);
                producer_task.run();
            };
            // let worker = Worker::new(task);
            let handle = thread::spawn(task);

            handlers.push(handle);
        }

        let garded_producer_rx = Arc::new(Mutex::new(producer_rx));

        Producer {
            handlers,
            garded_producer_rx,
        }
    }

    pub fn check_threads_finished(&self) -> bool {
        let producer_threads_finished = self.handlers.iter().all(|handler| handler.is_finished());
        producer_threads_finished
    }

    pub fn join_threads(&mut self) {
        for worker in self.handlers.drain(..) {
            println!("Joining producer thread...");
            let join_res = worker.join();
            match join_res {
                Ok(_) => println!("Producer thread joined successfully"),
                Err(e) => eprintln!("Failed to join producer thread: {:?}", e),
            }
        }
    }
}
