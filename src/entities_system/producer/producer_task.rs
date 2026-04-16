use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicIsize, Ordering},
    },
    thread::sleep,
    time::Duration,
};

use crossbeam::channel::Sender;
use dashmap::DashMap;

use crate::{
    constants::SLEEP_DURATION,
    entities_system::{
        app_global_state::{
            GlobalState, GlobalStateChannelData, GuardedGlobalReceiverType, GuardedUrlDataType,
            UrlData,
        },
        producer::{NewUrls, ProducerChannelData, ProducerErr},
    },
};

pub struct ProducerTask {
    urls_data: Arc<DashMap<String, UrlData>>,
    global_state_receiver: GuardedGlobalReceiverType,
    new_urls: Arc<Mutex<Vec<String>>>,
    producer_tx: Sender<ProducerChannelData>,
    threat_name: String,
}

impl ProducerTask {
    pub fn new(
        guarded_global_state: Arc<GlobalState>,
        producer_tx: Sender<ProducerChannelData>,
        threat_name: String,
        new_urls_gearded: Arc<Mutex<Vec<String>>>,
    ) -> Result<Self, ProducerErr> {
        let (urls_data, global_state_receiver) = {
            let rx = {
                let mut rx_array = guarded_global_state
                    .global_state_rx_array
                    .lock()
                    .map_err(|_| ProducerErr::GlobalStateRxNoneErr)?;

                let res = rx_array.pop();

                res
            };

            let global_state_receiver = match rx {
                None => return Err(ProducerErr::GlobalStateRxNoneErr),
                Some(value) => value,
            };

            let urls_data = guarded_global_state.urls_data.clone();

            (urls_data, global_state_receiver)
        };

        Ok(ProducerTask {
            urls_data,
            global_state_receiver,
            new_urls: new_urls_gearded,
            producer_tx,
            threat_name,
        })
    }

    pub fn run(&mut self) {
        println!("Producer thread started");
        loop {
            let ok_data = match self.global_state_receiver.recv() {
                Ok(value) => value,
                Err(err) => {
                    println!("Producer thread encountered an error: : {} ", err);
                    break;
                }
            };

            match ok_data {
                GlobalStateChannelData::EndProcessing => {
                    println!("{} received EndProcessing signal", self.threat_name);
                    break;
                }
                GlobalStateChannelData::ContinueProcessing(data) => {
                    self.update_received_data(data);
                }
            };
        }
        println!("{} ended", self.threat_name);
    }

    fn update_received_data(&mut self, data: GuardedUrlDataType) {
        sleep(Duration::from_millis(SLEEP_DURATION));

        let url_visisted = data.1;

        let send_data_res = self.send_data_to_consumer(url_visisted.clone());

        match send_data_res {
            Ok(_) => {
                let unvisited_url_key = data.0;
                if let Some(mut value) = self.urls_data.get_mut(&unvisited_url_key) {
                    if !value.value().visited {
                        value.value_mut().visited = true;

                        url_visisted.fetch_add(1, Ordering::Relaxed);
                    }
                };
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }

    fn send_data_to_consumer(&mut self, url_visisted: Arc<AtomicIsize>) -> Result<(), String> {
        let new_value: ProducerChannelData = self.get_new_urls_two_values(url_visisted.clone());

        if let Err(err) = self.producer_tx.send(new_value) {
            let error = err.0;

            url_visisted.fetch_sub(1, Ordering::Relaxed);

            match error {
                ProducerChannelData::ContinueProcessing(err_new_urls, url_visisted) => {
                    let mut values = Vec::new();
                    if let Some(value) = err_new_urls.urls1 {
                        let u1 = value;
                        values.push(u1);
                    };

                    if let Some(value) = err_new_urls.urls2 {
                        let u2 = value;
                        values.push(u2);
                    };

                    if !values.is_empty() {
                        self.new_urls_add(values);
                    }

                    url_visisted.fetch_sub(1, Ordering::Relaxed);

                    return Err("Fail to send producer value".to_string());
                }
                ProducerChannelData::EndProcessing => {
                    return Err(
                        "Fail to send end process signal from producer to consumer".to_string()
                    );
                }
            }
        }
        Ok(())
    }

    fn new_urls_add(&self, values: Vec<String>) {
        let new_urls = &self.new_urls;
        match new_urls.lock() {
            Err(err) => {
                println!("err: {}", err);
            }
            Ok(mut lock) => {
                for value in values {
                    lock.push(value);
                }
            }
        }
    }

    fn get_new_urls_two_values(&self, url_visisted: Arc<AtomicIsize>) -> ProducerChannelData {
        let new_urls = &self.new_urls;

        match new_urls.lock() {
            Err(err) => {
                println!("err: {}", err);
                ProducerChannelData::EndProcessing
            }
            Ok(mut lock) => {
                let u1 = lock.pop();
                let u2 = lock.pop();

                let is_empty = lock.is_empty() && u1.is_none() && u2.is_none();
                let payload = if !is_empty {
                    let new_urls_to_send = NewUrls {
                        urls1: u1,
                        urls2: u2,
                    };

                    ProducerChannelData::ContinueProcessing(new_urls_to_send, url_visisted)
                } else {
                    println!("{} end signal to consumer", self.threat_name);
                    ProducerChannelData::EndProcessing
                };

                payload
            }
        }
    }
}
