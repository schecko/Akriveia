
// this "manager" exists for two main purposes, first the initial implementation will involve
// serial communcation with the beacons, which will be synchronous. Since Actix is a single threaded
// event loop this would not be a good time for our webserver, so instead the work will be shoved
// off to "SyncArbitors" which gives an api like normal actix, but gives each actor(or each beacon
// in this case) its own communcation thread. The second reason is to hopefully abstract this
// functionality a little bit, so that it will be a little bit easier to move to the wireless
// implementation.
use std::sync::{Arc, Mutex};
use actix::prelude::*;
use serialport::prelude::*;
use serialport::*;
use std::time::Duration;
use crate::beacon_serial::*;
use futures::future::*;

#[derive(Message)]
pub struct MyMessage(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
pub struct Connect {
    pub addr: Recipient<MyMessage>,
}

pub struct BeaconManager {
    pub serial_connections: Vec<Addr<BeaconSerialConn>>,
}

const BAUD_RATE: u32 = 9600;
impl BeaconManager {
    pub fn new() -> BeaconManager {

        let mut res = BeaconManager {
            serial_connections: Vec::new(),
        };
        res.find_beacons();
        res
    }

    fn find_beacons(&mut self) {
        if let Ok(avail_ports) = serialport::available_ports() {
            for port in avail_ports {
                println!("\t{}", port.port_name);
                let name = Box::new(port.port_name);
                match port.port_type {
                    SerialPortType::UsbPort(info) => {
                        // only print out, and keep track of, arduino usbs
                        if info.vid == 0x2341 {
                            println!("\t\tType: USB");
                            println!("\t\tVID:{:04x}", info.vid);
                            println!("\t\tPID:{:04x}", info.pid);
                            println!("\t\tSerial Number: {}", info.serial_number.as_ref().map_or("", String::as_str));
                            println!("\t\tManufacturer: {}", info.manufacturer.as_ref().map_or("", String::as_str));
                            println!("\t\tProduct: {}", info.product.as_ref().map_or("", String::as_str));

                            self.serial_connections.push(SyncArbiter::start(1, move || {
                                BeaconSerialConn {
                                    port_name: (*name).clone(), //(&port.port_name).clone(),
                                    vid: info.vid,
                                    pid: info.pid,
                                }
                            }));
                        }
                    }
                    _ => {}
                }
            }
        } else {
            print!("Error listing serial ports");
        }
    }
}

pub struct ScanForBeacons;
impl Message for ScanForBeacons {
    type Result = Result<u64>;
}

pub struct StartEmergency;
impl Message for StartEmergency {
    type Result = Result<u64>;
}

/*
struct EndEmergency;
impl Message for EndEmergency {
    type Result = Result<u64>;
}
*/

impl Actor for BeaconManager {
    type Context = Context<Self>;
}

impl Handler<ScanForBeacons> for BeaconManager {
    type Result = Result<u64>;

    fn handle(&mut self, msg: ScanForBeacons, _: &mut Context<Self>) -> Self::Result {
        // find the beacons
        println!("find beacons called!");
        self.find_beacons();
        Ok(1)
    }
}

impl Handler<StartEmergency> for BeaconManager {
    type Result = Result<u64>;

    fn handle(&mut self, msg: StartEmergency, context: &mut Context<Self>) -> Self::Result {
        for connection in &self.serial_connections {
            connection.do_send(StartDataCollection);
        }

        context.run_interval(Duration::from_millis(1000), |a: &mut BeaconManager, context: &mut Context<BeaconManager>| {

            println!("hello get beacon data");
            for connection in &a.serial_connections {
                connection.do_send(GetBeaconData);
            }
        });
        Ok(1)
    }
}

/*
impl Handler<EndEmergency> for BeaconManager {
    type Result = Vec<Future<Item = (), Error = ()>;

    fn handle(&mut self, msg: EndEmergency, _: &mut Context<Self>) -> Self::Result {
        for connection in &self.serial_connections {
            connection.do_send(StartDataCollection);
        }

        Ok(1)
    }
} */
