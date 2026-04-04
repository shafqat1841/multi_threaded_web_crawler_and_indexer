use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use crate::{
    app_global_state::UrlData,
    constants::{MAX_URLS_TO_PROCESS, SLEEP_DURATION, THREAD_COUNT},
};

#[derive(Debug)]
pub struct Producer {
    pub handlers: Vec<JoinHandle<()>>,
    pub producer_sender: Arc<Mutex<Sender<Option<(isize)>>>>,
    pub producer_rx: Receiver<Option<(isize)>>,
}

impl Producer {
    pub fn new(global_state_receiver: &Arc<Mutex<Receiver<Option<(isize)>>>>) -> Self {
        let (producer_tx, producer_rx) = channel::<Option<(isize)>>();
        let mut handlers: Vec<JoinHandle<()>> = Vec::new();
        let producer_sender = Arc::new(Mutex::new(producer_tx));

        for i in 0..THREAD_COUNT {
            let global_state_receiver_clone = Arc::clone(global_state_receiver);
            let producer_tx_clone = Arc::clone(&producer_sender);

            let handler = thread::spawn(move || {
                loop {
                    producer_tx_clone
                        .lock()
                        .unwrap()
                        .send(Some(i))
                        .expect("Failed to send from producer thread");
                    let received_data = global_state_receiver_clone.lock().unwrap().recv();
                    match received_data {
                        Ok(data) => {
                            match data {
                                Some(value) => {
                                    println!("Producer thread {} received: {:?}", i, value);
                                    // Simulate work by sleeping for a short duration
                                    sleep(Duration::from_millis(SLEEP_DURATION));
                                    println!("Producer thread {} has finished", i);
                                }
                                None => {
                                    println!(
                                        "Producer thread {} received termination signal. Exiting...",
                                        i
                                    );
                                    break;
                                }
                            }
                        }
                        Err(_) => {
                            println!("Producer thread {} channel closed", i);
                            break;
                        }
                    }
                }
            });

            handlers.push(handler);
        }

        Producer {
            handlers,
            producer_sender,
            producer_rx,
        }
    }

    pub fn check_threads_finished(&self) -> bool {
        self.handlers.iter().all(|handler| handler.is_finished())
    }

    pub fn join_threads(&mut self) {
        for handler in self.handlers.drain(..) {
            println!("Joining producer thread...");
            handler.join().expect("Producer thread panicked");
        }
    }
}
