use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    thread::{self, JoinHandle},
};

use crate::entities_system::producer::NewUrls;

pub struct Consumer {
    handler: JoinHandle<()>,
    producer_rx: Arc<Mutex<Receiver<Option<NewUrls>>>>,
    consumer_rx: Receiver<Option<String>>,
}

impl Consumer {
    pub fn new(producer_rx: Arc<Mutex<Receiver<Option<NewUrls>>>>) -> Self {
        let (consumer_tx, consumer_rx) = channel::<Option<String>>();
        let producer_rx_clone = producer_rx.clone();
        let task = move || {
            loop {
                let rec = producer_rx_clone.lock().unwrap().recv().unwrap();

                match rec {
                    Some(value) => {
                        println!("Consumer received value: {:?}", value);
                    }
                    None => {
                        println!("Consumer received None");
                        break;
                    }
                }
            }
        };
        let handler = thread::spawn(task);

        Consumer {
            producer_rx,
            handler,
            consumer_rx,
        }
    }
}
