use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, TryRecvError},
};

use crate::entities_system::{
    app_global_state::{GlobalState, UrlData},
    producer::ProducerChannelData,
};

pub struct ConsumerTask {
    producer_rx: Arc<Mutex<Receiver<ProducerChannelData>>>,
    guarded_global_state: Arc<Mutex<GlobalState>>,
}

impl ConsumerTask {
    pub fn new(
        guarded_global_state: Arc<Mutex<GlobalState>>,
        producer_rx: Arc<Mutex<Receiver<ProducerChannelData>>>,
    ) -> Self {
        ConsumerTask {
            producer_rx,
            guarded_global_state,
        }
    }

    pub fn run(&self) {
        println!("Consumer loop start");
        loop {
            let rec_lock = {
                match self.producer_rx.lock() {
                    Ok(lock_res) => lock_res,
                    Err(err) => {
                        eprintln!("lock_res error: {:?}", err);
                        break;
                    }
                }
            };

            let rec_res = match rec_lock.try_recv() {
                Ok(value) => value,
                Err(err) => {
                    // eprintln!("rec_lock_res_recv error: {:?}", err);
                    // println!("rec_lock_res_recv error: {:?}", err);
                    if err == TryRecvError::Empty {
                        continue;
                    }
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
            content: "".to_string(),
            // url: val.clone(),
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
