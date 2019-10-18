
use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use actix_identity::Identity;
use common::*;
use crate::AkriveiaState;
use crate::db_utils;
use std::sync::*;
use futures::future::{ Either, ok, Future, };

pub fn login(id: Identity, state: web::Data<Mutex<AkriveiaState>>, payload: web::Json<LoginInfo>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    if payload.name == "responder" {
        let mut info = LoginInfo::new();
        info.name = payload.name.clone();
        info.pw = payload.name.clone();
        id.remember(payload.name.clone());
        let mut s = state.lock().unwrap();
        s.pools.insert(info.name.clone(), info);
        Either::A(ok(HttpResponse::Ok().finish()))
    } else {
        Either::B(db_utils::connect_login(&payload.0)
            .and_then(move |_client| {
                id.remember(payload.name.clone());
                let mut s = state.lock().unwrap();
                s.pools.insert(payload.name.clone(), payload.0);
                ok(HttpResponse::Ok().finish())
            })
            .map_err(|_postgres_err| {
                error::ErrorUnauthorized("Invalid Login Credentials.")
            })
        )
    }
}

pub fn check(id: Identity, _state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    if let Some(_name) = id.identity() {
        ok(HttpResponse::Ok().finish())
    } else {
        ok(HttpResponse::Unauthorized().finish())
    }
}

pub fn logout(id: Identity, _state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    id.forget();
    ok(HttpResponse::Ok().finish())
}

