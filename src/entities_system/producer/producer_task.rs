use std::{
    mem::take,
    sync::{Arc, Mutex, MutexGuard, mpsc::Sender},
    thread::sleep,
    time::Duration,
};

use crate::{
    constants::SLEEP_DURATION,
    entities_system::{
        app_global_state::{ChannelData, GuardedGlobalReceiverType, UrlData},
        producer::NewUrls,
    },
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

fn get_new_urls_two_values(new_urls: Arc<Mutex<Vec<String>>>) -> Result<NewUrls, String> {
    match new_urls.lock() {
        Ok(mut urls) => {
            let u1 = urls.pop();
            let u2 = urls.pop();

            let is_empty = urls.is_empty();
            let payload = if !is_empty {
                NewUrls {
                    urls1: u1,
                    urls2: u2,
                }
            } else {
                NewUrls {
                    urls1: None,
                    urls2: None,
                }
            };

            Ok(payload)
        }
        Err(_) => {
            // eprintln!("Failed to lock new_urls");
            Err("Failed to lock new_urls".to_string())
        }
    }
}

fn send_data_to_consumer(new_urls: Arc<Mutex<Vec<String>>>, producer_tx_clone: Sender<NewUrls>) {
    let new_value = match get_new_urls_two_values(new_urls.clone()) {
        Err(err) => {
            println!("{}", err);
            return;
        }
        Ok(value) => value,
    };

    if let Err(err) = producer_tx_clone.send(new_value) {
        eprintln!("Failed to send data to consumer: {:?}", err);
        let err_new_urls = err.0;

        match new_urls.lock() {
            Err(err) => {
                println!("err: {}", err);
            }
            Ok(mut new_urls_lock) => {
                if let Some(value) = err_new_urls.urls1 {
                    let u1 = value;
                    new_urls_lock.push(u1);
                };

                if let Some(value) = err_new_urls.urls2 {
                    let u2 = value;
                    new_urls_lock.push(u2);
                };
            }
        };
    }
}

fn update_received_data(
    data: (Arc<Mutex<UrlData>>, Arc<Mutex<isize>>),
    new_urls: Arc<Mutex<Vec<String>>>,
    producer_tx_clone: Sender<NewUrls>,
) {
    sleep(Duration::from_millis(SLEEP_DURATION));

    send_data_to_consumer(new_urls, producer_tx_clone);

    {
        let mut received_value_0 = data.0.lock().unwrap();

        received_value_0.visited = true;
    }
    {
        let mut received_value_1 = data.1.lock().unwrap();
        *received_value_1 += 1;
    }
}

pub fn producer_task(
    global_state_receiver: GuardedGlobalReceiverType,
    new_urls: Arc<Mutex<Vec<String>>>,
    producer_tx: Sender<NewUrls>,
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
                update_received_data(data, new_urls.clone(), producer_tx.clone());
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
