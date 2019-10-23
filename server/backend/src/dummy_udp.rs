extern crate actix;
extern crate tokio;
extern crate futures;
extern crate bytes;

use actix::prelude::*;
use actix::{ Actor, Context, StreamHandler, };
use crate::db_utils;
use crate::beacon_manager::*;
use crate::conn_common;
use std::io;

pub struct DummyUDP {
    manager: Addr<BeaconManager>,
    state: Option<BeaconCommand>,
    data_task: SpawnHandle,
}

impl Actor for DummyUDP {
    type Context = Context<Self>;
}

#[derive(Message)]
struct Internal { }

impl Handler<Internal> for DummyUDP {
    type Result = ResponseActFuture<Self, (), ()>;

    fn handle(&mut self, msg: Internal, _: &mut Context<Self>) {
        let beacon_manager_addr = self.manager.clone();
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


        self.manager
            .do_send(TagDataMessage { data: msg });
    }
}

impl Handler<BeaconCommand> for DummyUDP {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: BeaconCommand, context: &mut Context<Self>) -> Self::Result {
        match msg {
            BeaconCommand::StartEmergency => {
                self.data_task = context.run_interval(Duration::from_millis(1000), |actor, context| {
                    context.notify(Internal);
                });
            },
            BeaconCommand::EndEmergency => {
                context.cancel_future(self.data_task);
            },
            BeaconCommand::Ping => {
            },
            _ => {
            }
        }

        Ok(())
    }
}

impl DummyUDP {
    pub fn new(manager: Addr<BeaconManager>) -> Addr<DummyUDP> {
        DummyUDP::create(move |context| {
            DummyUDP {
                manager,
            }
        })
    }
}
