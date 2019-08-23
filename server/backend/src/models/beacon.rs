
use futures::{ Stream, Future, };
use common::*;
use tokio_postgres::row::Row;
use tokio_postgres::types::Type;
use na;

fn row_to_beacon(row: Option<Row>) -> Option<Beacon> {
    match row {
        Some(data) => {
            let coordinates: Vec<f64> = data.get(1);
            for column in data.columns().iter() {
                println!("col {:?}", column);
            }
            Some(Beacon {
                mac_address: data.get(0),
                coordinates: na::Vector2::new(coordinates[0], coordinates[1]),
                map_id: data.get(2),
                name: data.get(3),
                note: data.get(4),
            })
        },
        None => None,
    }
}

pub fn select_beacon(mut client: tokio_postgres::Client, mac_address: String) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=tokio_postgres::Error> {
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
