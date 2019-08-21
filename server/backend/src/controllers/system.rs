use tokio_postgres::{ NoTls, error::SqlState, };
use futures::{ Future, future::err, future::Loop, future::ok, future::Either, future::loop_fn, };

fn connect_db(params: &str) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    tokio_postgres::connect(params, NoTls)
        .map(|(client, connection)| {
            let connection = connection.map_err(|e| eprintln!("connect db error: {}", e));
            tokio::spawn(connection);
            client
        })
}

// dont bother undoing table creations, the entire ak database is dropped and recreated.
const UNDO_SCHEMA: [&str; 4] = [
    "DROP USER responder",
    "DROP USER admin",
    "DROP ROLE ak_responder_role",
    "DROP ROLE ak_admin_role",
];

const SCHEMA: [&str; 7] = [
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
        phone_number VARCHAR(20),
        type INTEGER
    );",
    "CREATE TABLE beacons (
        mac_address MACADDR PRIMARY KEY,
        coordinates real[2],
        map_id VARCHAR(255) REFERENCES maps(floor_id),
        name VARCHAR(255)
    );",
    "CREATE ROLE ak_admin_role WITH LOGIN",
    "CREATE ROLE ak_responder_role WITH LOGIN",
    "CREATE USER admin WITH PASSWORD 'admin' SYSID 1 ROLE ak_admin_role",
    "CREATE USER responder WITH PASSWORD NULL SYSID 2 ROLE ak_responder_role",
];

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

fn loop_db_commands(client: tokio_postgres::Client, commands: Vec<&str>, chug_along: bool) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> + '_ {
    loop_fn((client, commands.into_iter()), move |(mut client, mut schema_it)| {
        let chug = chug_along;
        let it = Box::new(schema_it.next());
        ok::<_, tokio_postgres::Error>(it)
            .and_then(move |it| {
                match *it {
                    Some(command) => {
                        println!("executing command {}", command);
                        Either::A(client.prepare(command)
                            .map(|statement| (client, statement))
                            .and_then(move |(mut client, statement)| {
                                client.execute(&statement, &[])
                                    .then(move |res| {
                                        match res {
                                            Ok(_) => {
                                                ok(())
                                            },
                                            Err(e) => {
                                                if chug {
                                                    ok(())
                                                } else {
                                                    err(e)
                                                }
                                            },
                                        }
                                    })
                                    .map(|_| {
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
    .map(|x: (tokio_postgres::Client, Option<&str>)| {
        x.0
    })
}

pub fn create_db() {
    println!("creating db");
    let fut = ensure_ak()
        .and_then(|_| {
            connect_db("dbname=ak host=localhost password=postgres user=postgres")
        })
        .and_then(|client| {
            loop_db_commands(client, UNDO_SCHEMA.to_vec(), true)
        })
        .and_then(|client| {
            loop_db_commands(client, SCHEMA.to_vec(), false)
        })
        .map(|_| {
        })
        .map_err(|e| {
            eprintln!("db error: {}", e);
        });

        actix::spawn(fut);
}
