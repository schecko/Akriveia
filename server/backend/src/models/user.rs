
use common::*;
use futures::{ Stream, Future, IntoFuture, };
use na;
use tokio_postgres::row::Row;
use tokio_postgres::types::Type;

fn row_to_user(row: &Row) -> TrackedUser {
    let mut entry = TrackedUser::new();
    for (i, column) in row.columns().iter().enumerate() {
        match column.name() {
            "u_id" => entry.id = row.get(i),
            "u_coordinates" => {
                let coords: Vec<f64> = row.get(i);
                entry.coordinates = na::Vector2::new(coords[0], coords[1]);
            },
            "u_emergency_contact" => entry.emergency_contact = row.get(i),
            "u_employee_id" => entry.employee_id = row.get(i),
            "u_last_active" => entry.last_active = row.get(i),
            "u_mac_address" => entry.mac_address = row.get(i),
            "u_map_id" => entry.map_id = row.get(i),
            "u_name" => entry.name = row.get(i),
            "u_note" => entry.note = row.get(i),
            "u_work_phone" => entry.work_phone = row.get(i),
            "u_mobile_phone" => entry.mobile_phone = row.get(i),
            unhandled if unhandled.starts_with("u_") => { panic!("unhandled user column {}", unhandled); },
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
                    (client, rows.into_iter().map(|row| row_to_user(&row)).collect())
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
                        Some(r) => (client, Some(row_to_user(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn select_user_random(mut client: tokio_postgres::Client) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=tokio_postgres::Error> {
    client
        .prepare("
            SELECT *
            FROM runtime.users
            ORDER BY random()
            LIMIT 1
        ")
        .and_then(move |statement| {
            client
                .query(&statement, &[])
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
                u_work_phone,
                u_mobile_phone
            )
            VALUES( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10 )
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::INT4,
            Type::VARCHAR,
            Type::TIMESTAMPTZ,
            Type::MACADDR,
            Type::INT4,
            Type::VARCHAR,
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
                    &user.work_phone,
                    &user.mobile_phone,
                ])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_user(&r))),
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
                u_work_phone = $9,
                u_mobile_phone = $10
             WHERE
                u_id = $11
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::INT4,
            Type::VARCHAR,
            Type::TIMESTAMPTZ,
            Type::MACADDR,
            Type::INT4,
            Type::VARCHAR,
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
                    &user.work_phone,
                    &user.mobile_phone,
                    &user.id,
                ])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    match row {
                        Some(r) => (client, Some(row_to_user(&r))),
                        _ => (client, None),
                    }
                })
        })
}

pub fn update_user_coords_by_mac(mut client: tokio_postgres::Client, mac: MacAddress, coords: na::Vector2<f64>) -> impl Future<Item=(tokio_postgres::Client, Option<TrackedUser>), Error=tokio_postgres::Error> {
    client
        .prepare_typed("
            UPDATE runtime.users
            SET
                u_coordinates = $1
             WHERE
                u_mac_address = $2
            RETURNING *
        ", &[
            Type::FLOAT8_ARRAY,
            Type::MACADDR,
        ])
        .and_then(move |statement| {
            let coordinates = vec![coords[0], coords[1]];
            client
                .query(&statement, &[
                    &coordinates,
                    &mac,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_utils;
    use tokio::runtime::current_thread::Runtime;

    #[test]
    fn insert() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db()).unwrap();

        let user = TrackedUser::new();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .map(|(_client, _opt_user)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert user");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn update() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db()).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();
        let mut updated_user = user.clone();
        updated_user.name = "user_1".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                updated_user.id = opt_user.unwrap().id;
                update_user(client, updated_user)
            })
            .map(|(_client, _user)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert user");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn update_coord_by_mac() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db()).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();
        user.mac_address = MacAddress::from_bytes(&[0, 0, 3, 0, 0, 0]).unwrap();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                let mac = opt_user.unwrap().mac_address;
                update_user_coords_by_mac(client, mac, na::Vector2::<f64>::new(5.0, 5.0))
            })
            .map(|(_client, _user)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db()).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                select_user(client, opt_user.unwrap().id)
            })
            .map(|(_client, _opt_user)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to select user");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_random() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db()).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, _opt_user)| {
                select_user_random(client)
            })
            .map(|(_client, opt_user)| {
                assert!(opt_user.is_some());
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to insert beacon");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn select_many() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db()).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, _opt_user)| {
                select_users(client)
            })
            .map(|(_client, _users)| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to select multiple users");
            });
        runtime.block_on(task).unwrap();
    }

    #[test]
    fn delete() {
        let mut runtime = Runtime::new().unwrap();
        runtime.block_on(crate::system::create_db()).unwrap();

        let mut user = TrackedUser::new();
        user.name = "user_0".to_string();

        let task = db_utils::default_connect()
            .and_then(|client| {
                insert_user(client, user)
            })
            .and_then(|(client, opt_user)| {
                delete_user(client, opt_user.unwrap().id)
            })
            .map(|_client| {
            })
            .map_err(|e| {
                println!("db error {:?}", e);
                panic!("failed to delete user");
            });
        runtime.block_on(task).unwrap();
    }
}



