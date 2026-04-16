use std::{
    sync::{Arc, Mutex, atomic::AtomicIsize},
    thread::{self, JoinHandle},
};

mod producer_task;

use crossbeam::channel::{Receiver, unbounded};
use thiserror::Error;

use crate::{
    constants::THREAD_COUNT,
    entities_system::{app_global_state::GlobalState, producer::producer_task::ProducerTask},
};

#[derive(Debug)]
pub struct NewUrls {
    pub urls1: Option<String>,
    pub urls2: Option<String>,
}

pub enum ProducerChannelData {
    ContinueProcessing(NewUrls, Arc<AtomicIsize>),
    EndProcessing,
}

#[derive(Debug)]
pub struct Producer {
    pub handlers: Vec<JoinHandle<()>>,
    pub producer_rx: Receiver<ProducerChannelData>,
}

#[derive(Error, Debug)]
pub enum ProducerErr {
    #[error("An error occured during locking of global state receiver")]
    GlobalStateRxNoneErr,
      #[error("Error in creating the handler")]
    HandlerError,
}

impl Producer {
    pub fn new(guarded_global_state: Arc<GlobalState>) -> Result<Self, ProducerErr> {
        let mut handlers: Vec<JoinHandle<()>> = Vec::new();

        let (producer_tx, producer_rx) = unbounded::<ProducerChannelData>();

        let new_urls: [String; 15] = [
            "https://www.example2.com".to_string(),
            "https://www.rust-lang2.org".to_string(),
            "https://www.wikipedia2.org".to_string(),
            "https://www.github2.com".to_string(),
            "https://www.stackoverflow2.com".to_string(),
            "https://www.example3.com".to_string(),
            "https://www.rust-lang3.org".to_string(),
            "https://www.wikipedia3.org".to_string(),
            "https://www.github3.com".to_string(),
            "https://www.stackoverflow3.com".to_string(),
            "https://www.example4.com".to_string(),
            "https://www.rust-lang4.org".to_string(),
            "https://www.wikipedia4.org".to_string(),
            "https://www.github4.com".to_string(),
            "https://www.stackoverflow4.com".to_string(),
        ];

        let new_urls_gearded = Arc::new(Mutex::new(new_urls.to_vec()));

        for i in 0..THREAD_COUNT {
            let guarded_global_state = guarded_global_state.clone();

            let producer_tx_clone = producer_tx.clone();
            let mut threat_name: String = "Producer thread".to_string();

            threat_name.push_str(&" ".to_string());
            threat_name.push_str(&i.to_string());

            let threat_name_clone = threat_name.clone();

            let new_urls_gearded_clone = new_urls_gearded.clone();

            let task = move || match ProducerTask::new(
                guarded_global_state,
                producer_tx_clone,
                threat_name_clone,
                new_urls_gearded_clone,
            ) {
                Ok(mut producer_task) => {
                    producer_task.run();
                }
                Err(err) => {
                    eprintln!("error: {}", err);
                }
            };

            let handler = thread::Builder::new().name(threat_name).spawn(task);

            match handler {
                Err(err) => {
                    println!("Error in creating the handler: {}", err);
                    return Err(ProducerErr::HandlerError);
                }
                Ok(value) => handlers.push(value),
            }

        }

        Ok(Producer {
            handlers,
            producer_rx,
        })
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
