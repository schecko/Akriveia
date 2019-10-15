
use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use actix_identity::Identity;
use common::*;
use crate::AkriveiaState;
use crate::db_utils;
use serde_derive::{ Deserialize, };
use std::sync::*;
use futures::future::{ ok, Future, };

pub fn login(id: Identity, _state: web::Data<Mutex<AkriveiaState>>, payload: web::Json<LoginInfo>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {

    req.headers().iter().for_each(|(k, v)| println!("kv {}, {:?}", k, v));
    db_utils::connect_login(&payload.0)
        .and_then(move |client| {
            id.remember(payload.name.clone());
            ok(HttpResponse::Ok().finish())
        })
        .map_err(|_postgres_err| {
            error::ErrorUnauthorized("")
        })
}

pub fn logout(id: Identity, _state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    id.forget();
    ok(HttpResponse::Ok().finish())
}

