use actix_web::{ Error, web, HttpRequest, HttpResponse, };
use crate::AkriveiaState;
use futures::{ future::ok, Future, };
use std::sync::*;
use crate::beacon_manager::{ BeaconCommand, GetDiagnosticData, };
use common::*;

pub fn post_emergency(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest, payload: web::Json<SystemCommandResponse>) -> HttpResponse {
    let s = state.lock().unwrap();
    if payload.emergency {
        s.beacon_manager.do_send(BeaconCommand::StartEmergency);
    } else {
        s.beacon_manager.do_send(BeaconCommand::EndEmergency);
    }
    HttpResponse::Ok().finish()
}

pub fn get_emergency(state: web::Data<Mutex<AkriveiaState>>, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
    let s = state.lock().unwrap();
    s.beacon_manager
        .send(BeaconCommand::GetEmergency)
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

