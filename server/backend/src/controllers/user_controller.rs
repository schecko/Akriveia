
use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use common::*;
use crate::AKData;
use crate::data_processor::OutUserData;
use crate::db_utils;
use crate::models::user;
use futures::{ future::ok, Future, future::Either, };
use serde_derive::{ Deserialize, };
use std::sync::*;
use actix_identity::Identity;

#[derive(Deserialize)]
pub struct GetParams {
    prefetch: Option<bool>,
    include_contacts: Option<bool>,
}

pub fn users_status(_uid: Identity, state: AKData, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
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

pub fn get_user(uid: Identity, state: AKData, req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    let prefetch = params.prefetch.unwrap_or(false);
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    let fut = if prefetch {
                        Either::A(user::select_user_prefetch(client, id))
                    } else {
                        Either::B(user::select_user(client, id))
                    };

                    ok(fut).flatten()
                        .map_err(|postgres_err| {
                            // TODO can this be better?
                            error::ErrorBadRequest(postgres_err)
                        })
                })
                .and_then(|(_client, opt_user, opt_e_user)| {
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

pub fn get_users(uid: Identity, state: AKData, _req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=Error> {
    let include_contacts = params.include_contacts.unwrap_or(true);
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            user::select_users(client, include_contacts)
                .map_err(|postgres_err| {
                    // TODO can this be better?
                    error::ErrorBadRequest(postgres_err)
                })
        })
        .and_then(|(_client, users)| {
            HttpResponse::Ok().json(users)
        })
}

// new user
pub fn post_user(uid: Identity, state: AKData, _req: HttpRequest, payload: web::Json<(TrackedUser, Option<TrackedUser>)>) -> impl Future<Item=HttpResponse, Error=Error> {
    let (user, opt_e_user) = payload.into_inner();

    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            user::insert_user(client, user)
                .and_then(move |(client, opt_user)| {
                    match &opt_user {
                        Some(created_user) => {
                            match opt_e_user {
                                Some(mut e_user) => {
                                    e_user.attached_user = Some(created_user.id);
                                    Either::A(user::insert_user(client, e_user)
                                        .map(move |(client, created_e_user)| {
                                            (client, opt_user, created_e_user)
                                        })
                                    )
                                },
                                None => Either::B(ok((client, opt_user, None))),
                            }

                        },
                        None => {
                            Either::B(ok((client, None, None)))
                        },
                    }
                })
                .map_err(|postgres_err| {
                    println!("{}", postgres_err);
                    error::ErrorBadRequest(postgres_err)
                })
        })
        .and_then(|(_client, user, opt_e_user)| {
            match user {
                Some(u) => HttpResponse::Ok().json((u, opt_e_user)),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

pub fn put_user(uid: Identity, state: AKData, _req: HttpRequest, payload: web::Json<(TrackedUser, Option<TrackedUser>)>) -> impl Future<Item=HttpResponse, Error=Error> {
    let (user, opt_e_user) = payload.into_inner();

    db_utils::connect_id(&uid, &state)
        .and_then(|client| {
            user::update_user(client, user)
                .and_then(move |(client, opt_user)| {
                    match opt_e_user {
                        Some(e_user) => {
                            let fut = if e_user.id != -1 {
                                Either::A(user::update_user(client, e_user))
                            } else {
                                Either::B(user::insert_user(client, e_user))
                            };

                            Either::A(fut.map(move |(client, opt_e_user)| {
                                    (client, opt_user, opt_e_user)
                                })
                            )
                        },
                        None => {
                            Either::B(ok((client, opt_user, None)))
                        }
                    }
                })
                .map_err(|postgres_err| {
                    error::ErrorBadRequest(postgres_err)
                })
        })
        .and_then(|(_client, opt_user, opt_e_user)| {
            match opt_user {
                Some(u) => HttpResponse::Ok().json((u, opt_e_user)),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

pub fn delete_user(uid: Identity, state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    user::delete_user(client, id)
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

