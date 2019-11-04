
use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use common::*;
use crate::AkriveiaState;
use crate::db_utils;
use crate::models::map;
use futures::{ Stream, future::ok, Future, future::Either, };
use std::sync::*;
use actix_identity::Identity;

pub fn get_map(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    map::select_map(client, id)
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            error::ErrorBadRequest(postgres_err)
                        })
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
}

pub fn get_map_blueprint(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    map::select_map_blueprint(client, id)
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            error::ErrorBadRequest(postgres_err)
                        })
                })
                .and_then(|(_client, res)| {
                    match res {
                        Some(img) => HttpResponse::Ok().body(img),
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

pub fn get_maps(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            map::select_maps(client)
                .map_err(|postgres_err| {
                    // TODO can this be better?
                    error::ErrorBadRequest(postgres_err)
                })
        })
        .and_then(|(_client, maps)| {
            HttpResponse::Ok().json(maps)
        })
}

// new map
pub fn post_map(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<Map>) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            map::insert_map(client, payload.0)
                .map_err(|postgres_err| {
                    error::ErrorBadRequest(postgres_err)
                })
        })
        .and_then(|(_client, map)| {
            match map {
                Some(m) => HttpResponse::Ok().json(m),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

// update map
pub fn put_map(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<Map>) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            map::update_map(client, payload.0)
                .map_err(|postgres_err| {
                    error::ErrorBadRequest(postgres_err)
                })
        })
        .and_then(|(_client, map)| {
            match map {
                Some(m) => HttpResponse::Ok().json(m),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

// update map blueprint
pub fn put_map_blueprint(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest, payload: web::Payload) -> impl Future<Item=HttpResponse, Error=Error> {
    println!("put map blueprint called");
    let mid = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match mid {
        Ok(id) => {
            Either::A(payload
                .map_err(Error::from)
                .fold(web::BytesMut::new(), move |mut acc_body, chunk| {
                    println!("file chunk");
                    acc_body.extend_from_slice(&chunk);
                    Ok::<_, Error>(acc_body)
                })
                .and_then(move |blueprint_img| {
                    println!("connecting");
                    db_utils::connect_id(&uid, &state)
                        .and_then(move |client| {
                            println!("query");
                            map::update_map_blueprint(client, id, blueprint_img)
                                .map_err(|postgres_err| {
                                    dbg!(&postgres_err);
                                    error::ErrorBadRequest(postgres_err)
                                })
                        })
                })
                .map(|_client| {
                    println!("successful upload");
                    HttpResponse::Ok().finish()
                })
            )
        },
        _ => {
            Either::B(ok(HttpResponse::NotFound().finish()))
        }
    }
}

pub fn delete_map(uid: Identity, state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    map::delete_map(client, id)
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

