
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

pub struct DataProcessor {
}

impl DataProcessor {
    pub fn new() -> DataProcessor {
        DataProcessor {}
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
            },
            _ => {
                println!("eek");
            },
        }

        Ok(1)

    }
}
