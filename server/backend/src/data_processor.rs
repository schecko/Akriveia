
use actix::prelude::*;
use actix::fut as afut;
use actix_web::Result;
use common::{ MacAddress8, ShortAddress, };
use crate::db_utils;
use crate::models::beacon;
use crate::models::user;
use futures::future as fut;
use na;
use std::collections::{ HashMap, BTreeMap, VecDeque };
use std::io;
use common::*;

const LOCATION_HISTORY_SIZE: usize = 5;

// contains a vector of tag data from multiple beacons
#[derive(Debug)]
struct TagHistory {
    pub tag_mac: ShortAddress,
    pub beacon_history: BTreeMap<MacAddress8, VecDeque<f64>>,
}

pub struct DataProcessor {
    // this hash maps the id_tag mac address to data points for that id tag.
    tag_hash: HashMap<ShortAddress, Box<TagHistory>>,
    // TODO support floors
    // TODO init with db data?
    // this tree maps tag mac addresses to users
    // scanning the entire tree for all entries will likely be a very common,
    // so hash is likely not a good choice.
    users: BTreeMap<ShortAddress, RealtimeUserData>
}

impl DataProcessor {
    pub fn new() -> DataProcessor {
        DataProcessor {
            tag_hash: HashMap::new(),
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

        let bloc1 = beacons[0].coordinates;
        let bloc2 = beacons[1].coordinates;
        let bloc3 = beacons[2].coordinates;

        let d1 = tag_data[0].tag_distance;
        let d2 = tag_data[1].tag_distance;
        let d3 = tag_data[2].tag_distance;

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
                self.tag_hash.clear();
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
}

impl Handler<InLocationData> for DataProcessor {
    type Result = ResponseActFuture<Self, (), ()>;

    fn handle (&mut self, msg: InLocationData, _: &mut Context<Self>) -> Self::Result {
        let tag_data = msg.0;

        // append the data to in memory structures,
        // then if there are enough data points, return them in opt_averages
        // so that they can be used to trilaterate
        let opt_averages = match self.tag_hash.get_mut(&tag_data.tag_mac) {
            Some(mut tag_entry) => {
                append_history(&mut tag_entry, &tag_data);

                // TODO pick the most recent 3 beacons
                if tag_entry.beacon_history.len() >= 3 {
                    let averaged_data: Vec<common::TagData> = tag_entry.beacon_history.iter().map(|(beacon_mac, hist_vec)| {
                        common::TagData {
                            tag_mac: tag_entry.tag_mac.clone(),
                            beacon_mac: beacon_mac.clone(),
                            tag_distance: hist_vec.into_iter().sum::<f64>() / hist_vec.len() as f64,
                        }
                    }).collect();

                    Some(averaged_data)
                } else {
                    None
                }
            },
            None => {
                // create new entry
                let mut hash_entry = TagHistory {
                    tag_mac: tag_data.tag_mac.clone(),
                    beacon_history: BTreeMap::new(),
                };
                let mut deque = VecDeque::new();
                deque.push_back(tag_data.tag_distance);
                hash_entry.beacon_history.insert(tag_data.beacon_mac.clone(), deque);
                self.tag_hash.insert(tag_data.tag_mac.clone(), Box::new(hash_entry));
                None
            },
        };

        let fut = match opt_averages {
            // perform trilateration
            Some(averages) => {
                let beacon_macs: Vec<MacAddress8>  = averages.iter().map(|tagdata| tagdata.beacon_mac).collect();
                afut::Either::A(db_utils::default_connect()
                    .and_then(|client| {
                        beacon::select_beacons_by_mac(client, beacon_macs)
                    })
                    .into_actor(self)
                    .and_then(move |(client, beacons), actor, _context| {
                        // perform trilateration calculation
                        let mut beacon_sources: Vec<BeaconTOFToUser> = Vec::new();
                        let new_tag_location = Self::calc_trilaterate(&beacons, &averages);

                        averages.iter().for_each(|tag_data| {
                            let beacon = beacons.iter().find(|beacon| beacon.mac_address == tag_data.beacon_mac).unwrap();
                            beacon_sources.push(BeaconTOFToUser {
                                name: beacon.name.clone(),
                                location: beacon.coordinates,
                                distance_to_tag: tag_data.tag_distance,
                            });
                        });

                        // ensure the user exists in memory, go to db if we must.
                        let tag_mac = tag_data.tag_mac.clone();
                        let fetch_user_fut = match actor.users.contains_key(&tag_data.tag_mac) {
                            true => afut::Either::A(fut::ok::<_, tokio_postgres::Error>(client).into_actor(actor)),
                            false => {
                                afut::Either::B(user::select_user_by_short(client, tag_data.tag_mac)
                                    .into_actor(actor)
                                    .map(move |(client, opt_user), actor, _context| {
                                        match opt_user {
                                            Some(u) => {
                                                let user_data = RealtimeUserData::from(u);
                                                actor.users.insert(user_data.addr.clone(), user_data);
                                            },
                                            None => {
                                                // user doesn't exist, cannot continue processing.
                                                println!("tag {} does not have an associated user, make one", tag_mac);
                                            }
                                        }
                                        client
                                    })
                                )
                            }
                        };

                        fetch_user_fut
                            .map(move |client, actor, context| {
                                (client, tag_data.tag_mac, new_tag_location, beacon_sources)
                            })
                        //afut::ok((client, tag_data.tag_mac, new_tag_location, beacon_sources))
                    })
                    .and_then(|(client, tag_addr, new_tag_location, beacon_sources), actor, context| {
                        // update the user information
                        match actor.users.get_mut(&tag_addr) {
                            Some(user) => {
                                user.beacon_tofs = beacon_sources;
                                user.coordinates = new_tag_location;
                                afut::Either::A(user::update_user_from_realtime(client, user.clone())
                                    .map(|(_client, _opt_user)| { })
                                    .into_actor(actor)
                                )
                            },
                            None => {
                                // if the user doesnt exist by now, then there is a tag without an
                                // associated user sending us data, just ignore it
                                afut::Either::B(afut::ok(()))
                            }
                        }
                    })
                )
            },
            // not enough data, do nothing for now
            None => {
                afut::Either::B(fut::ok(()).into_actor(self))
            }
        };

        Box::new(fut.map_err(|_postgres_err, _, _| { }))
    }
}

pub struct OutUserData { }

impl Message for OutUserData {
    type Result = Result<Vec<RealtimeUserData>, io::Error>;
}

impl Handler<OutUserData> for DataProcessor {
    type Result = Result<Vec<RealtimeUserData>, io::Error>;

    fn handle (&mut self, _msg: OutUserData, _: &mut Context<Self>) -> Self::Result {
        Ok(self.users.values().cloned().collect())
    }
}
