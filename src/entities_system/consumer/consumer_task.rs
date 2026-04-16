use std::sync::{Arc, atomic::AtomicIsize};

use crossbeam::channel::{Receiver, Sender};
use dashmap::DashMap;

use crate::entities_system::{
    app_global_state::{GlobalState, GlobalStateChannelData, GuardedUrlDataType, UrlData},
    producer::{ProducerChannelData, ProducerErr},
};

pub struct ConsumerTask {
    producer_rx: Receiver<ProducerChannelData>,
    urls_data: Arc<DashMap<String, UrlData>>,
    global_state_tx: Sender<GlobalStateChannelData>,
}

impl ConsumerTask {
    pub fn new(
        guarded_global_state: Arc<GlobalState>,
        producer_rx: Receiver<ProducerChannelData>,
    ) -> Result<Self, ProducerErr> {
        let urls_data = guarded_global_state.urls_data.clone();
        let global_state_tx = guarded_global_state.global_state_tx.clone();
        Ok(ConsumerTask {
            producer_rx,
            urls_data,
            global_state_tx,
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
                ProducerChannelData::ContinueProcessing(new_urls, url_visisted) => {
                    let url_to_process_1 = new_urls.urls1;
                    let url_to_process_2 = new_urls.urls2;

                    let urls_to_process = [url_to_process_1, url_to_process_2];

                    urls_to_process.into_iter().for_each(|item| match item {
                        None => {}
                        Some(val) => {
                            self.insert_new_value(val, url_visisted.clone());
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

    fn insert_new_value(&self, val: String, url_visisted: Arc<AtomicIsize>) {
        let urls_data = &self.urls_data;

        let res = urls_data.get(&val);

        if let None = res {
            let new_data: UrlData = UrlData { visited: false };

            self.urls_data.insert(val.clone(), new_data);

            let send_data: GuardedUrlDataType = GuardedUrlDataType(val, url_visisted);

            let msg: GlobalStateChannelData = GlobalStateChannelData::ContinueProcessing(send_data);

            let send_res = self.global_state_tx.send(msg);

            match send_res {
                Ok(_) => {}
                Err(err) => {
                    println!(
                        "file: consumer_task.rs ~ line 84 ~ ifletErr ~ err : {} ",
                        err
                    );
                }
            }
        }
    }
}
