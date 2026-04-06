use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
};

use crate::constants::INITIAL_URLS;

#[derive(Debug)]
pub struct UrlData {
    pub url: String,
    pub content: String,
    pub visited: bool,
    pub in_processing: bool,
}

type GuardedUrlDataType = (Arc<Mutex<UrlData>>, Arc<Mutex<isize>>);
type GlobalSenderType = Sender<Option<GuardedUrlDataType>>;
pub type GlobalReceiverType = Receiver<Option<GuardedUrlDataType>>;
pub type GuardedGlobalReceiverType = Arc<Mutex<GlobalReceiverType>>;

#[derive(Debug)]
pub struct GlobalState {
    // The Coordinator (Shared State): A central record that keeps track of which URLs have already been visited
    // so you don't crawl the same page twice.
    pub url_visited: Arc<Mutex<isize>>,
    pub urls_data: HashMap<String, Arc<Mutex<UrlData>>>,
    // pub urls_data_gauarded: Arc<Mutex<HashMap<String, UrlData>>>,
    // pub urls_data: HashMap<String, UrlData>,
    pub global_state_tx: GlobalSenderType,
    // pub global_state_rx: Receiver<Option<isize>>,
    pub quarded_global_state_rx: GuardedGlobalReceiverType,
}

impl GlobalState {
    pub fn new() -> Self {
        // let (global_state_tx, global_state_rx) = channel::<Option<GuardedUrlDataType>>();
        let (global_state_tx, global_state_rx) = channel::<Option<GuardedUrlDataType>>();

        let mut urls_data: HashMap<String, Arc<Mutex<UrlData>>> = HashMap::new();
        let url_visited = 0;
        INITIAL_URLS.iter().for_each(|url| {
            let url_data = UrlData {
                url: url.to_string(),
                content: String::new(),
                visited: false,
                in_processing: false,
            };
            let url_data_quarded = Arc::new(Mutex::new(url_data));
            urls_data.insert(url.to_string(), url_data_quarded);
        });

        // let urls_data_gauarded = Arc::new(Mutex::new(urls_data));

        let quarded_global_state_rx = Arc::new(Mutex::new(global_state_rx));
        let garded_url_visited = Arc::new(Mutex::new(url_visited));

        GlobalState {
            urls_data,
            // urls_data_gauarded,
            url_visited: garded_url_visited,
            global_state_tx,
            quarded_global_state_rx,
        }
    }
}
