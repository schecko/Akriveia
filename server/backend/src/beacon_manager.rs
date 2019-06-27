
// this "manager" exists for two main purposes, first the initial implementation will involve
// serial communcation with the beacons, which will be synchronous. Since Actix is a single threaded
// event loop this would not be a good time for our webserver, so instead the work will be shoved
// off to "SyncArbitors" which gives an api like normal actix, but gives each actor(or each beacon
// in this case) its own communcation thread. The second reason is to hopefully abstract this
// functionality a little bit, so that it will be a little bit easier to move to the wireless
// implementation.
use actix::prelude::*;
use actix_web::{ Error, Result };
use crate::beacon_serial::*;
use crate::beacon_serial::*;
use futures::{future::ok, Future};
use serialport::prelude::*;
use serialport;
use std::sync::mpsc;
use std::sync::{ Arc, Mutex, };
use std::thread;
use std::time::Duration;
use std::io;

#[derive(Default)]
pub struct BeaconManager {
    pub serial_connections: Vec<mpsc::Sender<BeaconCommand>>,
    pub diagnostic_data: common::DiagnosticData,
}

const VENDOR_WHITELIST: &[u16] = &[0x2341, 0x10C4];

impl BeaconManager {
    pub fn new() -> BeaconManager {

        let mut res = BeaconManager {
            serial_connections: Vec::new(),
            diagnostic_data: common::DiagnosticData::new(),

        };
        res
    }

    fn find_beacons(&mut self, context: &mut Context<Self>) {
        if let Ok(avail_ports) = serialport::available_ports() {
            for port in avail_ports {
                println!("\t{}", port.port_name);
                let name = Box::new(port.port_name);
                match port.port_type {
                    serialport::SerialPortType::UsbPort(info) => {
                        // only print out, and keep track of, arduino usbs
                        if VENDOR_WHITELIST.iter().any( |vid| vid == &info.vid ) {
                            println!("\t\tType: USB");
                            println!("\t\tVID:{:04x}", info.vid);
                            println!("\t\tPID:{:04x}", info.pid);
                            println!("\t\tSerial Number: {}", info.serial_number.as_ref().map_or("", String::as_str));
                            println!("\t\tManufacturer: {}", info.manufacturer.as_ref().map_or("", String::as_str));
                            println!("\t\tProduct: {}", info.product.as_ref().map_or("", String::as_str));



                            let (serial_send, serial_receive): (mpsc::Sender<BeaconCommand>, mpsc::Receiver<BeaconCommand>) = mpsc::channel();
                            let address = context.address().recipient();
                            let beacon_info = BeaconSerialConn {
                                port_name: (*name).clone(),
                                vid: info.vid,
                                pid: info.pid,
                                receive: serial_receive,
                                manager: address.clone(),

                            };
                            thread::spawn(move || {
                                serial_beacon_thread(beacon_info);
                            });
                            self.serial_connections.push(serial_send);
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
    type Result = Result<u64, io::Error>;
}

pub struct StartEmergency;
impl Message for StartEmergency {
    type Result = Result<u64, io::Error>;
}

pub struct GetDiagnosticData;
impl Message for GetDiagnosticData {
    type Result = Result<common::DiagnosticData, io::Error>;
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
    type Result = Result<u64, io::Error>;

    fn handle(&mut self, msg: ScanForBeacons, context: &mut Context<Self>) -> Self::Result {
        // find the beacons
        println!("find beacons called!");
        self.find_beacons(context);
        Ok(1)
    }
}

impl Handler<GetDiagnosticData> for BeaconManager {
    type Result = Result<common::DiagnosticData, io::Error>;

    fn handle(&mut self, msg: GetDiagnosticData, context: &mut Context<Self>) -> Self::Result {
        // find the beacons
        println!("get diagnostic data called");
        Ok(self.diagnostic_data.clone())
    }
}

// unfortunately, the common library cant import actix, so just make another struct here...
pub struct InternalTagData {
    pub name: String,
    pub mac_address: String,
    pub distance: common::DataType,
}
impl Message for InternalTagData {
    type Result = Result<u64, io::Error>;
}
impl Handler<InternalTagData> for BeaconManager {
    type Result = Result<u64, io::Error>;

    fn handle(&mut self, msg: InternalTagData, context: &mut Context<Self>) -> Self::Result {
        // find the beacons
        println!("tag data message sent to beacon manager");
        self.diagnostic_data.tag_data.push(common::TagData {
            name: msg.name,
            mac_address: msg.mac_address,
            distance: msg.distance,
        });
        Ok(1)
    }
}

impl Handler<StartEmergency> for BeaconManager {
    type Result = Result<u64, io::Error>;

    fn handle(&mut self, msg: StartEmergency, context: &mut Context<Self>) -> Self::Result {
        self.diagnostic_data = common::DiagnosticData::new();
        for connection in &self.serial_connections {
            connection.send(BeaconCommand::StartEmergency);
        }

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
