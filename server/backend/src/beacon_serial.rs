extern crate serialport;
extern crate regex;

use actix::prelude::*;
use crate::beacon_manager::*;
use crate::conn_common::{ self, MessageError, };
use serialport::*;
use std::io::*;
use std::io::{ self, Write };
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub struct BeaconSerialConn {
    pub port_name: String,
    pub vid: u16,
    pub pid: u16,
    pub receive: mpsc::Receiver<BeaconCommand>,
    pub manager: Recipient<TagDataMessage>,
}

#[allow(dead_code)]
fn send_command(command: String, port: &mut Box<dyn SerialPort>, attempts: u64) -> bool {
    for i in 0.. {
        if let Ok(_) = port.write(command.as_bytes()) {};

        let mut serial_buffer: Vec<u8> = vec![0; 1000];
        match port.read(serial_buffer.as_mut_slice()) {
            Ok(_) => {
                let result = String::from_utf8_lossy(&serial_buffer);
                if result.contains("ack") {
                    break;
                } else {
                    println!("failed to send command {} to beacon", command);
                    if i > attempts {
                        println!("reached maximum retries");
                        return false;
                    }
                }

            },
            Err(e) => {
                if i > attempts {
                    println!("error {}", e);
                    return false;
                }

                println!("serial communication failed on reading ack {}", e);
            }
        }
        thread::sleep(Duration::from_millis(300));
    }

    true
}

pub fn serial_beacon_thread(beacon_info: BeaconSerialConn) {
    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(1000);
    settings.baud_rate = 115200;
    loop {
        println!("opening port");
        match serialport::open_with_settings(&beacon_info.port_name, &settings) {
            Ok(mut opened_port) => {

                // loop infinitely until told to start polling
                loop {
                    match beacon_info.receive.recv() {
                        Ok(BeaconCommand::StartEmergency) => {
                            break;
                        },
                        _ => { },
                    }
                }
                // uncomment after new beacons support commands
                /*match send_command("start".to_string(), &mut opened_port, 4) {
                    true => {},
                    false => {
                        println!("failed to send start command, reopenining port");
                        continue;
                    },
                }*/

                // start polling data
                let mut serial_buffer: Vec<u8> = Vec::new();
                loop {
                    match beacon_info.receive.try_recv() {
                        Ok(BeaconCommand::EndEmergency) => {
                            // break the loop, go to the next stage
                            break;
                        },
                        Err(mpsc::TryRecvError::Empty) => {
                            // do nothing, dont care
                        }
                        Err(mpsc::TryRecvError::Disconnected) => {
                            println!("disconnected from manager");
                        },
                        _ => {
                            // some other type of command, just ignore it for now
                        },
                    }
                    thread::sleep(Duration::from_millis(100));

                    let mut temp_buffer: Vec<u8> = vec![0; 4000];
                    match opened_port.read(temp_buffer.as_mut_slice()) {
                        Ok(num) => {
                            serial_buffer.extend_from_slice(&mut temp_buffer[..num]);
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
                    }

                    let serial_string = String::from_utf8_lossy(&serial_buffer[..]).to_string();

                    let mut last_line = "";
                    for line in serial_string.split("\n") {

                        match conn_common::parse_message(line) {
                            Ok(msg) => {
                                beacon_info.manager
                                    .do_send(TagDataMessage { data: msg })
                                    .expect("serial beacon could not send message to manager");
                            },
                            Err(e) => {
                                match e {
                                    // ignore bad data
                                    MessageError::ParseFloat | MessageError::ParseMac => continue,
                                    // preserve partial serial data
                                    MessageError::ParseFormat => {
                                        last_line = line;
                                    },
                                }
                            }
                        }
                    }
                    serial_buffer = Vec::new();
                    serial_buffer.extend_from_slice(last_line.as_bytes());
                }

                // uncomment after new beacons support commands
                /*match send_command("end".to_string(), &mut opened_port, 4) {
                    true => {},
                    false => {
                        println!("failed to send end command to beacon");
                        continue;
                    },
                }*/
            }
            Err(e) => {
                eprintln!("Failed to open arduino port \"{}\". Error: {}", beacon_info.port_name, e);
            }
        }
    }
}
