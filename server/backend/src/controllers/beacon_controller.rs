use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use crate::AkriveiaState;
use crate::db_utils;
use crate::models::beacon;
use futures::{ Stream, IntoFuture, };
use crate::models::map;
use futures::{ future::ok, Future, future::Either, };
use serde_derive::{ Deserialize, };
use std::sync::*;

pub fn get_beacon(_state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
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

#[derive(Deserialize)]
pub struct GetBeaconsParams {
    prefetch: Option<bool>,
}

pub fn get_beacons(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, params: web::Query<GetBeaconsParams>) -> impl Future<Item=HttpResponse, Error=Error> {
    let prefetch = params.prefetch.unwrap_or(false);

    if prefetch {
        println!("prefetch is true");
        Either::A(
            db_utils::connect(db_utils::DEFAULT_CONNECTION)
                .and_then(move |mut client| {
                    client
                        .prepare("
                            SELECT *
                            FROM runtime.maps AS map, runtime.beacons AS beacon
                            WHERE map.m_id = beacon.b_map_id
                        ")
                        .and_then(move |statement| {
                            client
                                .query(&statement, &[])
                                .collect()
                                .into_future()
                                .map(|rows| -> (tokio_postgres::Client, Vec<(common::Beacon, common::Map)>) {
                                    (
                                        client,
                                        rows
                                            .into_iter()
                                            .map(|row| (beacon::row_to_beacon(&row), map::row_to_map(&row)))
                                            .collect()
                                    )
                                })
                        })
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
            db_utils::connect(db_utils::DEFAULT_CONNECTION)
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

