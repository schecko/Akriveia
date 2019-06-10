extern crate actix_web;
extern crate serialport;

//use actix_web::{server, App, HttpRequest};
use serialport::prelude::*;
use std::io::{self, Write};
use std::slice;
use std::time::Duration;

/*fn index(_req: &HttpRequest) -> &'static str {
    "Hello world!"
}*/

fn main() {
    let port_name = "/dev/ttyACM0";
    let baud_rate = 9600;

    let mut settings: SerialPortSettings = Default::default();
    settings.timeout = Duration::from_millis(10);
    settings.baud_rate = baud_rate;

    match serialport::open_with_settings(&port_name, &settings) {
        Ok(mut port) => {
            let mut serial_buf: Vec<u8> = vec![0; 1000];
            println!("Receiving data on {} at {} baud:", &port_name, &baud_rate);
            loop {
                match port.read(serial_buf.as_mut_slice()) {
                    Ok(t) => io::stdout().write_all(&serial_buf[..t]).unwrap(),
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => eprintln!("{:?}", e),
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to open \"{}\". Error: {}", port_name, e);
        }
    }

    /*server::new(|| App::new().resource("/", |r| r.f(index)))
        .bind("0.0.0.0:8080")
        .unwrap()
        .run();*/
    println!("end");
}


