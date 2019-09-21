
use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use common::*;
use crate::AkriveiaState;
// What does the OutUserData stuct do?
use crate::data_processor::OutUserData;
use crate::db_utils;
use crate::models::user;
use futures::{ future::ok, Future, future::Either, };
use std::sync::*;

pub fn realtime_users(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let s = state.lock().unwrap();
    s.data_processor
        .send(OutUserData{})
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

pub fn get_user(_state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                .and_then(move |client| {
                    user::select_user(client, id)
                })
                .map_err(|postgres_err| {
                    // TODO can this be better?
                    error::ErrorBadRequest(postgres_err)
                })
                .and_then(|(_client, user)| {
                    match user {
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

pub fn get_users(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            user::select_users(client)
        })
        .map_err(|postgres_err| {
            // TODO can this be better?
            // More specific error message (UserRequestError)
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, users)| {
            HttpResponse::Ok().json(users)
        })
}

// new user
pub fn post_user(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<TrackedUser>) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            user::insert_user(client, payload.0)
        })
        .map_err(|postgres_err| {
            println!("{}", postgres_err);
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, user)| {
            match user {
                Some(u) => HttpResponse::Ok().json(u),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

// update user
pub fn put_user(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<TrackedUser>) -> impl Future<Item=HttpResponse, Error=Error> {
    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            user::update_user(client, payload.0)
        })
        .map_err(|postgres_err| {
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, user)| {
            match user {
                Some(u) => HttpResponse::Ok().json(u),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

pub fn delete_user(_state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) => {
            Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                .and_then(move |client| {
                    user::delete_user(client, id)
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

