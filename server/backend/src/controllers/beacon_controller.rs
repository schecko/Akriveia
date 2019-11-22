use actix_web::{ web, HttpRequest, HttpResponse, };
use crate::AKData;
use crate::beacon_manager::{ OutBeaconData, BMCommand, };
use crate::db_utils;
use crate::models::beacon;
use futures::{ future::ok, Future, future::Either, };
use serde_derive::{ Deserialize, };
use actix_identity::Identity;
use common::BeaconRequest;
use crate::ak_error::AkError;

#[derive(Deserialize)]
pub struct GetParams {
    prefetch: Option<bool>,
}

pub fn beacons_status(_uid: Identity, state: AKData) -> impl Future<Item=HttpResponse, Error=AkError> {
    let s = state.lock().unwrap();
    s.beacon_manager
        .send(OutBeaconData{})
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

pub fn beacon_command(_uid: Identity, state: AKData, payload: web::Json<common::BeaconRequest>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let s = state.lock().unwrap();
    let command = match payload.0 {
        BeaconRequest::StartEmergency(mac) => BMCommand::StartEmergency(mac),
        BeaconRequest::EndEmergency(mac) => BMCommand::EndEmergency(mac),
        BeaconRequest::Ping(mac) => BMCommand::Ping(mac),
        BeaconRequest::Reboot(mac) => BMCommand::Reboot(mac),
    };
    s.beacon_manager
        .send(command)
        .then(|res| {
            match res {
                Ok(Ok(_data)) => {
                    ok(HttpResponse::Ok().finish())
                },
                _ => {
                    ok(HttpResponse::BadRequest().finish())
                }
        }})
}

pub fn get_beacon(uid: Identity, state: AKData, req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    let prefetch = params.prefetch.unwrap_or(false);
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    let fut = if prefetch {
                        Either::A(beacon::select_beacon(client, id))
                    } else {
                        Either::B(beacon::select_beacon_prefetch(client, id))
                    };

                    ok::<_, AkError>(fut).flatten()
                })
                .map(|(_client, beacon_and_map)| {
                    match beacon_and_map {
                        Some((beacon, opt_map)) => {
                            match opt_map {
                                Some(map) => HttpResponse::Ok().json(Some((beacon, Some(map)))),
                                None => HttpResponse::Ok().json(Some(beacon)),
                            }

                        },
                        None => HttpResponse::NotFound().finish(),
                    }
                })
            )
        },
        _ => {
            Either::B(ok(HttpResponse::NotFound().finish()))
        }
    }
}

pub fn get_beacons_for_map(uid: Identity, state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    beacon::select_beacons_for_map(client, id)
                })
                .map(|(_client, beacons)| {
                    HttpResponse::Ok().json(beacons)
                })
            )
        },
        _ => {
            Either::B(ok(HttpResponse::NotFound().finish()))
        }
    }
}

pub fn get_beacons(uid: Identity, state: AKData, _req: HttpRequest, params: web::Query<GetParams>) -> impl Future<Item=HttpResponse, Error=AkError> {
    let prefetch = params.prefetch.unwrap_or(false);
    let connect = db_utils::connect_id(&uid, &state);

    if prefetch {
        Either::A(
            connect
                .and_then(move |client| {
                    beacon::select_beacons_prefetch(client)
                })
                .map(|(_client, beacons_and_maps)| {
                    HttpResponse::Ok().json(beacons_and_maps)
                })
        )
    } else {
        Either::B(
            connect
                .and_then(move |client| {
                    beacon::select_beacons(client)
                })
                .map(|(_client, beacons)| {
                    HttpResponse::Ok().json(beacons)
                })
        )
    }
}

// new beacon
pub fn post_beacon(uid: Identity, state: AKData, _req: HttpRequest, payload: web::Json<common::Beacon>) -> impl Future<Item=HttpResponse, Error=AkError> {
    println!("post beacon");

    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            println!("connect post beacon");
            beacon::insert_beacon(client, payload.0)
                .map_err(|_bleh| {
                    AkError::internal("wtf")
                })
        })
        .map(|(_client, beacon)| {
            println!("returning post beacon");
            match beacon {
                Some(b) => HttpResponse::Ok().json(b),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

// update beacon
pub fn put_beacon(uid: Identity, state: AKData, _req: HttpRequest, payload: web::Json<common::Beacon>) -> impl Future<Item=HttpResponse, Error=AkError> {
    db_utils::connect_id(&uid, &state)
        .and_then(move |client| {
            beacon::update_beacon(client, payload.0)
        })
        .map(|(_client, beacon)| {
            match beacon {
                Some(b) => HttpResponse::Ok().json(b),
                None => HttpResponse::NotFound().finish(),
            }
        })
}

pub fn delete_beacon(uid: Identity, state: AKData, req: HttpRequest) -> impl Future<Item=HttpResponse, Error=AkError> {
    let id = req.match_info().get("id").unwrap_or("-1").parse::<i32>();
    match id {
        Ok(id) if id != -1 => {
            Either::A(db_utils::connect_id(&uid, &state)
                .and_then(move |client| {
                    beacon::delete_beacon(client, id)
                })
                .map(|_client| {
                    HttpResponse::Ok().finish()
                })
            )
        },
        _ => {
            Either::B(ok(HttpResponse::NotFound().finish()))
        }
    }
}

