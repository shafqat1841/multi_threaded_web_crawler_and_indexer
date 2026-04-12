mod consumer_task;

use std::{
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
};

use crossbeam::channel::Receiver;

use crate::entities_system::{
    app_global_state::GlobalState, consumer::consumer_task::ConsumerTask,
    producer::ProducerChannelData,
};

pub struct Consumer {
    handler: JoinHandle<()>,
}

impl Consumer {
    pub fn new(
        guarded_global_state: Arc<Mutex<GlobalState>>,
        producer_rx: Receiver<ProducerChannelData>,
    ) -> Self {
        let producer_rx_clone = producer_rx.clone();
        let guarded_global_state_clone = guarded_global_state.clone();
        let task = move || {
            let consumer_task =
                ConsumerTask::new(guarded_global_state_clone, producer_rx_clone);
            consumer_task.run();
        };

        let handler = thread::Builder::new()
            .name("consumer thread".to_string())
            .spawn(task)
            .unwrap();

        Consumer { handler }
    }

    pub fn check_threads_finished(&self) -> bool {
        self.handler.is_finished()
    }

    pub fn join_thread(self) {
        self.handler.join().unwrap()
    }
}
