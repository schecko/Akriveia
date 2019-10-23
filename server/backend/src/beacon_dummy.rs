
extern crate rand;

use actix::prelude::*;
use crate::beacon_manager::*;
use rand::Rng;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use crate::models::user;
use crate::db_utils;
use common::*;
use futures::future::ok;

const MESSAGE_INTERVAL: Duration = Duration::from_millis(1000);
const MIN_DISTANCE: f64 = 1.0;
const MAX_DISTANCE: f64 = 4.0;

pub fn dummy_beacon_thread(beacon: Beacon,
                           receive: mpsc::Receiver<BeaconCommand>,
                           beacon_manager: Addr<BeaconManager>)
{
    println!("starting dummy beacon {}", &beacon.id);

    let mut rng = rand::thread_rng();
    loop {
        // wait for instructions to start.
        loop {
            match receive.recv() {
                Ok(BeaconCommand::StartEmergency) => {
                    break;
                },
                Ok(BeaconCommand::Ping) => {
                    println!("hello health check non emergency");
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
                Ok(BeaconCommand::Ping) => {
                    println!("hello health check in emergency");
                },
                Err(mpsc::TryRecvError::Empty) => { } // do nothing, dont care
                Err(mpsc::TryRecvError::Disconnected) => {
                    println!("dummy {} was disconnected from manager", &beacon.id);
                },
                _ => { },
            }

            let beacon_manager_addr = beacon_manager.clone();
            let user_distance = rng.gen_range(MIN_DISTANCE, MAX_DISTANCE);
            let beacon_mac = beacon.mac_address.clone();

            let data_gen_fut = db_utils::default_connect()
                .and_then(|client| {
                    // select_user_random must return a user with a valid address
                    user::select_user_random(client)
                })
                .and_then(move |(_client, opt_user)| {
                    if let Some(user) = opt_user {
                        beacon_manager_addr
                            .do_send( TagDataMessage {
                                data: common::TagData {
                                    beacon_mac: beacon_mac,
                                    tag_distance: user_distance,
                                    tag_mac: user.mac_address.unwrap(),
                                    timestamp: Utc::now(),
                                }
                            });
                    }
                    ok(())
                })
                .map_err(|_err| {
                });

            tokio::run(data_gen_fut);

            thread::sleep(MESSAGE_INTERVAL);
        }

    }
}
