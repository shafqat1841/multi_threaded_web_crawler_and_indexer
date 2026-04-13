use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use crossbeam::channel::{Sender};

use crate::{
    constants::SLEEP_DURATION,
    entities_system::{
        app_global_state::{GlobalStateChannelData, GuardedGlobalReceiverType, UrlData},
        producer::{NewUrls, ProducerChannelData},
    },
};

#[derive(Debug)]
enum GetGlobalStateDataErr {
    DisconnectErr,
}

pub struct ProducerTask {
    global_state_receiver: GuardedGlobalReceiverType,
    new_urls: Vec<String>,
    producer_tx: Sender<ProducerChannelData>,
}

impl ProducerTask {
    pub fn new(
        global_state_receiver: GuardedGlobalReceiverType,
        producer_tx: Sender<ProducerChannelData>,
    ) -> Self {
        let new_urls: [String; 15] = [
            "https://www.example2.com".to_string(),
            "https://www.rust-lang2.org".to_string(),
            "https://www.wikipedia2.org".to_string(),
            "https://www.github2.com".to_string(),
            "https://www.stackoverflow2.com".to_string(),
            "https://www.example3.com".to_string(),
            "https://www.rust-lang3.org".to_string(),
            "https://www.wikipedia3.org".to_string(),
            "https://www.github3.com".to_string(),
            "https://www.stackoverflow3.com".to_string(),
            "https://www.example4.com".to_string(),
            "https://www.rust-lang4.org".to_string(),
            "https://www.wikipedia4.org".to_string(),
            "https://www.github4.com".to_string(),
            "https://www.stackoverflow4.com".to_string(),
        ];
        ProducerTask {
            global_state_receiver,
            new_urls: new_urls.to_vec(),
            producer_tx,
        }
    }

    pub fn run(&mut self) {
        println!("Producer thread started");
        loop {
            let global_state_data = self.get_global_state_data();

            let ok_data = match global_state_data {
                Ok(value) => value,
                Err(err) => match err {
                    GetGlobalStateDataErr::DisconnectErr => {
                        println!("  Producer thread encountered an error: {:?}", err);
                        break;
                    }
                },
            };

            match ok_data {
                GlobalStateChannelData::EndProcessing => {
                    println!("Producer thread received EndProcessing signal");
                    break;
                }
                GlobalStateChannelData::ContinueProcessing(data) => {
                    self.update_received_data(data);
                }
            };
        }
        println!("Producer thread ended");
    }

    fn get_global_state_data(&self) -> Result<GlobalStateChannelData, GetGlobalStateDataErr> {
        let res = match self.global_state_receiver.recv() {
            Ok(value) => value,
            Err(_) => {
                return Err(GetGlobalStateDataErr::DisconnectErr);
            }
        };

        Ok(res)
    }

    fn update_received_data(&mut self, data: (Arc<Mutex<UrlData>>, Arc<Mutex<isize>>)) {
        sleep(Duration::from_millis(SLEEP_DURATION));

        let send_data_res = self.send_data_to_consumer();

        match send_data_res {
            Ok(_) => {
                {
                    let mut received_value_0 = data.0.lock().unwrap();

                    received_value_0.visited = true;
                }
                {
                    let mut received_value_1 = data.1.lock().unwrap();
                    *received_value_1 += 1;
                }
            }
            Err(err) => {
                println!("{}", err);
                let mut received_value_0 = data.0.lock().unwrap();

                received_value_0.in_processing = false;
            }
        }
    }

    fn send_data_to_consumer(&mut self) -> Result<(), String> {
        let new_value: ProducerChannelData = self.get_new_urls_two_values();

        if let Err(err) = self.producer_tx.send(new_value) {
            let error = err.0;

            match error {
                ProducerChannelData::ContinueProcessing(err_new_urls) => {
                    if let Some(value) = err_new_urls.urls1 {
                        let u1 = value;
                        self.new_urls.push(u1);
                    };

                    if let Some(value) = err_new_urls.urls2 {
                        let u2 = value;
                        self.new_urls.push(u2);
                    };
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

    fn get_new_urls_two_values(&mut self) -> ProducerChannelData {
        let new_urls = &mut self.new_urls;

        let u1 = new_urls.pop();
        let u2 = new_urls.pop();

        let is_empty = new_urls.is_empty() && u1.is_none() && u2.is_none();
        let payload = if !is_empty {
            ProducerChannelData::ContinueProcessing(NewUrls {
                urls1: u1,
                urls2: u2,
            })
        } else {
            ProducerChannelData::EndProcessing
        };

        payload
    }
}
