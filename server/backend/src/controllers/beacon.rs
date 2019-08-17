use actix_web::{ get, middleware, Error, web, App, HttpRequest, HttpResponse, HttpServer, };
use crate::AkriveiaState;
use crate::data_processor::OutUserData;
use futures::{ future::ok, Future, };
use std::sync::*;
use std::thread::*;

pub fn get_beacon(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(beacon_id) => {
            ok(HttpResponse::Ok().json(common::Beacon::new(beacon_id.to_string())))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

pub fn get_beacons(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    ok(HttpResponse::Ok().json(vec![common::Beacon::new("dummy_id".to_string())]))
}

// new beacon
pub fn post_beacon(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(beacon_id) => {
            ok(HttpResponse::Ok().json(common::Beacon::new(beacon_id.to_string())))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

// update beacon
pub fn put_beacon(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(beacon_id) => {
            ok(HttpResponse::Ok().json(common::Beacon::new(beacon_id.to_string())))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

pub fn delete_beacon(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(beacon_id) => {
            ok(HttpResponse::Ok().json(common::Beacon::new(beacon_id.to_string())))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

