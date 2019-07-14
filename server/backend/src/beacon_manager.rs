
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
use crate::data_processor::*;
use futures::{ future::ok, Future };
use serialport::prelude::*;
use serialport;
use std::io;
use std::sync::mpsc;
use std::sync::{ Arc, Mutex, };
use std::thread;
use std::time::Duration;

pub struct BeaconManager {
    pub data_processor: Addr<DataProcessor>,
    pub diagnostic_data: common::DiagnosticData,
    pub serial_connections: Vec<mpsc::Sender<BeaconCommand>>,
}
impl Actor for BeaconManager {
    type Context = Context<Self>;
}

const VENDOR_WHITELIST: &[u16] = &[0x2341, 0x10C4];

impl BeaconManager {
    pub fn new() -> BeaconManager {
        let mut res = BeaconManager {
            data_processor: DataProcessor::new().start(),
            diagnostic_data: common::DiagnosticData::new(),
            serial_connections: Vec::new(),
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

pub enum BeaconCommand {
    ScanBeacons,
    StartEmergency,
    EndEmergency,
}
impl Message for BeaconCommand {
    type Result = Result<u64, io::Error>;
}
impl Handler<BeaconCommand> for BeaconManager {
    type Result = Result<u64, io::Error>;

    fn handle(&mut self, msg: BeaconCommand, context: &mut Context<Self>) -> Self::Result {
        match msg {
            BeaconCommand::ScanBeacons => {
                println!("find beacons called!");
                self.find_beacons(context);
            },
            BeaconCommand::StartEmergency => {
                self.diagnostic_data = common::DiagnosticData::new();
                for connection in &self.serial_connections {
                    connection.send(BeaconCommand::StartEmergency);
                }
            }
            BeaconCommand::EndEmergency => {
                for connection in &self.serial_connections {
                    connection.send(BeaconCommand::EndEmergency);
                }
            },

        }
        Ok(1)
    }
}

pub struct GetDiagnosticData;
impl Message for GetDiagnosticData {
    type Result = Result<common::DiagnosticData, io::Error>;
}
impl Handler<GetDiagnosticData> for BeaconManager {
    type Result = Result<common::DiagnosticData, io::Error>;

    fn handle(&mut self, msg: GetDiagnosticData, context: &mut Context<Self>) -> Self::Result {
        // find the beacons
        println!("get diagnostic data called {:?}", &self.diagnostic_data);
        let res = self.diagnostic_data.clone();
        self.diagnostic_data.tag_data = Vec::new();
        Ok(res)
    }
}

pub struct TagDataMessage {
    pub data: common::TagData,
}
impl Message for TagDataMessage {
    type Result = Result<u64, io::Error>;
}
impl Handler<TagDataMessage> for BeaconManager {
    type Result = Result<u64, io::Error>;

    fn handle(&mut self, msg: TagDataMessage, context: &mut Context<Self>) -> Self::Result {
        // find the beacons
        println!("tag data message sent to beacon manager");
        self.diagnostic_data.tag_data.push(msg.data.clone());
        self.data_processor.do_send(DPMessage::LocationData(msg.data.clone()));
        Ok(1)
    }
}

