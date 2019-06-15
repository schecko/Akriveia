extern crate serialport;

use std::sync::{Arc, Mutex};
use serialport::prelude::*;
use actix::prelude::*;
use serialport::*;
use std::io::{self, Write};
use std::time::Duration;
use std::thread;


#[allow(dead_code)] // remove this once vid/pid are actually used.

pub struct BeaconSerialConn {
    pub port_name: String,
    pub vid: u16,
    pub pid: u16,
}

impl Actor for BeaconSerialConn {
    type Context = SyncContext<Self>;
}

pub struct StartDataCollection;
impl Message for StartDataCollection {
    type Result = Result<u64>;
}

pub struct GetBeaconData;
impl Message for GetBeaconData {
    type Result = Result<u64>;
}

impl Handler<StartDataCollection> for BeaconSerialConn {
    type Result = Result<u64>;

    fn handle(&mut self, msg: StartDataCollection, context: &mut SyncContext<Self>) -> Self::Result {

        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(10);
        settings.baud_rate = 9600;
        match serialport::open_with_settings(&self.port_name, &settings) {
            Ok(mut opened_port) => {
                if let Ok(_) = opened_port.clear(ClearBuffer::All) {}

                thread::sleep(Duration::from_millis(100));
                println!("writing");
                if let Ok(_) = opened_port.write(b"start") {};
            }
            Err(e) => {
                eprintln!("Failed to open arduino port \"{}\". Error: {}", self.port_name, e);
            }
        }

        Ok(1)

    }
}

impl Handler<GetBeaconData> for BeaconSerialConn {
    type Result = Result<u64>;

    fn handle(&mut self, msg: GetBeaconData, context: &mut SyncContext<Self>) -> Self::Result {

        let mut settings: SerialPortSettings = Default::default();
        settings.timeout = Duration::from_millis(10);
        settings.baud_rate = 9600;
        match serialport::open_with_settings(&self.port_name, &settings) {
            Ok(mut opened_port) => {
                let mut serial_buf: Vec<u8> = vec![0; 1000];
                println!("Receiving data on {} :", &self.port_name);
                loop {
                    println!("clearing port");
                    if let Ok(_) = opened_port.clear(ClearBuffer::All) {}

                    thread::sleep(Duration::from_millis(100));
                    println!("writing");
                    if let Ok(_) = opened_port.write(b"start") {};
                    for _ in 1..300 {
                        match opened_port.read(serial_buf.as_mut_slice()) {
                            Ok(t) => io::stdout().write_all(&serial_buf[..t]).unwrap(),
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    if let Ok(_) = opened_port.write(b"end") {};

                    thread::sleep(Duration::from_millis(1000));
                }
            }
            Err(e) => {
                eprintln!("Failed to open arduino port \"{}\". Error: {}", self.port_name, e);
            }
        }

        Ok(1)
    }
}
