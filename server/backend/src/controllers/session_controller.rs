
use actix_web::{ error, Error, web, HttpRequest, HttpResponse, };
use common::*;
use crate::AkriveiaState;
use crate::db_utils;
use serde_derive::{ Deserialize, };
use std::sync::*;
use futures::future::{ ok, Future, };

pub fn login(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    ok(HttpResponse::Ok().finish())
}

pub fn logout(_state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    ok(HttpResponse::Ok().finish())
}

