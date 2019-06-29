extern crate actix;
extern crate actix_files;
extern crate actix_session;
extern crate actix_web;
extern crate common;
extern crate env_logger;
extern crate futures;

mod beacon_manager;
mod beacon_serial;

use actix::prelude::*;
use actix_files as fs;
use actix_web::{ get, middleware, Error, web, App, HttpRequest, HttpResponse, HttpServer, };
use beacon_manager::*;
use futures::{ future::ok, Future, };
use serde_derive::{ Deserialize, Serialize, };
use std::env;
use std::sync::*;
use std::thread::*;

#[derive(Clone)]
struct AkriveiaState {
    pub beacon_manager: Addr<BeaconManager>,
}

fn hello(req: HttpRequest) -> HttpResponse {
    println!("hello called");
    let hello_data = common::HelloFrontEnd {
        data: 0xDEADBEEF,
    };
    HttpResponse::Ok().json(hello_data)
}

#[get("/scan_beacons")]
fn scan_beacons(req: HttpRequest) -> HttpResponse {
    println!("scanning for beacons");

    HttpResponse::Ok().finish()
}

fn emergency(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> HttpResponse {
    println!("emergency initiated!");
    let s = state.lock().unwrap();
    s.beacon_manager.do_send(StartEmergency);
    HttpResponse::Ok().finish()
}

fn end_emergency(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> HttpResponse {
    println!("emergency stopped!");
    let s = state.lock().unwrap();
    s.beacon_manager.do_send(EndEmergency);
    HttpResponse::Ok().finish()
}

fn diagnostics(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> HttpResponse {
    let diag_data = common::DiagnosticData {
        tag_data: vec![
            common::TagData {
                name: "hello".to_string(),
                mac_address: "mac_0111".to_string(),
                distance: common::DataType::RSSI(33),
            }
        ]
    };
    HttpResponse::Ok().json(diag_data)
}

fn async_diagnostics(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    println!("async diagnostics called");
    let s = state.lock().unwrap();
    s.beacon_manager
        .send(GetDiagnosticData)
        .then(|res| {
            match res {
                Ok(Ok(data)) => {
                    println!("data is {:?}", data);
                    ok(HttpResponse::Ok().json(data))
                },
                _ => {
                    ok(HttpResponse::BadRequest().finish())
                }
        }})
}

fn index_async(req: HttpRequest) -> impl Future<Item = HttpResponse, Error = Error> {
    println!("{:?}", req);

    ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(format!("Hello {}!", req.match_info().get("name").unwrap())))
}

fn default_route(req: HttpRequest) -> HttpResponse {
    println!("default route called");
    println!("request was: {:?}", req);
    HttpResponse::NotFound().finish()
}

fn main() -> std::io::Result<()> {
    let system = System::new("Akriviea");
    env::set_var("RUST_LOG", "actix_server=debug,actix_web=debug");
    env_logger::init();

    let beacon_manager = beacon_manager::BeaconManager::new().start();

    beacon_manager.do_send(beacon_manager::ScanForBeacons);

    let state = web::Data::new(Mutex::new(AkriveiaState {
        beacon_manager: beacon_manager,
    }));

    // start the webserver
    HttpServer::new(move || {
        App::new()
            .register_data(state.clone())
            .wrap(middleware::DefaultHeaders::new().header("X-Version", "0.2"))
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(web::resource(common::PING).to(hello))
            .service(scan_beacons)
            .service(web::resource(common::EMERGENCY).to(emergency))
            .service(web::resource(common::END_EMERGENCY).to(end_emergency))
            .service(web::resource(common::DIAGNOSTICS).to_async(async_diagnostics))
            //.service(web::resource("/ad").to_async(async_diagnostics))
            // these two last !!
            .service(fs::Files::new("/", "static/").index_file("index.html"))
            .default_service(web::resource("").to(default_route))
    })
    .bind("0.0.0.0:8080")?
    .start();

    system.run()
}

