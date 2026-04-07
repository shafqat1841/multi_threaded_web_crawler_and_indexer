use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use crate::{
    constants::{NEW_URLS, NewUnReadUrl, SLEEP_DURATION, THREAD_COUNT},
    entities_system::{app_global_state::GuardedGlobalReceiverType, consumer::Consumer},
};

#[derive(Debug)]
pub struct NewUrls {
    pub urls1: Option<String>,
    pub urls2: Option<String>,
}

#[derive(Debug)]
pub struct Producer {
    pub handlers: Vec<Worker>,
    pub producer_tx: Sender<Option<NewUrls>>,
    pub garded_producer_rx: Arc<Mutex<Receiver<Option<NewUrls>>>>,
}

#[derive(Debug)]
pub struct Worker {
    thread: JoinHandle<()>,
}

impl Worker {
    pub fn new<F>(task: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let thread = thread::spawn(task);
        Worker { thread }
    }

    pub fn is_finished(&self) -> bool {
        self.thread.is_finished()
    }

    pub fn join(self) {
        self.thread.join().unwrap();
    }
}

impl Producer {
    pub fn new(global_state_receiver: GuardedGlobalReceiverType) -> Self {
        let mut handlers: Vec<Worker> = Vec::new();
        let (producer_tx, producer_rx) = channel::<Option<NewUrls>>();

        let new_urls = Arc::new(Mutex::new(NEW_URLS.to_vec()));

        for i in 0..THREAD_COUNT {
            let producer_tx_clone = producer_tx.clone();
            let global_state_receiver_clone = global_state_receiver.clone();
            let new_urls_clone = new_urls.clone();
            let task = move || {
                println!("thread {} started", i);
                loop {
                    println!("task started");
                    let global_state_data = {
                        let global_state_locked = global_state_receiver_clone.lock().unwrap();
                        global_state_locked.recv().unwrap()
                    };
                    match global_state_data {
                        Some(value) => {
                            let mut received_value_0 = { value.0.lock().unwrap() };
                            sleep(Duration::from_millis(SLEEP_DURATION));
                            let mut new_urls_locked = { new_urls_clone.lock().unwrap() };
                            if !new_urls_locked.is_empty() {
                                let url_1: Option<String> =
                                    new_urls_locked.pop().map(|s| s.to_string());
                                let url_2: Option<String> =
                                    new_urls_locked.pop().map(|s| s.to_string());

                                let new_value = Some(NewUrls {
                                    urls1: url_1,
                                    urls2: url_2,
                                });
                                producer_tx_clone.send(new_value).unwrap();
                            } else {
                                producer_tx_clone.send(None).unwrap();
                            }
                            println!(
                                "new_urls_clone len: {}",
                                new_urls_clone.lock().unwrap().len()
                            );
                            received_value_0.visited = true;
                            let mut received_value_1 = { value.1.lock().unwrap() };
                            *received_value_1 += 1;
                            println!("task ended");
                        }
                        None => {
                            println!("None value");
                            let mut new_urls_locked = { new_urls_clone.lock().unwrap() };
                            if !new_urls_locked.is_empty() {
                                let url_1: Option<String> =
                                    new_urls_locked.pop().map(|s| s.to_string());
                                let url_2: Option<String> =
                                    new_urls_locked.pop().map(|s| s.to_string());

                                let new_value = Some(NewUrls {
                                    urls1: url_1,
                                    urls2: url_2,
                                });
                                producer_tx_clone.send(new_value).unwrap();
                            } else {
                                producer_tx_clone.send(None).unwrap();
                                println!("No more URLs to add, breaking the loop");
                                break;
                            }
                        }
                    }
                }
                println!("thread {} ended", i);
            };
            let worker = Worker::new(task);

            handlers.push(worker);
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
            worker.join();
        }
    }
}
