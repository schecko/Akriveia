
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
use std::collections::HashMap;

// contains a vector of tag data from multiple beacons
#[derive(Debug)]
struct TagHashEntry {
    tag_data_points: Vec<common::TagData>,
}

pub struct DataProcessor {
    // this hash maps the id_tag mac address to data points for that id tag.
    tag_hash: HashMap<String, Box<TagHashEntry>>,
}

impl DataProcessor {
    pub fn new() -> DataProcessor {
        DataProcessor {
            tag_hash: HashMap::new(),
        }
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
                println!("hello processor");
                if self.tag_hash.contains_key(&tag_data.tag_mac) {
                    // append the data
                    if let Some(hash_entry) = self.tag_hash.get_mut(&tag_data.tag_mac) {
                        hash_entry.tag_data_points.push(tag_data.clone());
                    }
                } else {
                    // create new entry
                    let mut hash_entry = TagHashEntry {
                        tag_data_points: Vec::new(),
                    };
                    hash_entry.tag_data_points.push(tag_data.clone());
                    self.tag_hash.insert(tag_data.tag_mac.clone(), Box::new(hash_entry));
                }
                println!("tag hash {:?}", self.tag_hash);
            },
            _ => {
                println!("eek");
            },
        }

        Ok(1)

    }
}
