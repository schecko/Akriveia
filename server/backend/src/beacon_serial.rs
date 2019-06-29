extern crate serialport;
extern crate regex;

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
use std::str;
use regex::Regex;


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

fn send_command(command: String, port: &mut Box<SerialPort>, attempts: u64) -> bool {
    for i in 0.. {
        if i % 2 == 0 {
            //if let Ok(_) = opened_port.clear(ClearBuffer::All) {}
        }

        println!(", attempt {}", i);

        if let Ok(_) = port.write(command.as_bytes()) {};

        let mut serial_buffer: Vec<u8> = vec![0; 1000];
        match port.read(serial_buffer.as_mut_slice()) {
            Ok(_) => {
                let result = String::from_utf8_lossy(&serial_buffer);
                println!("buffer is: {}", result);
                if result.contains("ack") {
                    println!("successfully received ack from beacon for command {}", command);
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
    settings.baud_rate = 9600;
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
                        _ => {
                            println!("ignoring command");
                        },
                    }
                }
                match send_command("start".to_string(), &mut opened_port, 4) {
                    true => {},
                    false => {
                        println!("failed to send start command, reopenining port");
                        continue;
                    },
                }

                // start polling data
                println!("Receiving data on {} :", &beacon_info.port_name);
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
                            println!("ignoring command");
                        },
                    }
                    println!("reading...");
                    thread::sleep(Duration::from_millis(100));

                    let mut char_count = 0;
                    let mut temp_buffer: Vec<u8> = vec![0; 4000];
                    match opened_port.read(temp_buffer.as_mut_slice()) {
                        Ok(num) => {
                            if num > 0 {
                                println!("temp string is: {}", String::from_utf8_lossy(&temp_buffer[..num]));
                            } else {
                                println!("temp empty");
                            }
                            serial_buffer.extend_from_slice(&mut temp_buffer[..num]);
                            char_count += num;
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => eprintln!("{:?}", e),
                    }

                    let data = String::from_utf8_lossy(&serial_buffer[..char_count]).to_string();
                    let serial_string = String::from_utf8_lossy(&serial_buffer[..]).to_string();
                    //println!("serial buffer is: {}", String::from_utf8_lossy(&serial_buffer[..]).to_string());


                    let mut last_line = "";
                    for line in serial_string.split("\n") {
                        //println!("line is: {}", line);

                        let mut split: Vec<&str> = line.split("|").collect();
                        if split.len() == 3 {
                            let name = split[0];
                            let mac = split[1];
                            let mut rssi = split[2];
                            //println!(" name, mac, rssi: {} {} {} ", name, mac, rssi);
                            let reg = Regex::new(r"/[^$0-9]+/").unwrap();
                            let rssi_stripped = reg.replace_all(&rssi, "");

                            // remove the last character every time, idk why but there is always
                            // a newline at the end of rssi_stripped. from_str_radix REQUIRES
                            // all numeric characters.
                            match i64::from_str_radix(&rssi_stripped[..rssi_stripped.len() - 1], 10) {
                                Ok(rssi_numeric) => {
                                    beacon_info.manager
                                        .do_send( InternalTagData {
                                            name: name.to_string(),
                                            mac_address: mac.to_string(),
                                            distance: common::DataType::RSSI(rssi_numeric),
                                        });
                                },
                                Err(e) => {
                                    println!("parsed a bad number: {}, error {}", e, rssi_stripped);
                                }
                            }
                        } else {
                            // incomplete transmission, keep the data and append to the
                            // serial_buffer after its reset
                            last_line = line;
                        }
                    }
                    serial_buffer = Vec::new();
                    serial_buffer.extend_from_slice(last_line.as_bytes());
                    char_count = 0;
                }

                match send_command("end".to_string(), &mut opened_port, 4) {
                    true => {},
                    false => {
                        println!("failed to send end command to beacon");
                        continue;
                    },
                }
            }
            Err(e) => {
                eprintln!("Failed to open arduino port \"{}\". Error: {}", beacon_info.port_name, e);
            }
        }
    }
}