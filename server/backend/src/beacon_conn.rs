extern crate serialport;

use serialport::prelude::*;
use serialport::SerialPortType;
use std::io::{self, Write};
use std::slice;
use std::time::Duration;
use std::thread;


const BAUD_RATE: u32 = 9600;

struct AkSerialPort {
    port_name: String,
    vid: u16,
    pid: u16,
}

pub fn init() {
    thread::spawn(move || {
        connect_read();
    });
}

fn connect_read() {

    let mut ports: Vec<AkSerialPort> = Vec::new();
    if let Ok(avail_ports) = serialport::available_ports() {
        for port in avail_ports {
            println!("\t{}", port.port_name);
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

                        ports.push(AkSerialPort {
                            port_name: port.port_name,
                            vid: info.vid,
                            pid: info.pid
                        });
                    }
                }
                _ => {}
            }
        }
    } else {
        print!("Error listing serial ports");
    }

    // make ports immutable now.
    let ports = ports;

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = BAUD_RATE;

    for port in ports {
        match serialport::open_with_settings(&port.port_name, &settings) {
            Ok(mut opened_port) => {
                let mut serial_buf: Vec<u8> = vec![0; 1000];
                println!("Receiving data on {} at {} baud:", &port.port_name, &settings.baud_rate);
                loop {
                    println!("clearing port");
                    opened_port.clear(ClearBuffer::All);
                    thread::sleep_ms(100);
                    println!("writing");
                    opened_port.write(b"start");
                    for n in 1..300 {
                        match opened_port.read(serial_buf.as_mut_slice()) {
                            Ok(t) => io::stdout().write_all(&serial_buf[..t]).unwrap(),
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    opened_port.write(b"end");

                    thread::sleep_ms(1000);
                }
            }
            Err(e) => {
                eprintln!("Failed to open \"{}\". Error: {}", port.port_name, e);
            }
        }
    }
}

