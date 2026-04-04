mod app_global_state;
mod constants;
mod producer;

use std::sync::mpsc::channel;

use crate::{
    app_global_state::{GlobalState, UrlData},
    producer::Producer,
};

pub fn run() {
    let (global_state_tx, global_state_rx) = channel::<Option<()>>();
    let (producer_tx, producer_rx) = channel::<Option<()>>();

    let mut global_state = GlobalState::new(global_state_tx, producer_rx);
    let mut producer = Producer::new(producer_tx, global_state_rx);
    producer.run();
    global_state.run();
}
