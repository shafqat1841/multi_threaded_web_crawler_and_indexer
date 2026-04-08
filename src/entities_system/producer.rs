use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, JoinHandle},
};

mod producer_task;

use crate::{
    constants::THREAD_COUNT,
    entities_system::{
        app_global_state::GuardedGlobalReceiverType, producer::producer_task::producer_task,
    },
};

#[derive(Debug)]
pub struct NewUrls {
    pub urls1: Option<String>,
    pub urls2: Option<String>,
}

#[derive(Debug)]
pub struct Producer {
    pub handlers: Vec<JoinHandle<()>>,
    pub producer_tx: Sender<NewUrls>,
    pub garded_producer_rx: Arc<Mutex<Receiver<NewUrls>>>,
}

impl Producer {
    pub fn new(global_state_receiver: GuardedGlobalReceiverType) -> Self {
        let mut handlers: Vec<JoinHandle<()>> = Vec::new();
        let (producer_tx, producer_rx) = channel::<NewUrls>();

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

        let new_urls = Arc::new(Mutex::new(new_urls.to_vec()));

        for i in 0..THREAD_COUNT {
            let producer_tx_clone = producer_tx.clone();
            let global_state_receiver = global_state_receiver.clone();
            let new_urls = new_urls.clone();
            let task = move || {
                producer_task(global_state_receiver, new_urls, producer_tx_clone, i);
            };
            // let worker = Worker::new(task);
            let handle = thread::spawn(task);

            handlers.push(handle);
        }

        let garded_producer_rx = Arc::new(Mutex::new(producer_rx));

        Producer {
            handlers,
            producer_tx,
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
