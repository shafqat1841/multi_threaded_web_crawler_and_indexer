mod app_global_state;
mod constants;
mod producer;

use crate::app_global_state::GlobalState;

pub fn run() {
    let mut global_state = GlobalState::new();
    global_state.run();
}
