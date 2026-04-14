use std::sync::{Arc, Mutex};

use crossbeam::channel::Receiver;
use dashmap::DashMap;

use crate::entities_system::{
    app_global_state::{GlobalState, UrlData},
    producer::{ProducerChannelData, ProducerErr},
};

pub struct ConsumerTask {
    producer_rx: Receiver<ProducerChannelData>,
    urls_data: Arc<DashMap<String, UrlData>>,
}

impl ConsumerTask {
    pub fn new(
        guarded_global_state: Arc<GlobalState>,
        producer_rx: Receiver<ProducerChannelData>,
    ) -> Result<Self, ProducerErr> {
        let urls_data = guarded_global_state.urls_data.clone();

        Ok(ConsumerTask {
            producer_rx,
            urls_data
        })
    }

    pub fn run(&self) {
        println!("Consumer loop start");
        loop {
            let rec_res = match self.producer_rx.recv() {
                Ok(value) => value,
                Err(_) => {
                    println!("consumer loop break here 1");
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
                    println!("consumer loop break here 2");
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

        self.urls_data.insert(val, new_data);

        // match self.guarded_global_state.lock() {
        //     Err(err) => {
        //         println!("guarded_global_state.lock error: {}", err)
        //     }
        //     Ok(mut global_state) => {
        //         let urls_data = &mut global_state.urls_data;

        //         // urls_data.insert(val, arc_data);
        //         urls_data.insert(val, new_data);
        //     }
        // }
    }
}
