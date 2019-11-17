use actix::System;
use actix_identity::Identity;
use actix_web::{ Error, error, web, HttpRequest, HttpResponse, };
use common::*;
use crate::AKData;
use crate::WatcherCommand;
use crate::beacon_manager::{ BMCommand, GetDiagnosticData, };
use futures::{ future::err, future::ok, Future, };
use std::sync::*;

pub fn post_emergency(state: AKData, _req: HttpRequest, payload: web::Json<SystemCommandResponse>) -> impl Future<Item=HttpResponse, Error=Error> {
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

pub fn get_emergency(state: AKData, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
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

pub fn diagnostics(state: AKData, _req: HttpRequest) -> impl Future<Item=HttpResponse, Error=Error> {
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

pub fn restart(id: Identity, state: AKData, payload: web::Json<SystemCommand>) -> impl Future<Item=HttpResponse, Error=Error> {
    if let Some(name) = id.identity() {
        if name == "admin" {
            let s = state.lock().unwrap();
            let command = match payload.0 {
                SystemCommand::StartNormal => WatcherCommand::StartNormal,
                SystemCommand::RebuildDB => WatcherCommand::RebuildDB,
            };
            match s.tx.send(command) {
                Ok(()) => {},
                Err(e) => {
                    // TODO just change to println I guess...
                    panic!("Failed to notify watcher we are shutting down");
                },
            }

            let system = System::current();
            system.stop();
            // TODO test to see if this request ever returns
            ok(HttpResponse::Ok().finish())
        } else {
            err(error::ErrorUnauthorized("invalid credentials"))
        }
    } else {
        err(error::ErrorUnauthorized("invalid credentials"))
    }
}
