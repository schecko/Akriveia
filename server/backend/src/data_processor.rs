
use actix::prelude::*;
use actix::fut as afut;
use actix_web::Result;
use common::{ MacAddress8, ShortAddress, };
use crate::db_utils;
use crate::models::beacon;
use crate::models::user;
use futures::future as fut;
use na;
use std::collections::{ BTreeMap, VecDeque };
use std::io;
use common::*;
use chrono::{ Utc, };

const LOCATION_HISTORY_SIZE: usize = 5;

// contains a vector of tag data from multiple beacons
#[derive(Debug)]
struct TagHistory {
    pub user: RealtimeUserData,
    pub beacon_history: BTreeMap<MacAddress8, VecDeque<f64>>,
}

pub struct DataProcessor {
    // this hash maps the id_tag mac address to data points for that id tag.
    // TODO support floors
    // TODO init with db data?
    // this tree maps tag mac addresses to users
    // scanning the entire tree for all entries will likely be a very common,
    // so hash is likely not a good choice.
    users: BTreeMap<ShortAddress, Box<TagHistory>>,
    beacons: BTreeMap<MacAddress8, Beacon>,
}

impl DataProcessor {
    pub fn new() -> DataProcessor {
        DataProcessor {
            beacons: BTreeMap::new(),
            users: BTreeMap::new(),
        }
    }

    fn calc_trilaterate(beacons: &Vec<common::Beacon>, tag_data: &Vec<common::TagData>) -> na::Vector2<f64> {
        if tag_data.len() < 3 {
            panic!("not enough data points to trilaterate");
        }
        if beacons.len() < 3 {
            panic!("not enough beacons to trilaterate");
        }

        let (sorted_beacons, sorted_data): (Vec<_>, Vec<_>) = beacons.iter().map(|b| {
            (b, tag_data.iter().find(|t| t.beacon_mac == b.mac_address).unwrap())
        }).unzip();
        assert!(sorted_beacons[0].mac_address == sorted_data[0].beacon_mac);
        assert!(sorted_beacons[1].mac_address == sorted_data[1].beacon_mac);
        assert!(sorted_beacons[2].mac_address == sorted_data[2].beacon_mac);

        let bloc1 = sorted_beacons[0].coordinates;
        let bloc2 = sorted_beacons[1].coordinates;
        let bloc3 = sorted_beacons[2].coordinates;

        let d1 = sorted_data[0].tag_distance;
        let d2 = sorted_data[1].tag_distance;
        let d3 = sorted_data[2].tag_distance;

        // Trilateration solver
        let a = -2.0 * bloc1.x + 2.0 * bloc2.x;
        let b = -2.0 * bloc1.y + 2.0 * bloc2.y;
        let c = d1 * d1 - d2 * d2 - bloc1.x * bloc1.x + bloc2.x * bloc2.x - bloc1.y * bloc1.y + bloc2.y * bloc2.y;
        let d = -2.0 * bloc2.x + 2.0 * bloc3.x;
        let e = -2.0 * bloc2.y + 2.0 * bloc3.y;
        let f = d2 * d2 - d3 * d3 - bloc2.x * bloc2.x + bloc3.x * bloc3.x - bloc2.y * bloc2.y + bloc3.y * bloc3.y;

        let x = (c * e - f * b) / (e * a - b * d);
        let y = (c * d - a * f) / (b * d - a * e);

        na::Vector2::new(x as f64, y as f64)
    }
}

impl Actor for DataProcessor {
    type Context = Context<Self>;
}

pub enum DPMessage {
    ResetData, // Reset the stored data
}
impl Message for DPMessage {
    type Result = Result<u64, io::Error>;
}

impl Handler<DPMessage> for DataProcessor {
    type Result = Result<u64, io::Error>;

    fn handle (&mut self, msg: DPMessage, _: &mut Context<Self>) -> Self::Result {
        match msg {
            DPMessage::ResetData => {
                self.beacons.clear();
                self.users.clear();
            },
        }

        Ok(1)
    }
}

pub struct InLocationData(pub common::TagData);

impl Message for InLocationData {
    type Result = Result<(), ()>;
}
fn append_history(tag_entry: &mut Box<TagHistory>, tag_data: &common::TagData) {
    if let Some(beacon_entry) = tag_entry.beacon_history.get_mut(&tag_data.beacon_mac) {
        beacon_entry.push_back(tag_data.tag_distance);
        if beacon_entry.len() > LOCATION_HISTORY_SIZE {
            beacon_entry.pop_front();
        }
    } else {
        let mut deque = VecDeque::new();
        deque.push_back(tag_data.tag_distance);
        tag_entry.beacon_history.insert(tag_data.beacon_mac.clone(), deque);
    }
    tag_entry.user.last_active = tag_data.timestamp;
}

impl Handler<InLocationData> for DataProcessor {
    type Result = ResponseActFuture<Self, (), ()>;

    fn handle (&mut self, msg: InLocationData, _: &mut Context<Self>) -> Self::Result {
        let tag_data = msg.0.clone();
        let tag_data_update = msg.0;

        // append the data to in memory structures,
        // then if there are enough data points, return them in opt_averages
        // so that they can be used to trilaterate
        let prep_fut = match self.users.get_mut(&tag_data.tag_mac) {
            Some(mut tag_entry) => {
                append_history(&mut tag_entry, &tag_data);

                // TODO pick the most recent 3 beacons
                if tag_entry.beacon_history.len() >= 3 {
                    let averaged_data: Vec<common::TagData> = tag_entry.beacon_history.iter().map(|(beacon_mac, hist_vec)| {
                        common::TagData {
                            tag_mac: tag_entry.user.addr.clone(),
                            beacon_mac: beacon_mac.clone(),
                            tag_distance: hist_vec.into_iter().sum::<f64>() / hist_vec.len() as f64,
                            timestamp: tag_entry.user.last_active,
                        }
                    }).collect();

                    afut::Either::A(fut::ok(averaged_data).into_actor(self))
                } else {
                    afut::Either::A(fut::err(()).into_actor(self))
                }
            },
            None => {
                // create new entry
                let tag_mac = tag_data.tag_mac;

                afut::Either::B(db_utils::default_connect()
                    .and_then(move |client| {
                        user::select_user_by_short(client, tag_mac)
                    })
                    .map_err(|_x| {})
                    .into_actor(self)
                    .map(move |(_client, opt_user), actor, _context| {
                        match opt_user {
                            Some(u) => {
                                let mut hash_entry = TagHistory {
                                    user: RealtimeUserData::from(u),
                                    beacon_history: BTreeMap::new(),
                                };

                                let mut deque = VecDeque::new();
                                deque.push_back(tag_data.tag_distance);
                                hash_entry.beacon_history.insert(tag_data.beacon_mac.clone(), deque);
                                actor.users.insert(tag_data.tag_mac.clone(), Box::new(hash_entry));
                                fut::err::<Vec<TagData>, _>(()).into_actor(actor)
                            },
                            None => {
                                // user doesn't exist, cannot continue processing.
                                println!("tag {} does not have an associated user, make one", tag_data.tag_mac);
                                fut::err::<Vec<TagData>, _>(()).into_actor(actor)
                            }
                        }
                    })
                    .and_then(|x, _, _| x)
                )

            },
        };

        let fut = prep_fut
            .and_then(move |averages, actor, _context| {
                // perform trilateration
                let beacon_macs: Vec<MacAddress8> = averages
                    .iter()
                    .map(|tagdata| tagdata.beacon_mac)
                    .collect();

                let fut = db_utils::default_connect()
                    .and_then(|client| {
                        beacon::select_beacons_by_mac(client, beacon_macs)
                    })
                    .into_actor(actor)
                    .map_err(|_e, _, _| {})
                    .and_then(move |(client, beacons), actor, _context| {
                        // perform trilateration calculation
                        let mut beacon_sources: Vec<BeaconTOFToUser> = Vec::new();
                        let new_tag_location = Self::calc_trilaterate(&beacons, &averages);
                        let timestamp = averages.iter().fold(Utc.timestamp(0, 0), |max, tag_point| {
                            if max < tag_point.timestamp {
                                tag_point.timestamp
                            } else {
                                max
                            }
                        });
                        let map_id = beacons[0].map_id; // TODO HACK.

                        averages.iter().for_each(|tag_data| {
                            let beacon = beacons.iter().find(|beacon| beacon.mac_address == tag_data.beacon_mac).unwrap();
                            beacon_sources.push(BeaconTOFToUser {
                                name: beacon.name.clone(),
                                location: beacon.coordinates,
                                distance_to_tag: tag_data.tag_distance,
                            });
                        });

                        // update the user information
                        let update_db_fut = match actor.users.get_mut(&tag_data_update.tag_mac) {
                            Some(hist) => {
                                hist.user.beacon_tofs = beacon_sources;
                                hist.user.coordinates = new_tag_location;
                                hist.user.last_active = timestamp;
                                hist.user.map_id = map_id;
                                afut::Either::A(
                                    user::update_user_from_realtime(client, hist.user.clone())
                                        .map_err(|_e| {})
                                        .map(|(_client, _opt_user)| { })
                                        .into_actor(actor)
                                )
                            },
                            None => {
                                // if the user doesnt exist by now, then there is a tag without an
                                // associated user sending us data, just ignore it
                                afut::Either::B(afut::err(()))
                            }
                        };

                        update_db_fut
                    });
                fut
            })
            .map(|_, _actor, _context| {
            })
            .map_err(|_, _actor, _context| {
            });
        Box::new(fut)
    }
}

pub struct OutUserData { }

impl Message for OutUserData {
    type Result = Result<Vec<RealtimeUserData>, io::Error>;
}

impl Handler<OutUserData> for DataProcessor {
    type Result = Result<Vec<RealtimeUserData>, io::Error>;

    fn handle (&mut self, _msg: OutUserData, _: &mut Context<Self>) -> Self::Result {
        Ok(self.users.iter().map(|(_addr, hist)| hist.user.clone()).collect())
    }
}
