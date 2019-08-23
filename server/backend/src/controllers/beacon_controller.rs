use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use crate::AkriveiaState;
use futures::{ future::ok, Future, future::Either, };
use std::sync::*;
use crate::models::beacon;
use crate::db_utils;

pub fn get_beacon(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id_string_out = req.match_info().get("id");
    match id_string_out {
        Some(id_string) => {
            match id_string.parse::<i32>() {
                Ok(id) => {
                    Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                        .and_then(move |client| {
                            beacon::select_beacon(client, id)
                        })
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            error::ErrorBadRequest(postgres_err)
                        })
                        .and_then(|(_client, beacon)| {
                            match beacon {
                                Some(b) => HttpResponse::Ok().json(b),
                                None => HttpResponse::NotFound().finish(),
                            }
                        })
                    )
                },
                _ => {
                    Either::B(ok(HttpResponse::NotFound().finish()))
                }
            }
        },
        _ => {
            Either::B(ok(HttpResponse::NotFound().finish()))
        }
    }
}

pub fn get_beacons(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    ok(HttpResponse::Ok().json(vec![common::Beacon::new()]))
}

// new beacon
pub fn post_beacon(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<common::Beacon>) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            beacon::insert_beacon(client, payload.0)
        })
        .map_err(|postgres_err| {
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, beacon)| {
            match beacon {
                Some(b) => HttpResponse::Ok().json(b),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

// update beacon
pub fn put_beacon(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(_beacon_id) => {
            ok(HttpResponse::Ok().json(common::Beacon::new()))
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
        Some(_beacon_id) => {
            ok(HttpResponse::Ok().json(common::Beacon::new()))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

