
use common::*;
use futures::{ Stream, Future, };
use na;
use tokio_postgres::row::Row;
use tokio_postgres::types::Type;

fn row_to_beacon(row: Option<Row>) -> Option<Beacon> {
    match row {
        Some(data) => {
            let mut b = Beacon::new();
            for (i, column) in data.columns().iter().enumerate() {
                match column.name() {
                    "id" => b.id = data.get(i),
                    "mac_address" => b.mac_address = data.get(i),
                    "coordinates" => {
                        let coordinates: Vec<f64> = data.get(i);
                        b.coordinates = na::Vector2::new(coordinates[0], coordinates[1]);
                    }
                    "map_id" => b.map_id = data.get(i),
                    "name" => b.name = data.get(i),
                    "note" => b.note = data.get(i),
                    unhandled => { panic!("unhandled beacon column {}", unhandled); },
                }
            }

            Some(b)
        },
        None => None,
    }
}

pub fn select_beacon(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=tokio_postgres::Error> {
    client
        .prepare("
            SELECT * FROM runtime.beacons
            WHERE id = $1::INTEGER
        ")
        .and_then(move |statement| {
            client
                .query(&statement, &[&id])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    (client, row_to_beacon(row))
                })
        })
}

#[allow(dead_code)]
pub fn select_beacon_by_mac(mut client: tokio_postgres::Client, mac_address: String) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=tokio_postgres::Error> {
    client
        .prepare("
            SELECT FROM runtime.beacons
            WHERE mac_address = $1::MACADDR
        ")
        .and_then(move |statement| {
            client
                .query(&statement, &[&mac_address])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    (client, row_to_beacon(row))
                })
        })
}

pub fn insert_beacon(mut client: tokio_postgres::Client, beacon: Beacon) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            INSERT INTO runtime.beacons (
                mac_address,
                coordinates,
                map_id,
                name,
                note
            )
            VALUES( $1, $2, $3, $4, $5 )
            RETURNING *
        ", &[
            Type::MACADDR,
            Type::FLOAT8_ARRAY,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::VARCHAR,
        ])
        .and_then(move |statement| {
            let coords = vec![beacon.coordinates[0], beacon.coordinates[1]];
            client
                .query(&statement, &[
                   &beacon.mac_address,
                   &coords,
                   &beacon.map_id,
                   &beacon.name,
                   &beacon.note
                ])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    (client, row_to_beacon(row))
                })
        })
}

pub fn update_beacon(mut client: tokio_postgres::Client, beacon: Beacon) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            UPDATE runtime.beacons
            SET
                mac_address = $1,
                coordinates = $2,
                map_id = $3,
                name = $4,
                note = $5
             WHERE
                id = $6
            RETURNING *
        ", &[
            Type::MACADDR,
            Type::FLOAT8_ARRAY,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::INT4,
        ])
        .and_then(move |statement| {
            let coords = vec![beacon.coordinates[0], beacon.coordinates[1]];
            client
                .query(&statement, &[
                   &beacon.mac_address,
                   &coords,
                   &beacon.map_id,
                   &beacon.name,
                   &beacon.note,
                   &beacon.id,
                ])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    (client, row_to_beacon(row))
                })
        })
}

pub fn delete_beacon(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            DELETE FROM runtime.beacons
            WHERE (
                id = $1
            )
        ", &[
            Type::INT4,
        ])
        .and_then(move |statement| {
            client
                .query(&statement, &[&id])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(_row, _next)| {
                    client
                })
        })
}
