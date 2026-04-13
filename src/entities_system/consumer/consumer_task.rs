use std::sync::{Arc, Mutex};

use crossbeam::channel::{Receiver};

use crate::entities_system::{
    app_global_state::{GlobalState, UrlData},
    producer::ProducerChannelData,
};

pub struct ConsumerTask {
    producer_rx: Receiver<ProducerChannelData>,
    guarded_global_state: Arc<Mutex<GlobalState>>,
}

impl ConsumerTask {
    pub fn new(
        guarded_global_state: Arc<Mutex<GlobalState>>,
        producer_rx: Receiver<ProducerChannelData>,
    ) -> Self {
        ConsumerTask {
            producer_rx,
            guarded_global_state,
        }
    }

    pub fn run(&self) {
        println!("Consumer loop start");
        loop {
            let rec_res = match self.producer_rx.recv() {
                Ok(value) => value,
                Err(_) => {
                    break;
                }
            };

            match rec_res {
                ProducerChannelData::ContinueProcessing(new_urls) => {
                    let url_to_process_1 = new_urls.urls1;
                    let url_to_process_2 = new_urls.urls2;

                    let urls_to_process = [url_to_process_1, url_to_process_2];

                    urls_to_process.into_iter().for_each(|item| match item {
                        None => {}
                        Some(val) => {
                            self.insert_new_value(val);
                        }
                    });
                }

                ProducerChannelData::EndProcessing => {
                    break;
                }
            }
        }
        println!("Consumer loop end");
    }

    fn insert_new_value(&self, val: String) {
        let new_data: UrlData = UrlData {
            in_processing: false,
            visited: false,
        };

        let arc_data = Arc::new(Mutex::new(new_data));

        match self.guarded_global_state.lock() {
            Err(err) => {
                println!("guarded_global_state.lock error: {}", err)
            }
            Ok(mut global_state) => {
                let urls_data = &mut global_state.urls_data;

                urls_data.insert(val, arc_data);
            }
        }
    }
}
