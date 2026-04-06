use std::{
    sync::mpsc::{Receiver, Sender, channel},
    thread::{self, JoinHandle, sleep},
    time::Duration,
};

use crate::{
    constants::{SLEEP_DURATION, THREAD_COUNT},
    entities_system::app_global_state::GuardedGlobalReceiverType,
};

#[derive(Debug)]
pub struct Producer {
    pub handlers: Vec<Worker>,
    pub producer_tx: Sender<Option<isize>>,
    pub producer_rx: Receiver<Option<isize>>,
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
        let (producer_tx, producer_rx) = channel::<Option<isize>>();

        for i in 0..THREAD_COUNT {
            let global_state_receiver_clone = global_state_receiver.clone();
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
                            received_value_0.visited = true;
                            let mut received_value_1 = { value.1.lock().unwrap() };
                            *received_value_1 += 1;
                            println!("task ended");
                        }
                        None => {
                            println!("None value");
                            break;
                        }
                    }
                }
                println!("thread {} ended", i);
            };
            let worker = Worker::new(task);

            handlers.push(worker);
        }

        Producer {
            handlers,
            producer_tx,
            producer_rx,
        }
    }

    pub fn check_threads_finished(&self) -> bool {
        self.handlers.iter().all(|handler| handler.is_finished())
    }

    pub fn join_threads(&mut self) {
        for worker in self.handlers.drain(..) {
            println!("Joining producer thread...");
            worker.join();
        }
    }
}
