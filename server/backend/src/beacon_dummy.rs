
extern crate rand;

use actix::prelude::*;
use crate::beacon_manager::*;
use rand::Rng;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use common::MacAddress;

const MESSAGE_INTERVAL: Duration = Duration::from_millis(1000);
const NUM_USERS: u32 = 5;
const MIN_RSSI_DISTANCE: i64 = -82;
const MAX_RSSI_DISTANCE: i64 = -50;

pub fn dummy_beacon_thread(dummy_id: u32,
                           receive: mpsc::Receiver<BeaconCommand>,
                           beacon_manager: Addr<BeaconManager>)
{
    println!("starting dummy beacon {}", dummy_id);

    let mut rng = rand::thread_rng();
    loop {
        // wait for instructions to start.
        loop {
            match receive.recv() {
                Ok(BeaconCommand::StartEmergency) => {
                    break;
                },
                _ => { },
            }
        }

        loop {
            // see if there is a command to abort
            match receive.try_recv() {
                Ok(BeaconCommand::EndEmergency) => {
                    // stop sending data
                    break;
                },
                Err(mpsc::TryRecvError::Empty) => { } // do nothing, dont care
                Err(mpsc::TryRecvError::Disconnected) => {
                    println!("dummy {} was disconnected from manager", dummy_id);
                },
                _ => { },
            }

            // start generating data
            let user_number = rng.gen_range(0, NUM_USERS);
            let user_distance = rng.gen_range(MIN_RSSI_DISTANCE, MAX_RSSI_DISTANCE);
            beacon_manager
                .do_send( TagDataMessage {
                    data: common::TagData {
                        tag_name: format!("user_{}", user_number),
                        tag_mac: MacAddress::from_bytes(&[0, 0, 0, 0, 0, user_number as u8]).unwrap(),
                        tag_distance: common::DataType::RSSI(user_distance),
                        beacon_mac: MacAddress::from_bytes(&[0, 0, 0, 0, 0, dummy_id as u8]).unwrap(),
                    }
                });
            thread::sleep(MESSAGE_INTERVAL);
        }

    }
}
