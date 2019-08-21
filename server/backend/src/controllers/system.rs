use tokio_postgres::{ NoTls, error::SqlState, };
use futures::{ Future, future::Loop, future::ok, future::Either, future::loop_fn, };

fn connect_db(params: &str) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    tokio_postgres::connect(params, NoTls)
        .map(|(client, connection)| {
            let connection = connection.map_err(|e| eprintln!("connect db error: {}", e));
            tokio::spawn(connection);
            client
        })
}

fn ensure_ak() -> impl Future<Item=(), Error=tokio_postgres::Error> {
    tokio_postgres::connect("dbname=ak host=localhost password=postgres user=postgres", NoTls)
        .then(|res| {
            // make a connection to the default database to create the ak db
            let connect = connect_db("host=localhost password=postgres user=postgres");

            // check if the ak connection was successful, and delete ak if necessary
            match &res {
                Err(e) if e.code() == Some(&SqlState::INVALID_CATALOG_NAME) => {
                    Either::B(connect)
                },
                Ok(_) | Err(_) => {
                    // successfully connected with ak, we need to drop it before continuing
                    let prepared_drop = connect
                        .and_then(|mut client| {
                            client.prepare("DROP DATABASE ak")
                                .map(|statement| (client, statement))
                        })
                        .and_then(|(mut client, statement)| {
                            client.execute(&statement, &[])
                                .map(|row_count| {
                                    assert_eq!(row_count, 0);
                                    client
                                })
                        });
                    Either::A(prepared_drop)
                },
            }
        })
        // at this point, the ak database should not exist. now we can create it fresh
        .and_then(|mut client| {
            client.prepare("CREATE DATABASE ak")
                .map(|statement| (client, statement))
        })
        .and_then(|(mut client, statement)| {
            client.execute(&statement, &[])
                .map(|row_count| {
                    assert_eq!(row_count, 0);
                    client
                })
        })
        .map(|_client| {
            // map to no return type
        })
}

const SCHEMA: [&str; 3] = [
    "CREATE TABLE maps (
        floor_id VARCHAR(255) PRIMARY KEY,
        blueprint BYTEA,
        name TEXT
    );",
    "CREATE TABLE users (
        id INTEGER PRIMARY KEY,
        coordinates real[2],
        emergency_contact INTEGER REFERENCES users(id),
        employee_id VARCHAR(255),
        id_tag INTEGER,
        last_seen timestamp,
        mac_address MACADDR,
        map_id VARCHAR(255) REFERENCES maps(floor_id),
        name VARCHAR(255),
        note TEXT,
        phone_number VARCHAR(20)
    );",
    " CREATE TABLE beacons (
        mac_address MACADDR PRIMARY KEY,
        coordinates real[2],
        map_id VARCHAR(255) REFERENCES maps(floor_id),
        name VARCHAR(255)
    );",
];

pub fn create_db() {
    println!("creating db");
    let fut = ensure_ak()
        .and_then(|_| {
            connect_db("dbname=ak host=localhost password=postgres user=postgres")
        })
        .and_then(|client| {
           loop_fn((client, SCHEMA.iter()), |(mut client, mut schema_it)| {
                let it = Box::new(schema_it.next());
                ok::<_, tokio_postgres::Error>(it)
                    .and_then(|it| {
                        match *it {
                            Some(command) => {
                                Either::A(client.prepare(command)
                                    .map(|statement| (client, statement))
                                    .and_then(|(mut client, statement)| {
                                        client.execute(&statement, &[])
                                            .map(|row_count| {
                                                assert_eq!(row_count, 0);
                                                (it, client)
                                            })
                                    }))
                            },
                            None => {
                                Either::B(ok::<_, tokio_postgres::Error>((it, client)))
                            }
                        }
                    })
                    .and_then(|(it, client)| {
                        match *it {
                            Some(_) => {
                                Ok(Loop::Continue((client, schema_it)))
                            },
                            None => {
                                Ok(Loop::Break((client, None)))
                            }
                        }
                    })
            })
            .map(|_x: (tokio_postgres::Client, Option<&str>)| {
            })
        })
        .map(|_x| {
        })
        .map_err(|e| {
            eprintln!("db error: {}", e);
        });

        actix::spawn(fut);
}
