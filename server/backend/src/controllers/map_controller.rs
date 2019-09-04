
use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use common::*;
use crate::AkriveiaState;
use crate::db_utils;
use crate::models::map;
use futures::{ future::ok, Future, future::Either, };
use std::sync::*;

pub fn get_map(_state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id_string_out = req.match_info().get("id");
    match id_string_out {
        Some(id_string) => {
            match id_string.parse::<i32>() {
                Ok(id) => {
                    Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                        .and_then(move |client| {
                            map::select_map(client, id)
                        })
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            error::ErrorBadRequest(postgres_err)
                        })
                        .and_then(|(_client, map)| {
                            match map {
                                Some(m) => HttpResponse::Ok().json(m),
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

pub fn get_maps(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            map::select_maps(client)
        })
        .map_err(|postgres_err| {
            // TODO can this be better?
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, maps)| {
            HttpResponse::Ok().json(maps)
        })
}

// new map
pub fn post_map(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<Map>) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            map::insert_map(client, payload.0)
        })
        .map_err(|postgres_err| {
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, map)| {
            match map {
                Some(m) => HttpResponse::Ok().json(m),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

// update map
pub fn put_map(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<Map>) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            map::update_map(client, payload.0)
        })
        .map_err(|postgres_err| {
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, map)| {
            match map {
                Some(m) => HttpResponse::Ok().json(m),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

pub fn delete_map(_state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id_string_out = req.match_info().get("id");
    match id_string_out {
        Some(id_string) => {
            match id_string.parse::<i32>() {
                Ok(id) => {
                    Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                        .and_then(move |client| {
                            map::delete_map(client, id)
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

