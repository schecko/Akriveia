#![deny(warnings)]
extern crate actix;
extern crate actix_files;
extern crate actix_session;
extern crate actix_web;
extern crate common;
extern crate env_logger;
extern crate futures;
extern crate nalgebra as na;
extern crate tokio_postgres;

mod beacon_dummy;
mod beacon_manager;
mod beacon_serial;
mod beacon_udp;
mod controllers;
mod data_processor;
mod models;

use controllers::beacon_controller;
use controllers::map_controller;
use controllers::system_controller;
use controllers::user_controller;

//use models::beacon;
//use models::map;
use models::system;
//use models::user;

use actix::prelude::*;
use actix_files as fs;
use actix_web::{ middleware, web, App, HttpRequest, HttpResponse, HttpServer, };
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

        beacon_manager_addr.do_send(BeaconCommand::ScanBeacons);

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

fn main() -> std::io::Result<()> {
    let system = System::new("Akriviea");
    env::set_var("RUST_LOG", "actix_server=info,actix_web=info");
    env_logger::init();
    system::create_db();
    let state = AkriveiaState::new();

    // start the webserver
    HttpServer::new(move || {
        App::new()
            .data(web::JsonConfig::default().limit(4096))
            .register_data(state.clone())
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(
                web::resource(&beacons_url())
                    .route(web::get().to_async(beacon_controller::get_beacons))
            )
            .service(
                web::resource(&beacon_url("{id}"))
                    .route(web::get().to_async(beacon_controller::get_beacon))
                    .route(web::put().to_async(beacon_controller::put_beacon))
                    .route(web::post().to_async(beacon_controller::post_beacon))
                    .route(web::delete().to_async(beacon_controller::delete_beacon))
            )
            .service(
                web::resource(&users_url())
                    .route(web::get().to_async(user_controller::get_users))
            )
            .service(
                web::resource(&user_url("{id}"))
                    .route(web::get().to_async(user_controller::get_user))
                    .route(web::put().to_async(user_controller::put_user))
                    .route(web::post().to_async(user_controller::post_user))
                    .route(web::delete().to_async(user_controller::delete_user))
            )
            .service(
                web::resource(&maps_url())
                    .route(web::get().to_async(map_controller::get_maps))
            )
            .service(
                web::resource(&map_url("{id}"))
                    .route(web::get().to_async(map_controller::get_map))
                    .route(web::put().to_async(map_controller::put_map))
                    .route(web::post().to_async(map_controller::post_map))
                    .route(web::delete().to_async(map_controller::delete_map))
            )
            .service(
                web::resource(&system_emergency_url())
                    .route(web::get().to_async(system_controller::get_emergency))
                    .route(web::post().to_async(system_controller::post_emergency))
            )
            .service(
                web::resource(&system_diagnostics_url())
                    .route(web::get().to_async(system_controller::diagnostics))
            )
            .service(web::resource(common::REALTIME_USERS).to_async(user_controller::realtime_users))
            // these two last !!
            .service(fs::Files::new("/", "static").index_file("index.html"))
            .default_service(web::resource("").to(default_route))
    })
    .bind("0.0.0.0:8080")?
    .start();

    system.run()
}

