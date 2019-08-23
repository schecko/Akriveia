
use actix_web::{ Error, web, HttpRequest, HttpResponse, };
use crate::AkriveiaState;
use crate::data_processor::OutUserData;
use futures::{ future::ok, Future, };
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

pub fn get_user(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(_user_id) => {
            ok(HttpResponse::Ok().json(common::User::new()))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

pub fn get_users(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    ok(HttpResponse::Ok().json(vec![common::User::new()]))
}

// new user
pub fn post_user(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(_user_id) => {
            ok(HttpResponse::Ok().json(common::User::new()))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

// update user
pub fn put_user(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(_user_id) => {
            ok(HttpResponse::Ok().json(common::User::new()))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

pub fn delete_user(state: web::Data<Mutex<AkriveiaState>>, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let _ = state.lock().unwrap();
    let id = req.match_info().get("id");
    match id {
        Some(_user_id) => {
            ok(HttpResponse::Ok().json(common::User::new()))
        },
        None => {
            ok(HttpResponse::NotFound().finish())
        }
    }
}

