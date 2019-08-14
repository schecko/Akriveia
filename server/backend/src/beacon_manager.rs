
// this "manager" exists for two main purposes, first the initial implementation will involve
// serial communcation with the beacons, which will be synchronous. Since Actix is a single threaded
// event loop this would not be a good time for our webserver, so instead the work will be shoved
// off to "SyncArbitors" which gives an api like normal actix, but gives each actor(or each beacon
// in this case) its own communcation thread. The second reason is to hopefully abstract this
// functionality a little bit, so that it will be a little bit easier to move to the wireless
// implementation.


use actix::prelude::*;
use actix_web::{ Error, Result };
use crate::beacon_dummy::*;
use crate::beacon_serial::*;
use crate::beacon_udp::*;
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
    pub emergency: bool,
    pub data_processor: Addr<DataProcessor>,
    pub diagnostic_data: common::DiagnosticData,
    pub serial_connections: Vec<mpsc::Sender<BeaconCommand>>,
    pub udp_connections: Vec<Addr<BeaconUDP>>,
}
impl Actor for BeaconManager {
    type Context = Context<Self>;
}

const VENDOR_WHITELIST: &[u16] = &[0x2341, 0x10C4];
const NUM_DUMMY_BEACONS: u32 = 5;

const USE_DUMMY_BEACONS: bool = false;
const USE_SERIAL_BEACONS: bool = false;
const USE_UDP_BEACONS: bool = true;

impl BeaconManager {
    pub fn new(dp: Addr<DataProcessor>) -> BeaconManager {
        BeaconManager {
            emergency: false, // TODO get from db!
            udp_connections: Vec::new(),
            data_processor: dp,
            diagnostic_data: common::DiagnosticData::new(),
            serial_connections: Vec::new(),
        }
    }

    fn find_beacons(&mut self, context: &mut Context<Self>) {
        if USE_DUMMY_BEACONS { self.find_beacons_dummy(context); }
        if USE_SERIAL_BEACONS { self.find_beacons_serial(context); }
        if USE_UDP_BEACONS { self.find_beacons_udp(context); }
    }

    fn find_beacons_udp(&mut self, context: &mut Context<Self>) {
        self.udp_connections.push(BeaconUDP::new("127.0.0.1:0".parse().unwrap()));
    }

    fn find_beacons_dummy(&mut self, context: &mut Context<Self>) {
        for i in 0..NUM_DUMMY_BEACONS {
            let (send, receive): (mpsc::Sender<BeaconCommand>, mpsc::Receiver<BeaconCommand>) = mpsc::channel();
            let beacon_manager = context.address().clone();
            thread::spawn(move || {
                dummy_beacon_thread(i, receive, beacon_manager);
            });
            self.serial_connections.push(send);
        }
    }

    fn find_beacons_serial(&mut self, context: &mut Context<Self>) {
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
    GetEmergency,
    ScanBeacons,
    StartEmergency,
    EndEmergency,
}

impl Message for BeaconCommand {
    type Result = Result<common::SystemCommandResponse, io::Error>;
}
impl Handler<BeaconCommand> for BeaconManager {
    type Result = Result<common::SystemCommandResponse, io::Error>;

    fn handle(&mut self, msg: BeaconCommand, context: &mut Context<Self>) -> Self::Result {
        match msg {
            BeaconCommand::GetEmergency => { },
            BeaconCommand::ScanBeacons => {
                println!("find beacons called!");
                self.find_beacons(context);
            },
            BeaconCommand::StartEmergency => {
                self.diagnostic_data = common::DiagnosticData::new();
                for connection in &self.serial_connections {
                    connection.send(BeaconCommand::StartEmergency);
                }
                self.emergency = true;
            }
            BeaconCommand::EndEmergency => {
                for connection in &self.serial_connections {
                    connection.send(BeaconCommand::EndEmergency);
                }
                self.emergency = false;
            },

        }
        Ok(common::SystemCommandResponse::new(self.emergency))
    }
}

pub struct GetDiagnosticData;
impl Message for GetDiagnosticData {
    type Result = Result<common::DiagnosticData, io::Error>;
}
impl Handler<GetDiagnosticData> for BeaconManager {
    type Result = Result<common::DiagnosticData, io::Error>;

    fn handle(&mut self, msg: GetDiagnosticData, context: &mut Context<Self>) -> Self::Result {
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
        self.diagnostic_data.tag_data.push(msg.data.clone());
        self.data_processor.do_send(DPMessage::LocationData(msg.data.clone()));
        Ok(1)
    }
}

