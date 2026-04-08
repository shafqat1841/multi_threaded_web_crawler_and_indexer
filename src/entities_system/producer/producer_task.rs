use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use crate::{
    constants::SLEEP_DURATION,
    entities_system::app_global_state::{ChannelData, GuardedGlobalReceiverType},
};

#[derive(Debug)]
enum GetGlobalStateDataErr {
    DisconnectErr,
    PoisonErr,
}

type GetGlobalStateDataRes<'a> = Result<ChannelData, GetGlobalStateDataErr>;

fn get_global_state_data(
    global_state_receiver: &GuardedGlobalReceiverType,
    i: isize,
) -> GetGlobalStateDataRes<'_> {
    let global_state_locked = global_state_receiver.lock();

    let lock = match global_state_locked {
        Ok(guard) => guard,
        Err(poisoned) => {
            eprintln!(
                "Producer thread {} failed to lock global state receiver: {:?}",
                i, poisoned
            );
            return Err(GetGlobalStateDataErr::PoisonErr);
        }
    };

    let res = match lock.try_recv() {
        Ok(value) => value,
        Err(err) => {
            eprintln!(
                "Producer thread {} failed to receive data from global state: {:?}",
                i, err
            );
            if err == std::sync::mpsc::TryRecvError::Empty {
                return Ok(ChannelData::EndProcessing);
            }

            return Err(GetGlobalStateDataErr::DisconnectErr);
        }
    };

    Ok(res)
}

pub fn producer_task(
    global_state_receiver: GuardedGlobalReceiverType,
    new_urls: Arc<Mutex<Vec<&str>>>,
    i: isize,
) {
    println!("thread {} started", i);
    loop {
        let global_state_data = get_global_state_data(&global_state_receiver, i);
        let ok_data = match global_state_data {
            Ok(value) => value,
            Err(err) => {
                println!("  Producer thread {} encountered an error: {:?}", i, err);
                break;
            }
        };

        match ok_data {
            ChannelData::EndProcessing => {
                println!("Producer thread {} received EndProcessing signal", i);
                break;
            }
            ChannelData::ContinueProcessing(data) => {
                sleep(Duration::from_millis(SLEEP_DURATION));

                {
                    let mut received_value_0 = data.0.lock().unwrap();

                    received_value_0.visited = true;
                }
                {
                    let mut received_value_1 = data.1.lock().unwrap();
                    *received_value_1 += 1;
                }
            }
        };
    }
    println!("thread {} ended", i);
}

// pub fn producer_task(
//     global_state_receiver: GlobalStateReceiverCloneType,
//     new_urls: Arc<Mutex<Vec<&str>>>,
//     i: isize,
// ) {
//     println!("thread {} started", i);
//     loop {
//         println!("task started");
//         let global_state_data = get_global_state_data(&global_state_receiver, i);
//         match global_state_data {
//             Some(value) => {
//                 sleep(Duration::from_millis(SLEEP_DURATION));

//                 // let mut new_urls_locked = { new_urls.lock().unwrap() };
//                 // if !new_urls_locked.is_empty() {
//                 //     let url_1: Option<String> =
//                 //         new_urls_locked.pop().map(|s| s.to_string());
//                 //     let url_2: Option<String> =
//                 //         new_urls_locked.pop().map(|s| s.to_string());
//                 //     let new_value = Some(NewUrls {
//                 //         urls1: url_1,
//                 //         urls2: url_2,
//                 //     });
//                 //     producer_tx_clone.send(new_value).unwrap();
//                 // } else {
//                 //     producer_tx_clone.send(None).unwrap();
//                 // }
//                 {
//                     let mut received_value_0 = value.0.lock().unwrap();

//                     received_value_0.visited = true;
//                 }
//                 {
//                     let mut received_value_1 = value.1.lock().unwrap();
//                     *received_value_1 += 1;
//                 }
//                 println!("task ended");
//             }
//             None => {
//                 println!("None value");
//                 // break;

//                 // let mut new_urls_locked = { new_urls.lock().unwrap() };
//                 // if !new_urls_locked.is_empty() {
//                 //     let url_1: Option<String> =
//                 //         new_urls_locked.pop().map(|s| s.to_string());
//                 //     let url_2: Option<String> =
//                 //         new_urls_locked.pop().map(|s| s.to_string());
//                 //     let new_value = Some(NewUrls {
//                 //         urls1: url_1,
//                 //         urls2: url_2,
//                 //     });
//                 //     producer_tx_clone.send(new_value).unwrap();
//                 // } else {
//                 //     producer_tx_clone.send(None).unwrap();
//                 //     println!("No more URLs to add, breaking the loop");
//                 //     break;
//                 // }
//             }
//         }
//     }
//     println!("thread {} ended", i);
// }
