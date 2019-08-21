use actix_web::{ Error, web, HttpRequest, HttpResponse, };
use crate::AkriveiaState;
use futures::{ future::ok, Future, };
use std::sync::*;

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

pub fn get_beacons(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
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

