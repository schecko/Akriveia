
use actix_web::{ get, middleware, Error, web, App, HttpRequest, HttpResponse, HttpServer, };
use crate::AkriveiaState;
use crate::data_processor::OutUserData;
use futures::{ future::ok, Future, };
use std::sync::*;
use std::thread::*;

pub fn get_map(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(map_id) => {
            ok(HttpResponse::Ok().json(common::Map::new()))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

pub fn get_maps(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    ok(HttpResponse::Ok().json(vec![common::Map::new()]))
}

// new beacon
pub fn post_map(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(map_id) => {
            ok(HttpResponse::Ok().json(common::Map::new()))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

// update beacon
pub fn put_map(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(map_id) => {
            ok(HttpResponse::Ok().json(common::Map::new()))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

pub fn delete_map(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(map_id) => {
            ok(HttpResponse::Ok().json(common::Map::new()))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

