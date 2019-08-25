
use actix_web::{ Error, web, HttpRequest, HttpResponse, };
use crate::AkriveiaState;
use futures::{ future::ok, Future, };
use std::sync::*;

pub fn get_map(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(_map_id) => {
            ok(HttpResponse::Ok().json(common::Map::new("dunno".to_string())))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

pub fn get_maps(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    ok(HttpResponse::Ok().json(vec![common::Map::new("dunno".to_string())]))
}

// new beacon
pub fn post_map(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(_map_id) => {
            ok(HttpResponse::Ok().json(common::Map::new("dunno".to_string())))
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
        Some(_map_id) => {
            ok(HttpResponse::Ok().json(common::Map::new("dunno".to_string())))
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
        Some(_map_id) => {
            ok(HttpResponse::Ok().json(common::Map::new("dunno".to_string())))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

