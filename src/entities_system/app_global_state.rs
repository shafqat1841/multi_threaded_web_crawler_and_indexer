use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crossbeam::channel::{Receiver, SendError, Sender, unbounded};

use crate::constants::{INITIAL_URLS, MAX_URLS_TO_PROCESS, THREAD_COUNT};

#[derive(Debug)]
pub struct UrlData {
    // pub url: String,
    // pub content: String,
    pub visited: bool,
    pub in_processing: bool,
}

type GuardedUrlDataType = (Arc<Mutex<UrlData>>, Arc<Mutex<isize>>);
pub enum GlobalStateChannelData {
    ContinueProcessing(GuardedUrlDataType),
    EndProcessing,
}

type GlobalSenderType = Sender<GlobalStateChannelData>;

pub type GuardedGlobalReceiverType = Receiver<GlobalStateChannelData>;

#[derive(Debug)]
pub struct GlobalState {
    // The Coordinator (Shared State): A central record that keeps track of which URLs have already been visited
    // so you don't crawl the same page twice.
    pub url_visited: Arc<Mutex<isize>>,
    pub urls_data: HashMap<String, Arc<Mutex<UrlData>>>,
    pub global_state_tx: GlobalSenderType,
    pub global_state_rx_array: Vec<Receiver<GlobalStateChannelData>>
}

impl GlobalState {
    pub fn new() -> Self {
        let (global_state_tx, global_state_rx) = unbounded::<GlobalStateChannelData>();

        let mut global_state_rx_array: Vec<Receiver<GlobalStateChannelData>> = Vec::new();
        
        for _ in 0..THREAD_COUNT {
            let global_state_rx_clone = global_state_rx.clone();
            global_state_rx_array.push(global_state_rx_clone);
        }

        let mut urls_data: HashMap<String, Arc<Mutex<UrlData>>> = HashMap::new();
        let url_visited = 0;
        INITIAL_URLS.iter().for_each(|url| {
            let url_data = UrlData {
                visited: false,
                in_processing: false,
            };
            let url_data_quarded = Arc::new(Mutex::new(url_data));
            urls_data.insert(url.to_string(), url_data_quarded);
        });

        // let quarded_global_state_rx = Arc::new(Mutex::new(global_state_rx));
        let garded_url_visited = Arc::new(Mutex::new(url_visited));

        GlobalState {
            urls_data,
            url_visited: garded_url_visited,
            global_state_tx,
            // quarded_global_state_rx,
            global_state_rx_array,
        }
    }

    pub fn get_unvisited_url(&self) -> Option<(&String, &Arc<Mutex<UrlData>>)> {
        let unvisited_url = self.urls_data.iter().find(|item| {
            let url_info = {
                match item.1.lock() {
                    Ok(value) => Some(value),
                    Err(_) => {
                        eprintln!("Failed to lock url data for url: {:?}", item.0);
                        None
                    }
                }
            };
            let res = match url_info {
                Some(value) => !value.visited && !value.in_processing,
                None => false,
            };
            res
        });
        if let Some(value) = unvisited_url {
            println!("unvisited_url: {:?}", value.0);
        }
        unvisited_url
    }

    pub fn send_data_to_producer(&self) {
        let (url_key, url_arc) = match self.get_unvisited_url() {
            Some(found) => found,
            None => {
                return;
            }
        };

        {
            match url_arc.lock() {
                Ok(mut data) => data.in_processing = true,
                Err(e) => {
                    eprintln!("Poison error locking {}: {:?}", url_key, e);
                    let _ = self
                        .global_state_tx
                        .send(GlobalStateChannelData::EndProcessing);
                    return; // Can't process if lock is poisoned
                }
            }
        }

        let send_data = (Arc::clone(url_arc), Arc::clone(&self.url_visited));

        if let Err(e) = self
            .global_state_tx
            .send(GlobalStateChannelData::ContinueProcessing(send_data))
        {
            eprintln!("Channel send failed for {}: {:?}", url_key, e);

            if let Ok(mut data) = url_arc.lock() {
                data.in_processing = false;
            }
        }
    }

    pub fn send_end_process_signal(
        &self,
    ) -> Result<(), SendError<GlobalStateChannelData>> {
        let send_res = self
            .global_state_tx
            .send(GlobalStateChannelData::EndProcessing);
        if let Err(err) = send_res {
            println!("All global state recivers are disconnected");
            return Err(err);
        }
        Ok(())
    }

    fn is_max_url_visited(&self) -> bool {
        let url_visited = self.url_visited.lock().unwrap().clone();
        url_visited >= MAX_URLS_TO_PROCESS
    }

    fn are_all_url_visited(&self) -> bool {
        let res = self.urls_data.iter().all(|url_data| {
            let url_data_lock = url_data.1.lock().unwrap();
            url_data_lock.visited && url_data_lock.in_processing
        });
        res
    }

    pub fn is_all_urls_visiting_done(&self) -> bool {
        self.is_max_url_visited() && self.are_all_url_visited()
    }
}
