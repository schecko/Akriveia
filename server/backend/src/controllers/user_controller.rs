
use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use common::*;
use crate::AkriveiaState;
// What does the OutUserData stuct do?
// It's for data_processor to probably update user position
use crate::data_processor::OutUserData;
use crate::db_utils;
use crate::models::user;
use futures::{ future::ok, Future, future::Either, };
use serde_derive::{ Deserialize, };
use std::sync::*;

#[derive(Deserialize)]
pub struct GetParams {
    prefetch: Option<bool>,
}

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

pub fn get_user(_state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    let prefetch = params.prefetch.unwrap_or(false);
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect(db_utils::DEFAULT_CONNECTION)
                .and_then(move |client| {
                    if prefetch {
                        Either::A(user::select_user_prefetch(client, id))
                    } else {
                        Either::B(user::select_user(client, id))
                    }
                })
                .map_err(|postgres_err| {
                    // TODO can this be better?
                    error::ErrorBadRequest(postgres_err)
                })
                .and_then(|(_client, opt_user, opt_e_user)| {
                    // follow what was done in update_user
                    match opt_user {
                        Some(u) => {
                                HttpResponse::Ok().json((Some(u), opt_e_user))
                        },
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
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, users)| {
            HttpResponse::Ok().json(users)
        })
}

// new user
pub fn post_user(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<(TrackedUser, Option<TrackedUser>)>) -> impl Future<Item=HttpResponse, Error=Error> {
    let users = payload.into_inner();
    let mut user = users.0;
    let e_user = users.1;

    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            match e_user {
                Some(e) => Either::A(user::insert_user(client, e)),
                None => Either::B(ok((client, None))),
            }
        })
        .and_then(|(client, opt_e_user)| {
            match &opt_e_user {
                Some(e) => user.emergency_contact = Some(e.id),
                None => {},
            }
            user::insert_user(client, user)
                .map(|(_client, user)| {
                    (_client, user, opt_e_user)
                })
        })
        .map_err(|postgres_err| {
            println!("{}", postgres_err);
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, user, opt_e_user)| {
            match user {
                Some(u) => HttpResponse::Ok().json((u, opt_e_user)),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

// update user
pub fn put_user(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<(TrackedUser, Option<TrackedUser>)>) -> impl Future<Item=HttpResponse, Error=Error> {
    let users = payload.into_inner();
    let mut user = users.0;
    let opt_e_user = users.1;

    db_utils::connect(db_utils::DEFAULT_CONNECTION)
        .and_then(move |client| {
            match user.emergency_contact {
                Some(_contact) => {
                    let fut_a = match opt_e_user {
                        // update the emergency user with new info
                        Some(e) => Either::A(user::update_user(client, e)
                            .map(move |(client, opt_e)| (client, user, opt_e))),
                        // emergency user exists, but does not need to be updated(new data not provided)
                        None => Either::B(ok((client, user, None))),
                    };
                    Either::A(ok(fut_a).flatten())
                },
                None => {
                    let fut_b = match opt_e_user {
                        // emergency user needs to be created
                        Some(e) => Either::A(user::insert_user(client, e)
                                .and_then(move |(client, opt_e)| {
                                    if let Some(new_contact) = &opt_e {
                                        user.emergency_contact = Some(new_contact.id);
                                    }
                                    ok((client, user, opt_e))
                                })
                        ),
                        // do nothing, the contact doesnt exist, and changes are not required
                        None => Either::B(ok((client, user, None))),
                    };
                    Either::B(ok(fut_b).flatten())
                },
            }
        })
        .and_then(|(_client, user, opt_e_user)| {
            user::update_user(_client, user)
                .map(move |(client, opt_user)| {
                    (client, opt_user, opt_e_user)
                })
        })
        .map_err(|postgres_err| {
            error::ErrorBadRequest(postgres_err)
        })
        .and_then(|(_client, opt_user, opt_e_user)| {
            match opt_user {
                Some(u) => HttpResponse::Ok().json((u, opt_e_user)),
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

