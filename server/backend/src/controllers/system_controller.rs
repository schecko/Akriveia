use actix_web::{ Error, web, HttpRequest, HttpResponse, };
use crate::AkriveiaState;
use futures::{ future::ok, Future, };
use std::sync::*;
use crate::beacon_manager::{ BMCommand, GetDiagnosticData, };
use common::*;

pub fn post_emergency(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<SystemCommandResponse>) -> impl Future<Item=HttpResponse, Error=Error> {
    let s = state.lock().unwrap();
    let command = if payload.emergency {
        BMCommand::StartEmergency(None)
    } else {
        BMCommand::EndEmergency(None)
    };

    s.beacon_manager
        .send(command)
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

pub fn get_emergency(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let s = state.lock().unwrap();
    s.beacon_manager
        .send(BMCommand::GetEmergency)
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

pub fn diagnostics(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let s = state.lock().unwrap();
    s.beacon_manager
        .send(GetDiagnosticData)
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

