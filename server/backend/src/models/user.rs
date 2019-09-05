
use common::*;
use futures::{ Stream, Future, IntoFuture, };
use na;
use tokio_postgres::row::Row;
use tokio_postgres::types::Type;

fn row_to_user(row: Row) -> TrackedUser {
    let mut entry = TrackedUser::new();
    for (i, column) in row.columns().iter().enumerate() {
        match column.name() {
            "u_id" => entry.id = row.get(i),
            "u_coordinates" => {
                let coords: Vec<f64> = row.get(i);
                entry.coordinates = na::Vector2::new(coords[0], coords[1]);
            }
            "u_emergency_contact" => entry.emergency_contact = row.get(i),
            "u_employee_id" => entry.employee_id = row.get(i),
            "u_last_active" => entry.last_active = row.get(i),
            "u_mac_address" => entry.mac_address = row.get(i),
            "u_map_id" => entry.map_id = row.get(i),
            "u_name" => entry.name = row.get(i),
            "u_note" => entry.note = row.get(i),
            "u_phone_number" => entry.phone_number = row.get(i),
            unhandled if unhandled.starts_with("u_") => { panic!("unhandled beacon column {}", unhandled); },
            _ => {},
        }
    }
    entry
}

pub fn select_users(mut client: tokio_postgres::Client) -> impl Future<Item=(tokio_postgres::Client, Vec<TrackedUser>), Error=tokio_postgres::Error> {
    // TODO paging
    client
        .prepare("
            SELECT * FROM runtime.users
        ")
        .and_then(move |statement| {
            client
                .query(&statement, &[])
                .collect()
                .into_future()
                .map(|rows| {
                    (client, rows.into_iter().map(|row| row_to_user(row)).collect())
                })
        })
}

pub fn select_user(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=tokio_postgres::Error> {
    client
        .prepare("
            SELECT * FROM runtime.users
            WHERE u_id = $1::INTEGER
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
                        Some(r) => (client, Some(row_to_user(r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn insert_user(mut client: tokio_postgres::Client, user: TrackedUser) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            INSERT INTO runtime.users (
                u_coordinates,
                u_emergency_contact,
                u_employee_id,
                u_last_active,
                u_mac_address,
                u_map_id,
                u_name,
                u_note,
                u_phone_number
            )
            VALUES( $1, $2, $3, $4, $5, $6, $7, $8, $9 )
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::INT4,
            Type::VARCHAR,
            Type::ABSTIME,
            Type::MACADDR,
            Type::INT4,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::VARCHAR,
        ])
        .and_then(move |statement| {
            let coordinates = vec![user.coordinates[0], user.coordinates[1]];
            client
                .query(&statement, &[
                    &coordinates,
                    &user.emergency_contact,
                    &user.employee_id,
                    &user.last_active,
                    &user.mac_address,
                    &user.map_id,
                    &user.name,
                    &user.note,
                    &user.phone_number,
                ])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_user(r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_user(mut client: tokio_postgres::Client, user: TrackedUser) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            UPDATE runtime.users
            SET
                u_coordinates = $1,
                u_emergency_contact = $2,
                u_employee_id = $3,
                u_last_active = $4,
                u_mac_address = $5,
                u_map_id = $6,
                u_name = $7,
                u_note = $8,
                u_phone_number = $9,
             WHERE
                u_id = $11
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::INT4,
            Type::VARCHAR,
            Type::ABSTIME,
            Type::MACADDR,
            Type::INT4,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::VARCHAR,
            Type::INT4,
        ])
        .and_then(move |statement| {
            let coordinates = vec![user.coordinates[0], user.coordinates[1]];
            client
                .query(&statement, &[
                    &coordinates,
                    &user.emergency_contact,
                    &user.employee_id,
                    &user.last_active,
                    &user.mac_address,
                    &user.map_id,
                    &user.name,
                    &user.note,
                    &user.phone_number,
                    &user.id,
                ])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_user(r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn delete_user(mut client: tokio_postgres::Client, id: i32) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            DELETE FROM runtime.users
            WHERE (
                u_id = $1
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
