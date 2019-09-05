use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use crate::AkriveiaState;
use crate::db_utils;
use crate::models::beacon;
use futures::{ future::ok, Future, future::Either, };
use serde_derive::{ Deserialize, };
use std::sync::*;

#[derive(Deserialize)]
pub struct GetParams {
    prefetch: Option<bool>,
}


pub fn get_beacon(_state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    let prefetch = params.prefetch.unwrap_or(false);
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                .and_then(move |client| {
                    if prefetch {
                        Either::A(beacon::select_beacon(client, id))
                    } else {
                        Either::B(beacon::select_beacon_prefetch(client, id))
                    }
                })
                .map_err(|postgres_err| {
                    // TODO can this be better?
                    error::ErrorBadRequest(postgres_err)
                })
                .and_then(|(_client, beacon_and_map)| {
                    match beacon_and_map {
                        Some((beacon, opt_map)) => {
                            match opt_map {
                                Some(map) => HttpResponse::Ok().json(Some((beacon, Some(map)))),
                                None => HttpResponse::Ok().json(Some(beacon)),
                            }

                        },
                        None => HttpResponse::NotFound().finish(),
                    }
                })
            )
        },
        _ => {
            Either::B(ok(HttpResponse::NotFound().finish()))
        }
    }
}

pub fn get_beacons(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=Error> {
    let prefetch = params.prefetch.unwrap_or(false);
    let connect = db_utils::connect(db_utils::DEFAULT_CONNECTION);

    if prefetch {
        Either::A(
            connect
                .and_then(move |client| {
                    beacon::select_beacons_prefetch(client)
                })
                .map_err(|postgres_err| {
                    // TODO can this be better?
                    error::ErrorBadRequest(postgres_err)
                })
                .and_then(|(_client, beacons_and_maps)| {
                    HttpResponse::Ok().json(beacons_and_maps)
                })
        )
    } else {
        Either::B(
            connect
                .and_then(move |client| {
                    beacon::select_beacons(client)
                })
                .map_err(|postgres_err| {
                    // TODO can this be better?
                    error::ErrorBadRequest(postgres_err)
                })
                .and_then(|(_client, beacons)| {
                    HttpResponse::Ok().json(beacons)
                })
        )
    }
}

// new beacon
pub fn post_beacon(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<common::Beacon>) -> impl Future<Item=HttpResponse, Error=Error> {
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
pub fn put_beacon(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<common::Beacon>) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            beacon::update_beacon(client, payload.0)
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

pub fn delete_beacon(_state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id_string_out = req.match_info().get("id");
    match id_string_out {
        Some(id_string) => {
            match id_string.parse::<i32>() {
                Ok(id) => {
                    Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                        .and_then(move |client| {
                            beacon::delete_beacon(client, id)
                        })
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            error::ErrorBadRequest(postgres_err)
                        })
                        .and_then(|_client| {
                            HttpResponse::Ok().finish()
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

