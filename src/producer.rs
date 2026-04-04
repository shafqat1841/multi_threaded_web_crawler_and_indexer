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
    constants::{MAX_URLS_TO_PROCESS, THREAD_COUNT, sleep_duration},
};

pub struct Producer {
    total_processed: Arc<Mutex<isize>>,
    handlers: Vec<JoinHandle<()>>,
    producer_tx: Sender<Option<()>>,
    global_state_rx: Arc<Mutex<Receiver<Option<()>>>>,
}

impl Producer {
    pub fn new(producer_tx: Sender<Option<()>>, global_state_rx: Receiver<Option<()>>) -> Self {
        let handlers: Vec<JoinHandle<()>> = Vec::new();
        let receiver = Arc::new(Mutex::new(global_state_rx));
        let total_processed = Arc::new(Mutex::new(0));

        Producer {
            total_processed,
            handlers,
            producer_tx,
            global_state_rx: receiver,
        }
    }

    pub fn run(&mut self) {
        for i in 0..THREAD_COUNT {
            println!("Producer thread {} is sending signal to GlobalState...", i);
            self.producer_tx
                .send(Some(()))
                .expect("Failed to send signal to producer thread");
            let receiver_clone = Arc::clone(&self.global_state_rx);
            let total_processed_clone = Arc::clone(&self.total_processed);
            let handler = thread::spawn(move || {
                let received_data = receiver_clone.lock().unwrap().recv();
                let mut total_processed = total_processed_clone.lock().unwrap();
                match received_data {
                    Ok(data) => {
                        println!("Producer thread {} received: {:?}", i, data);
                        // Simulate work by sleeping for a short duration
                        sleep(Duration::from_millis(sleep_duration));
                        println!("Producer thread {} has finished", i);
                        *total_processed += 1;
                    }
                    Err(_) => println!("Producer thread {} channel closed", i),
                }
            });
            self.handlers.push(handler);
        }
    }
}

impl Drop for Producer {
    fn drop(&mut self) {
        for handler in self.handlers.drain(..) {
            println!("Joining producer thread and dropping producer sender...");
            handler.join().expect("Producer thread panicked");
        }
    }
}
