use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use crate::AkriveiaState;
use common::NetworkInterface;
use crate::db_utils;
use crate::models::network_interface;
use futures::{ future::ok, Future, future::Either, };
use std::sync::*;

pub fn get_network_interface(_state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                .and_then(move |client| {
                    network_interface::select_network_interface(client, id)
                })
                .map_err(|postgres_err| {
                    // TODO can this be better?
                    error::ErrorBadRequest(postgres_err)
                })
                .and_then(|(_client, iface)| {
                    match iface {
                        Some(u) => HttpResponse::Ok().json(u),
                        None => HttpResponse::NotFound().finish(),
                    }
                })
            )
        },
        _ => {
            Either::B(ok(HttpResponse::NotFound().finish()))
        },
    }
}

pub fn get_network_interfaces(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            network_interface::select_network_interfaces(client)
        })
        .map_err(|postgres_err| {
            // TODO can this be better?
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, ifaces)| {
            HttpResponse::Ok().json(ifaces)
        })
}

// new iface
pub fn post_network_interface(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<NetworkInterface>) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            network_interface::insert_network_interface(client, payload.0)
        })
        .map_err(|postgres_err| {
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, iface)| {
            match iface {
                Some(b) => HttpResponse::Ok().json(b),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

// update iface
pub fn put_network_interface(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<NetworkInterface>) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            network_interface::update_network_interface(client, payload.0)
        })
        .map_err(|postgres_err| {
            println!("faill {:?}", postgres_err);
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, iface)| {
            match iface {
                Some(b) => HttpResponse::Ok().json(b),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

pub fn delete_network_interface(_state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                .and_then(move |client| {
                    network_interface::delete_network_interface(client, id)
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
}

