use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use crate::AkriveiaState;
use crate::beacon_manager::OutBeaconData;
use crate::db_utils;
use crate::models::beacon;
use futures::{ future::ok, Future, future::Either, };
use serde_derive::{ Deserialize, };
use std::sync::*;
use actix_identity::Identity;

#[derive(Deserialize)]
pub struct GetParams {
    prefetch: Option<bool>,
}

pub fn beacons_status(_uid: Identity, state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let s = state.lock().unwrap();
    s.beacon_manager
        .send(OutBeaconData{})
        .then(|res| {
            match res {
                Ok(Ok(data)) => {
                    ok(HttpResponse::Ok().json(data))
                },
                _ => {
                    ok(HttpResponse::BadRequest().finish())
                }
        }})
}

pub fn get_beacon(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    let prefetch = params.prefetch.unwrap_or(false);
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    let fut = if prefetch {
                        Either::A(beacon::select_beacon(client, id))
                    } else {
                        Either::B(beacon::select_beacon_prefetch(client, id))
                    };

                    ok(fut).flatten()
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            println!("faill {:?}", postgres_err);
                            error::ErrorBadRequest(postgres_err)
                        })
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

pub fn get_beacons_for_map(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    beacon::select_beacons_for_map(client, id)
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            error::ErrorBadRequest(postgres_err)
                        })
                })
                .and_then(|(_client, beacons)| {
                    HttpResponse::Ok().json(beacons)
                })
            )
        },
        _ => {
            Either::B(ok(HttpResponse::NotFound().finish()))
        }
    }
}

pub fn get_beacons(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=Error> {
    let prefetch = params.prefetch.unwrap_or(false);
    let connect = db_utils::connect_id(&uid, &state);

    if prefetch {
        Either::A(
            connect
                .and_then(move |client| {
                    beacon::select_beacons_prefetch(client)
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            println!("error is {:?}", postgres_err);
                            error::ErrorBadRequest(postgres_err)
                        })
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
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            error::ErrorBadRequest(postgres_err)
                        })
                })
                .and_then(|(_client, beacons)| {
                    HttpResponse::Ok().json(beacons)
                })
        )
    }
}

// new beacon
pub fn post_beacon(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<common::Beacon>) -> impl Future<Item=HttpResponse, Error=Error> {

    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            beacon::insert_beacon(client, payload.0)
                .map_err(|postgres_err| {
                    println!("fail {}", postgres_err);
                    error::ErrorBadRequest(postgres_err)
                })
        })
        .and_then(|(_client, beacon)| {
            match beacon {
                Some(b) => HttpResponse::Ok().json(b),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

// update beacon
pub fn put_beacon(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<common::Beacon>) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            beacon::update_beacon(client, payload.0)
                .map_err(|postgres_err| {
                    println!("faill {:?}", postgres_err);
                    error::ErrorBadRequest(postgres_err)
                })
        })
        .and_then(|(_client, beacon)| {
            match beacon {
                Some(b) => HttpResponse::Ok().json(b),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

pub fn delete_beacon(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    beacon::delete_beacon(client, id)
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            error::ErrorBadRequest(postgres_err)
                        })
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
}

