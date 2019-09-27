//#![deny(warnings)]
extern crate actix;
extern crate actix_files;
extern crate actix_session;
extern crate actix_web;
extern crate common;
extern crate env_logger;
extern crate futures;
extern crate ipnet;
extern crate nalgebra as na;
extern crate tokio_postgres;

mod beacon_dummy;
mod beacon_manager;
mod beacon_serial;
mod beacon_udp;
mod controllers;
mod data_processor;
mod db_utils;
mod models;

use controllers::beacon_controller;
use controllers::map_controller;
use controllers::system_controller;
use controllers::user_controller;
use controllers::network_interface_controller;

//use models::beacon;
//use models::map;
use models::system;
//use models::user;

use actix::prelude::*;
use actix_files as fs;
use actix_web::{ error, middleware, web, App, HttpRequest, HttpResponse, HttpServer, };
use beacon_manager::*;
use data_processor::*;
use std::env;
use std::sync::*;
use common::*;

#[derive(Clone)]
pub struct AkriveiaState {
    pub beacon_manager: Addr<BeaconManager>,
    pub data_processor: Addr<DataProcessor>,
}

impl AkriveiaState {
    pub fn new() -> web::Data<Mutex<AkriveiaState>> {

        let data_processor_addr =  DataProcessor::new().start();
        let beacon_manager_addr = BeaconManager::new(data_processor_addr.clone()).start();

        beacon_manager_addr.do_send(BMCommand::ScanBeacons);

        web::Data::new(Mutex::new(AkriveiaState {
            beacon_manager: beacon_manager_addr,
            data_processor: data_processor_addr,
        }))
    }
}

fn default_route(req: HttpRequest) -> HttpResponse {
    println!("default route called");
    println!("request was: {:?}", req);
    HttpResponse::NotFound().finish()
}

use std::net::UdpSocket;
use bytes::{ Bytes };
use std::time;

fn main() -> std::io::Result<()> {
    /*{
        let mut i = 0;
        let sock = UdpSocket::bind("127.0.0.1:8081").expect("couldn't bind to address");
        sock.set_broadcast(true).expect("fuck fuck fuc");
        sock.connect("127.0.0.255:8082").expect("connect function failed");
        sock.set_read_timeout(Some(time::Duration::from_millis(10)));
        loop {
            match sock.send(&Bytes::from(format!("fuck fuck {}\n", i))) {
                Ok(_) => println!("yayyyyyy"),
                Err(e) => println!("what the fuck {}", e),
            }
            i += 1;
            std::thread::sleep(time::Duration::from_millis(1000));
            /*let mut buf = [0; 1000];
            match sock.recv(&mut buf) {
                Ok(received) => println!("received {} bytes {:?}", received, &buf[..received]),
                Err(e) => println!("recv function failed: {:?}", e),
            }*/
        }

    }*/
    let system = System::new("Akriviea");
    env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();
    let create_db_fut = system::create_db();
    // intentionally block all further execution
    tokio::run(create_db_fut);

    let state = AkriveiaState::new();

    let _insert = db_utils::default_connect()
        .and_then(|client| {
            println!("inserting");
            models::beacon::insert_beacon(client, common::Beacon::new())
        })
        .map(|result_beacon| {
            println!("result is : {:?}", result_beacon.1);
        })
        .map_err(|e| {
            println!("db error {:?}", e);
        });
    //tokio::run(insert);

    // start the webserver
    HttpServer::new(move || {
        App::new()
            .data(
                web::JsonConfig::default()
                    .limit(4096)
                    .error_handler(|err, req| {
                        println!("Failed to parse request body {:?},\n {:?}", err, req);
                        error::InternalError::from_response(
                            err,
                            HttpResponse::BadRequest().finish()
                        ).into()
                    })
            )
            .register_data(state.clone())
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())

            // beacon
            .service(
                web::resource(&beacons_url())
                    .route(web::get().to_async(beacon_controller::get_beacons))
            )

            .service(
                web::resource(&beacon_url("{id}"))
                    .route(web::get().to_async(beacon_controller::get_beacon))
                    .route(web::put().to_async(beacon_controller::put_beacon))
                    .route(web::delete().to_async(beacon_controller::delete_beacon))
            )
            .service(
                web::resource(&beacon_url(""))
                    .route(web::post().to_async(beacon_controller::post_beacon))
            )

            // user
            .service(
                web::resource(&users_url())
                    .route(web::get().to_async(user_controller::get_users))
            )
            .service(
                web::resource(&user_url("{id}"))
                    .route(web::get().to_async(user_controller::get_user))
                    .route(web::put().to_async(user_controller::put_user))
                    .route(web::delete().to_async(user_controller::delete_user))
            )
            .service(
                web::resource(&user_url(""))
                    .route(web::post().to_async(user_controller::post_user))
            )

            // map
            .service(
                web::resource(&maps_url())
                    .route(web::get().to_async(map_controller::get_maps))
            )
            .service(
                web::resource(&beacons_for_map_url("{id}"))
                    .route(web::get().to_async(beacon_controller::get_beacons_for_map))
            )
            .service(
                web::resource(&map_url("{id}"))
                    .route(web::get().to_async(map_controller::get_map))
                    .route(web::put().to_async(map_controller::put_map))
                    .route(web::delete().to_async(map_controller::delete_map))
            )
            .service(
                web::resource(&map_url(""))
                    .route(web::post().to_async(map_controller::post_map))
            )

            // system
            .service(
                web::resource(&system_emergency_url())
                    .route(web::get().to_async(system_controller::get_emergency))
                    .route(web::post().to_async(system_controller::post_emergency))
            )
            .service(
                web::resource(&system_diagnostics_url())
                    .route(web::get().to_async(system_controller::diagnostics))
            )

            // network
            .service(
                web::resource(&networks_url())
                    .route(web::get().to_async(network_interface_controller::get_network_interfaces))
            )
            .service(
                web::resource(&network_url("{id}"))
                    .route(web::get().to_async(network_interface_controller::get_network_interface))
                    .route(web::put().to_async(network_interface_controller::put_network_interface))
                    .route(web::delete().to_async(network_interface_controller::delete_network_interface))
            )
            .service(
                web::resource(&network_url(""))
                    .route(web::post().to_async(network_interface_controller::post_network_interface))
            )

            .service(web::resource(&users_realtime_url()).to_async(user_controller::realtime_users))
            // these two last !!
            .service(fs::Files::new("/", "static").index_file("index.html"))
            .default_service(web::resource("").to(default_route))
    })
    .bind("0.0.0.0:8080")?
    .start();

    system.run()
}

