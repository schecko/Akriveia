
use actix::prelude::*;
use actix_web::{ Error, Result };
use crate::beacon_serial::*;
use futures::{future::ok, Future};
use serialport::prelude::*;
use serialport;
use std::sync::{ Arc, Mutex, };
use std::thread;
use std::time::Duration;
use std::io;
use std::collections::{ HashMap, BTreeMap };
use na;

// contains a vector of tag data from multiple beacons
#[derive(Debug)]
struct TagHashEntry {
    tag_data_points: Vec<common::TagData>,
}

pub struct DataProcessor {
    // this hash maps the id_tag mac address to data points for that id tag.
    tag_hash: HashMap<String, Box<TagHashEntry>>,
    // TODO support floors
    // TODO init with db data?
    // this tree maps tag mac addresses to users
    // scanning the entire tree for all entries will likely be a very common,
    // so hash is likely not a good choice.
    users: BTreeMap<String, common::User>
}

impl DataProcessor {
    pub fn new() -> DataProcessor {
        DataProcessor {
            tag_hash: HashMap::new(),
            users: BTreeMap::new(),
        }
    }

    fn calc_trilaterate(tag_data: &Vec<common::TagData>) -> na::Vector2<f32> {
        if(tag_data.len() < 3) {
            panic!("not enough data points to trilaterate");
        }
        // TODO move to db
        let bloc1 = na::Vector2::new(0.0, 0.0);
        let bloc2 = na::Vector2::new(3.0, 0.0);
        let bloc3 = na::Vector2::new(0.0, 3.0);

        let env_factor = 2.0;
        let measure_power = -76.0;

        // TODO change calc based on type
        let tag_distance0 = match tag_data[0].tag_distance {
            common::DataType::RSSI(rssi) => rssi,
            common::DataType::TOF(tof) => tof,
        } as f64;
        let tag_distance1 = match tag_data[1].tag_distance {
            common::DataType::RSSI(rssi) => rssi,
            common::DataType::TOF(tof) => tof,
        } as f64;
        let tag_distance2 = match tag_data[2].tag_distance {
            common::DataType::RSSI(rssi) => rssi,
            common::DataType::TOF(tof) => tof,
        } as f64;


        let denom = 10.0 * env_factor;
        let d1 = 10f64.powf((measure_power - tag_distance0) / denom);
        let d2 = 10f64.powf((measure_power - tag_distance1) / denom);
        let d3 = 10f64.powf((measure_power - tag_distance2) / denom);
        println!("distances {} {} {}", d1, d2, d3);

        // Trilateration solver
        let a = -2.0 * bloc1.x + 2.0 * bloc2.x;
        let b = -2.0 * bloc1.y + 2.0 * bloc2.y;
        let c = d1 * d1 - d2 * d2 - bloc1.x * bloc1.x + bloc2.x * bloc2.x - bloc1.y * bloc1.y + bloc2.y * bloc2.y;
        let d = -2.0 * bloc2.x + 2.0 * bloc3.x;
        let e = -2.0 * bloc2.y + 2.0 * bloc3.y;
        let f = d2 * d2 - d3 * d3 - bloc2.x * bloc2.x + bloc3.x * bloc3.x - bloc2.y * bloc2.y + bloc3.y * bloc3.y;

        let x = (c * e - f * b) / (e * a - b * d);
        let y = (c * d - a * f) / (b * d - a * e);
        println!("calc {} {} {} {} {}", tag_distance0, tag_distance1, tag_distance2, x, y);
        na::Vector2::new(x as f32, y as f32)
    }
}

impl Actor for DataProcessor {
    type Context = Context<Self>;
}

pub enum DPMessage {
    LocationData(common::TagData),
}
impl Message for DPMessage {
    type Result = Result<u64, io::Error>;
}

impl Handler<DPMessage> for DataProcessor {
    type Result = Result<u64, io::Error>;

    fn handle (&mut self, msg: DPMessage, _: &mut Context<Self>) -> Self::Result {
        match msg {
            DPMessage::LocationData(tag_data) => {
                if self.tag_hash.contains_key(&tag_data.tag_mac) {
                    if let Some(hash_entry) = self.tag_hash.get_mut(&tag_data.tag_mac) {
                        println!("tag entry exists {}", &tag_data.tag_mac);
                        // replace any existing element, otherwise just add the new element to
                        // prevent duplicates
                        hash_entry.tag_data_points = hash_entry.tag_data_points.iter().filter(|it| it.beacon_mac != tag_data.beacon_mac).cloned().collect();
                        hash_entry.tag_data_points.push(tag_data.clone());

                        if(hash_entry.tag_data_points.len() >= 3) {
                            let tag_location = Self::calc_trilaterate(&hash_entry.tag_data_points);
                            // reset the point data
                            hash_entry.tag_data_points = Vec::new();

                            // update the user information
                            match self.users.get_mut(&tag_data.tag_mac) {
                                Some(user_ref) => {
                                    println!("updating user");
                                    user_ref.location = tag_location;
                                },
                                None => {
                                    // TODO this should probably eventually be an error if the user
                                    // is missing, but for now just make the user instead
                                    println!("creating new user");
                                    let mut user = common::User::new(tag_data.tag_mac.clone());
                                    user.location = tag_location;
                                    self.users.insert(tag_data.tag_mac.clone(), user);
                                }
                            }
                        }
                    }
                } else {
                    println!("creating new tag entry");
                    // create new entry
                    let mut hash_entry = TagHashEntry {
                        tag_data_points: Vec::new(),
                    };
                    hash_entry.tag_data_points.push(tag_data.clone());
                    self.tag_hash.insert(tag_data.tag_mac.clone(), Box::new(hash_entry));
                }
            },
            _ => {
                panic!("unknown message sent to data processor");
            },
        }

        Ok(1)
    }
}

pub struct OutUserData { }

impl Message for OutUserData {
    type Result = Result<Vec<common::User>, io::Error>;
}

impl Handler<OutUserData> for DataProcessor {
    type Result = Result<Vec<common::User>, io::Error>;

    fn handle (&mut self, msg: OutUserData, _: &mut Context<Self>) -> Self::Result {
        Ok(self.users.values().cloned().collect())
    }
}
