
use common::*;
use futures::{ Stream, Future, IntoFuture, };
use na;
use tokio_postgres::row::Row;
use tokio_postgres::types::Type;
use crate::models::map;

pub fn row_to_beacon(row: &Row) -> Beacon {
    let mut b = Beacon::new();
    for (i, column) in row.columns().iter().enumerate() {
        match column.name() {
            "b_id" => b.id = row.get(i),
            "b_mac_address" => b.mac_address = row.get(i),
            "b_ip" => b.ip = row.get(i),
            "b_coordinates" => {
                let coordinates: Vec<f64> = row.get(i);
                b.coordinates = na::Vector2::new(coordinates[0], coordinates[1]);
            }
            "b_map_id" => b.map_id = row.get(i),
            "b_name" => b.name = row.get(i),
            "b_note" => b.note = row.get(i),
            unhandled if unhandled.starts_with("b_") => { panic!("unhandled beacon column {}", unhandled); },
            _ => {},
        }
    }
    b
}

pub fn select_beacons(mut client: tokio_postgres::Client) -> impl Future<Item=(tokio_postgres::Client, Vec<Beacon>), Error=tokio_postgres::Error> {
    // TODO paging
    client
        .prepare("
            SELECT * FROM runtime.beacons
        ")
        .and_then(move |statement| {
            client
                .query(&statement, &[])
                .collect()
                .into_future()
                .map(|rows| {
                    (client, rows.into_iter().map(|row| row_to_beacon(&row)).collect())
                })
        })
}

pub fn select_beacons_prefetch(mut client: tokio_postgres::Client) -> impl Future<Item=(tokio_postgres::Client, Vec<(Beacon, Map)>), Error=tokio_postgres::Error> {
    // TODO paging
    client
        .prepare("
            SELECT *
            FROM runtime.maps AS map, runtime.beacons AS beacon
            WHERE map.m_id = beacon.b_map_id
        ")
        .and_then(move |statement| {
            client
                .query(&statement, &[])
                .collect()
                .into_future()
                .map(|rows| -> (tokio_postgres::Client, Vec<(common::Beacon, common::Map)>) {
                    (
                        client,
                        rows
                            .into_iter()
                            // this works because the row conversion functions only
                            // look for entries specific to the object they are
                            // converting for, and the keys are all unique.
                            .map(|row| (row_to_beacon(&row), map::row_to_map(&row)))
                            .collect()
                    )
                })
        })
}

pub fn select_beacon(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<(Beacon, Option<Map>)>), Error=tokio_postgres::Error> {
    client
        .prepare("
            SELECT * FROM runtime.beacons
            WHERE b_id = $1::INTEGER
        ")
        .and_then(move |statement| {
            client
                .query(&statement, &[&id])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some((row_to_beacon(&r), None))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn select_beacon_prefetch(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<(Beacon, Option<Map>)>), Error=tokio_postgres::Error> {
    // TODO paging
    client
        .prepare_typed("
            SELECT *
            FROM runtime.maps AS map, runtime.beacons AS beacon
            WHERE
                map.m_id = beacon.b_map_id
                AND beacon.b_id = $1
        ", &[
            Type::INT4
        ])
        .and_then(move |statement| {
            client
                .query(&statement, &[&id])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some((row_to_beacon(&r), Some(map::row_to_map(&r))))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn insert_beacon(mut client: tokio_postgres::Client, beacon: Beacon) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            INSERT INTO runtime.beacons (
                b_mac_address,
                b_coordinates,
                b_map_id,
                b_name,
                b_note
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
                    match row {
                        Some(r) => (client, Some(row_to_beacon(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_beacon(mut client: tokio_postgres::Client, beacon: Beacon) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            UPDATE runtime.beacons
            SET
                b_mac_address = $1,
                b_coordinates = $2,
                b_map_id = $3,
                b_name = $4,
                b_note = $5
             WHERE
                b_id = $6
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
                    match row {
                        Some(r) => (client, Some(row_to_beacon(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn delete_beacon(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            DELETE FROM runtime.beacons
            WHERE (
                b_id = $1
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
