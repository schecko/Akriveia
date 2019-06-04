extern crate actix_web;
extern crate libusb;
extern crate libusb_sys;

//use actix_web::{server, App, HttpRequest};
use std::slice;
use std::time::Duration;

#[derive(Debug)]
struct Endpoint {
    config: u8,
    iface: u8,
    setting: u8,
    address: u8,
    transfer_type: libusb::TransferType
}

struct UsbDevice<'a> {
    handle: libusb::DeviceHandle<'a>,
    language: libusb::Language,
    timeout: Duration
}

/*fn index(_req: &HttpRequest) -> &'static str {
    "Hello world!"
}*/

fn main() {
    let context = libusb::Context::new().unwrap();

    for mut device in context.devices().unwrap().iter() {
        let device_desc = match device.device_descriptor() {
            Ok(desc) => desc,
            Err(_) => continue,
        };

        print_device_desc(&device_desc);

        println!("Bus {:03} Device {:03} ID {:04x}:{:04x} {}", device.bus_number(), device.address(), device_desc.vendor_id(), device_desc.product_id(), get_speed(device.speed()));

        let timeout = Duration::from_secs(1);
        let mut dev_handle = match device.open() {
            Ok(handle) => {
                println!("successfully opened device");
                handle
            },
            Err(_) => {
                println!("failed to open device handle");
                continue;
            }
        };

        let languages = match dev_handle.read_languages(timeout) {
            Ok(langs) => {
                println!("successfully obtained language from device");
                langs
            },
            Err(_) => {
                println!("failed to obtain language for device");
                continue;
            }
        };

        //try!(dev_handle.reset());
        println!("Languages: {:?}", languages);

        if languages.len() > 0 {
            let language = languages[0];

            println!("Manufacturer: {:?}", dev_handle.read_manufacturer_string(language, &device_desc, timeout).ok());
            println!("Product: {:?}", dev_handle.read_product_string(language, &device_desc, timeout).ok());
            println!("Serial Number: {:?}", dev_handle.read_serial_number_string(language, &device_desc, timeout).ok());
        }

        let endpoint = match find_readable_endpoint(&mut device, &device_desc) {
            Some(endpoint) => endpoint,
            None => {
                println!("no readable interrupt endpoint");
                continue;
            }
        };


        read_endpoint(&mut dev_handle, endpoint);

    }

    /*server::new(|| App::new().resource("/", |r| r.f(index)))
        .bind("0.0.0.0:8080")
        .unwrap()
        .run();*/
    println!("end");
}

fn find_readable_endpoint(device: &mut libusb::Device, device_desc: &libusb::DeviceDescriptor) -> Option<Endpoint> {
    for n in 0..device_desc.num_configurations() {
        let config_desc = match device.config_descriptor(n) {
            Ok(c) => c,
            Err(_) => continue
        };

        for interface in config_desc.interfaces() {
            for interface_desc in interface.descriptors() {
                for endpoint_desc in interface_desc.endpoint_descriptors() {
                    for transfer_type in [libusb::TransferType::Interrupt, libusb::TransferType::Bulk].iter() {
                        if endpoint_desc.direction() == libusb::Direction::In && endpoint_desc.transfer_type() == *transfer_type {
                            return Some(Endpoint {
                                config: config_desc.number(),
                                iface: interface_desc.interface_number(),
                                setting: interface_desc.setting_number(),
                                address: endpoint_desc.address(),
                                transfer_type: *transfer_type
                            });
                        }
                    }
                }
            }
        }
    }

    None
}

fn read_endpoint(handle: &mut libusb::DeviceHandle, endpoint: Endpoint) {
    println!("Reading from endpoint: {:?}", endpoint);

    let has_kernel_driver = match handle.kernel_driver_active(endpoint.iface) {
        Ok(true) => {
            handle.detach_kernel_driver(endpoint.iface).ok();
            true
        },
        _ => false
    };

    println!(" - kernel driver? {}", has_kernel_driver);

    match configure_endpoint(handle, &endpoint) {
        Ok(_) => {
            let mut vec = Vec::<u8>::with_capacity(256);
            let mut buf = unsafe { slice::from_raw_parts_mut((&mut vec[..]).as_mut_ptr(), vec.capacity()) };

            let timeout = Duration::from_secs(1);

            match endpoint.transfer_type {
                libusb::TransferType::Interrupt => {
                    match handle.read_interrupt(endpoint.address, buf, timeout) {
                        Ok(len) => {
                            unsafe { vec.set_len(len) };
                            println!(" - read: {:?}", vec);
                        },
                        Err(err) => println!("could not read from endpoint: {}", err)
                    }
                },
                libusb::TransferType::Bulk => {
                    match handle.read_bulk(endpoint.address, buf, timeout) {
                        Ok(len) => {
                            unsafe { vec.set_len(len) };
                            println!(" - read: {:?}", vec);
                        },
                        Err(err) => println!("could not read from endpoint: {}", err)
                    }
                },
                _ => ()
            }
        },
        Err(err) => println!("could not configure endpoint: {}", err)
    }

    if has_kernel_driver {
        handle.attach_kernel_driver(endpoint.iface).ok();
    }
}

fn configure_endpoint<'a>(handle: &'a mut libusb::DeviceHandle, endpoint: &Endpoint) -> libusb::Result<()> {
    try!(handle.set_active_configuration(endpoint.config));
    try!(handle.claim_interface(endpoint.iface));
    try!(handle.set_alternate_setting(endpoint.iface, endpoint.setting));
    Ok(())
}
fn print_device_desc(device_desc: &libusb::DeviceDescriptor) {
    println!("Device Descriptor:");
    println!("  bcdUSB             {:2}.{}{}", device_desc.usb_version().major(), device_desc.usb_version().minor(), device_desc.usb_version().sub_minor());
    println!("  bDeviceClass        {:#04x}", device_desc.class_code());
    println!("  bDeviceSubClass     {:#04x}", device_desc.sub_class_code());
    println!("  bDeviceProtocol     {:#04x}", device_desc.protocol_code());
    println!("  bMaxPacketSize0      {:3}", device_desc.max_packet_size());
    println!("  idVendor          {:#06x}", device_desc.vendor_id());
    println!("  idProduct         {:#06x}", device_desc.product_id());
    println!("  bcdDevice          {:2}.{}{}", device_desc.device_version().major(), device_desc.device_version().minor(), device_desc.device_version().sub_minor());
    println!("  bNumConfigurations   {:3}", device_desc.num_configurations());
}

fn print_config(config_desc: &libusb::ConfigDescriptor, handle: &mut Option<UsbDevice>) {
    println!("  Config Descriptor:");
    println!("    bNumInterfaces       {:3}", config_desc.num_interfaces());
    println!("    bConfigurationValue  {:3}", config_desc.number());
    println!("    iConfiguration       {:3} {}",
             config_desc.description_string_index().unwrap_or(0),
             handle.as_mut().map_or(String::new(), |h| h.handle.read_configuration_string(h.language, config_desc, h.timeout).unwrap_or(String::new())));
    println!("    bmAttributes:");
    println!("      Self Powered     {:>5}", config_desc.self_powered());
    println!("      Remote Wakeup    {:>5}", config_desc.remote_wakeup());
    println!("    bMaxPower           {:4}mW", config_desc.max_power());
}

fn print_interface(interface_desc: &libusb::InterfaceDescriptor, handle: &mut Option<UsbDevice>) {
    println!("    Interface Descriptor:");
    println!("      bInterfaceNumber     {:3}", interface_desc.interface_number());
    println!("      bAlternateSetting    {:3}", interface_desc.setting_number());
    println!("      bNumEndpoints        {:3}", interface_desc.num_endpoints());
    println!("      bInterfaceClass     {:#04x}", interface_desc.class_code());
    println!("      bInterfaceSubClass  {:#04x}", interface_desc.sub_class_code());
    println!("      bInterfaceProtocol  {:#04x}", interface_desc.protocol_code());
    println!("      iInterface           {:3} {}",
             interface_desc.description_string_index().unwrap_or(0),
             handle.as_mut().map_or(String::new(), |h| h.handle.read_interface_string(h.language, interface_desc, h.timeout).unwrap_or(String::new())));
}

fn print_endpoint(endpoint_desc: &libusb::EndpointDescriptor) {
    println!("      Endpoint Descriptor:");
    println!("        bEndpointAddress    {:#04x} EP {} {:?}", endpoint_desc.address(), endpoint_desc.number(), endpoint_desc.direction());
    println!("        bmAttributes:");
    println!("          Transfer Type          {:?}", endpoint_desc.transfer_type());
    println!("          Synch Type             {:?}", endpoint_desc.sync_type());
    println!("          Usage Type             {:?}", endpoint_desc.usage_type());
    println!("        wMaxPacketSize    {:#06x}", endpoint_desc.max_packet_size());
    println!("        bInterval            {:3}", endpoint_desc.interval());
}

fn get_speed(speed: libusb::Speed) -> &'static str {
    match speed {
        libusb::Speed::Super   => "5000 Mbps",
        libusb::Speed::High    => " 480 Mbps",
        libusb::Speed::Full    => "  12 Mbps",
        libusb::Speed::Low     => " 1.5 Mbps",
        libusb::Speed::Unknown => "(unknown)"
    }
}
