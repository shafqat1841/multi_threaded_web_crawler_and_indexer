use std::{
    mem,
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
    pub consumer_rx: Receiver<Option<String>>,
}

impl Consumer {
    pub fn new(producer_rx: Arc<Mutex<Receiver<Option<NewUrls>>>>) -> Self {
        let (consumer_tx, consumer_rx) = channel::<Option<String>>();
        let producer_rx_clone = producer_rx.clone();
        let task = move || {
            loop {
                // println!("consumer loop");
                // let rec = producer_rx_clone.lock().unwrap().try_recv().unwrap();
                let rec_lock = { producer_rx_clone.lock() };
                let rec_lock_res = match rec_lock {
                    Ok(lock_res) => lock_res,
                    Err(err) => {
                        eprintln!("lock_res error: {:?}", err);
                        match consumer_tx.send(None) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("Fconsumer_tx.send err: {:?}", err);
                            }
                        }
                        continue;
                    }
                };

                let rec_lock_res_recv = rec_lock_res.recv() ;

                let rec = match rec_lock_res_recv {
                    Ok(value) => value,
                    Err(err) => {
                        eprintln!("rec_lock_res_recv error: {:?}", err);
                        match consumer_tx.send(None) {
                            Ok(()) => {}
                            Err(err) => {
                                eprintln!("Fconsumer_tx.send err: {:?}", err);
                            }
                        }
                        continue;
                    }
                };

                match rec {
                    Some(value) => {
                        let url_to_process_1 = value.urls1;
                        println!("Consumer processing url 1: {:?}", url_to_process_1);
                        let url_to_process_2 = value.urls2;
                        println!("Consumer processing url 2: {:?}", url_to_process_2);
                        match consumer_tx.send(url_to_process_1) {
                            Ok(()) => {}
                            Err(_) => {
                                eprintln!("Failed to send url 1 to consumer");
                            }
                        }
                        match consumer_tx.send(url_to_process_2) {
                            Ok(()) => {}
                            Err(_) => {
                                eprintln!("Failed to send url 2 to consumer");
                            }
                        }
                    }
                    None => {
                        println!("Consumer received None");
                        break;
                    }
                }
            }
        };

        let handler = thread::Builder::new()
            .name("consumer thread".to_string())
            .spawn(task)
            .unwrap();

        Consumer {
            producer_rx,
            handler,
            consumer_rx,
        }
    }

    pub fn check_threads_finished(&self) -> bool {
        self.handler.is_finished()
    }

    pub fn join_thread(self) {
        self.handler.join().unwrap()
    }
}
