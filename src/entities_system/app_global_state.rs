use std::sync::{Arc, Mutex};

use crossbeam::channel::{Receiver, SendError, Sender, unbounded};
use dashmap::DashMap;

use crate::constants::{INITIAL_URLS, MAX_URLS_TO_PROCESS, THREAD_COUNT};

#[derive(Debug)]
pub struct UrlData {
    // pub url: String,
    // pub content: String,
    pub visited: bool,
    pub in_processing: bool,
}

#[derive(Debug)]
pub struct GuardedUrlDataType(pub String, pub Arc<Mutex<isize>>);

#[derive(Debug)]
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
    pub urls_data: Arc<DashMap<String, UrlData>>,
    pub url_visited: Arc<Mutex<isize>>,
    pub global_state_tx: GlobalSenderType,
    pub global_state_rx_array: Mutex<Vec<Receiver<GlobalStateChannelData>>>,
}

impl GlobalState {
    pub fn new() -> Self {
        let (global_state_tx, global_state_rx) = unbounded::<GlobalStateChannelData>();

        let mut global_state_rx_array: Vec<Receiver<GlobalStateChannelData>> = Vec::new();

        for _ in 0..THREAD_COUNT {
            let global_state_rx_clone = global_state_rx.clone();
            global_state_rx_array.push(global_state_rx_clone);
        }

        let urls_data: DashMap<String, UrlData> = DashMap::new();
        let url_visited = 0;
        INITIAL_URLS.iter().for_each(|url| {
            let url_data = UrlData {
                visited: false,
                in_processing: false,
            };
            urls_data.insert(url.to_string(), url_data);
        });

        let garded_url_visited = Arc::new(Mutex::new(url_visited));

        let urls_data_arc = Arc::new(urls_data);

        GlobalState {
            urls_data: urls_data_arc,
            url_visited: garded_url_visited,
            global_state_tx,
            // quarded_global_state_rx,
            global_state_rx_array: Mutex::new(global_state_rx_array),
        }
    }

    pub fn get_unvisited_url(&self) -> Option<String> {
        let unvisited_url = self.urls_data.iter_mut().find(|item| {
            let item_value = item.value();

            let res = !item_value.visited && !item_value.in_processing;

            res
        });
        let value = unvisited_url.map(|mut item| {
            item.value_mut().in_processing = true;
            item.key().clone()
        });
        value
    }

    pub fn send_data_to_producer(&self) -> Result<(), &str> {
        let unvisited_url = match self.get_unvisited_url() {
            Some(found) => found,
            None => {
                return Ok(());
            }
        };

        let unvisited_url_clone = unvisited_url.clone();

        let send_data = GuardedUrlDataType(unvisited_url_clone, Arc::clone(&self.url_visited));

        if let Err(e) = self
            .global_state_tx
            .send(GlobalStateChannelData::ContinueProcessing(send_data))
        {
            eprintln!("Channel send failed for {}: {:?}", unvisited_url, e);

            if let Some(mut value) = self.urls_data.get_mut(&unvisited_url) {
                value.in_processing = false;
            };
        };
        Ok(())
    }

    pub fn send_end_process_signal(&self) -> Result<(), SendError<GlobalStateChannelData>> {
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
            let visited = url_data.value().visited;
            let in_processing = url_data.value().in_processing;
            let res = visited && in_processing;
            res
        });
        res
    }

    pub fn is_all_urls_visiting_done(&self) -> bool {
        let max_url_visited = self.is_max_url_visited();
        let all_url_visited = self.are_all_url_visited();
        max_url_visited && all_url_visited
    }
}
