extern crate actix_web;
extern crate libusb;
extern crate libusb_sys;

use actix_web::{server, App, HttpRequest};
use std::thread;

fn index(_req: &HttpRequest) -> &'static str {
    "Hello world!"
}

fn main() {
    println!("spawning background thread");
    //thread::spawn(move || {
        println!("hello from background thread");
        let mut context = libusb::Context::new().unwrap();

        for mut device in context.devices().unwrap().iter() {
            let device_desc = device.device_descriptor().unwrap();

            println!(
                "Bus: {:03}, Device {:03} ID {:04x}{:04x}",
                device.bus_number(),
                device.address(),
                device_desc.vendor_id(),
                device_desc.product_id()
            );

            println!("bleh bleh");
            let opt_dev_handle = device.open();
            /*match opt_dev_handle {
                Ok(handle) => println!("successfully opened usb device"),
                Err(error) => {
                    println!("error {}", error);
                    std::process::exit(-1);
                }
            }*/
            let dev_handle = opt_dev_handle.unwrap();

            println!("bleh hello");
            let mut buffer: [u8; 128] = [0; 128];
            let timeout = std::time::Duration::from_secs(1);
            let num_read = dev_handle.read_bulk(libusb_sys::LIBUSB_ENDPOINT_IN, &mut buffer, timeout);
            match num_read {
                Ok(num) => println!("successfully read from usb {} bytes", num),
                Err(error) => println!("error {}", error),
            }
        }
    //});

    server::new(|| App::new().resource("/", |r| r.f(index)))
        .bind("0.0.0.0:8080")
        .unwrap()
        .run();
    println!("hello");
}
