extern crate serialport;

use actix::prelude::*;
use crate::beacon_manager::*;
use serialport::*;
use serialport::prelude::*;
use std::io::{ self, Write };
use std::sync::mpsc;
use std::sync::{ Arc, Mutex, };
use std::thread;
use std::time::Duration;
use std::io::*;


#[allow(dead_code)] // remove this once vid/pid are actually used.
pub struct BeaconSerialConn {
    pub port_name: String,
    pub vid: u16,
    pub pid: u16,
    pub receive: mpsc::Receiver<BeaconCommand>,
    pub manager: Recipient<InternalTagData>,
}

pub enum BeaconCommand {
    StartEmergency,
    EndEmergency
}

fn serial_comms(beacon_info: BeaconSerialConn, mut opened_port: Box<SerialPort>) {

}

pub fn serial_beacon_thread(beacon_info: BeaconSerialConn) {
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(1000);
    settings.baud_rate = 9600;
    println!("opening port");
    let mut b = false;
    loop {
        match serialport::open_with_settings(&beacon_info.port_name, &settings) {
            Ok(mut opened_port) => {
                println!("initiating communication");
                b = false;
                for i in 0.. {
                    if i % 2 == 0 {
                        //if let Ok(_) = opened_port.clear(ClearBuffer::All) {}
                    }
                    if i > 4 {
                        b = true;
                        break;
                    }

                    println!("initiating, attempt {}", i);

                    if let Ok(_) = opened_port.write(b"start") {};

                    let mut serial_buffer: Vec<u8> = vec![0; 1000];

                    match opened_port.read(serial_buffer.as_mut_slice()) {
                        Ok(_) => {
                            let result = String::from_utf8_lossy(&serial_buffer);
                            println!("buffer is: {}", result);
                            if result.contains("ack") {
                                println!("successfully received ack from beacon");
                                break;
                            } else {
                                println!("failed to start beacon");
                            }

                        },
                        Err(e) => {
                            println!("serial communication failed on reading ack {}", e);
                        }
                    }

                    thread::sleep(Duration::from_millis(300));
                }
                if(b) {
                    break;
                }

                // loop infinitely until told to start polling
                loop {
                    match beacon_info.receive.recv() {
                        Ok(BeaconCommand::StartEmergency) => {
                            break;
                        },
                        _ => {
                            println!("ignoring command");
                        },
                    }
                }

                // start polling data
                println!("Receiving data on {} :", &beacon_info.port_name);
                let mut serial_buffer: Vec<u8> = vec![0; 1000];
                loop {
                    match beacon_info.receive.try_recv() {
                        Ok(BeaconCommand::EndEmergency) => {
                            break;
                        },
                        _ => {
                            println!("ignoring command");
                        },
                    }
                    println!("reading...");
                    thread::sleep(Duration::from_millis(100));
                    for _ in 1..3 {
                        match opened_port.read(serial_buffer.as_mut_slice()) {
                            Ok(t) => io::stdout().write_all(&serial_buffer[..t]).unwrap(),
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }

                    thread::sleep(Duration::from_millis(1000));

                    beacon_info.manager
                        .do_send( InternalTagData {
                            name: "hello".to_string(),
                            mac_address: "bleh bleh".to_string(),
                            distance: common::DataType::RSSI(55),
                        });

                }
            }
            Err(e) => {
                eprintln!("Failed to open arduino port \"{}\". Error: {}", beacon_info.port_name, e);
            }
        }
    }
}
