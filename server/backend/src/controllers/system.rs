use tokio_postgres::{ NoTls, error::SqlState, };
use futures::{ Future, future::ok, future::Either, };

fn connect_db(params: &str) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    tokio_postgres::connect(params, NoTls)
        .map(|(client, connection)| {
            let connection = connection.map_err(|e| eprintln!("db connection error: {}", e));
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

pub fn create_db() {
    println!("creating db");
    let fut = ensure_ak()
        .and_then(|_| {
            tokio_postgres::connect("dbname=ak host=localhost password=postgres user=postgres", NoTls)
        })
        .map(|(client, connection)| {
            let connection = connection.map_err(|e| eprintln!("db connection error: {}", e));
            actix::spawn(connection);
            client
        })
        .and_then(|mut _client| {
            let _schema = vec![
                "CREATE TABLE maps (
                    floor_id varchar(255) PRIMARY KEY,
                    name varchar(255),
                    blueprint bytea
                );",
                "CREATE TABLE users (
                    name varchar(255),
                    mac_address macaddr
                );",
                " CREATE TABLE beacons (
                    mac_address macaddr PRIMARY KEY,
                    name varchar(255),
                    map_id INTEGER REFERENCES maps(id)
                );",
            ];

           /* loop_fn((client, schema.iter()), |(mut client, schema_it)| {
                ok(())
                    .and_then(|_| {
                        match schema_it.next() {
                            Some(command) => {
                                println!("looping {}", command);
                                Ok(Loop::Continue((client, schema_it)))
                            },
                            None => {
                                println!("stop looping");
                                Ok(Loop::Break((client, None)))
                            }
                        }
                    })
                /*match schema_it {
                    Some(command) => client.prepare(command)
                        .map(|statement| (client, statement))
                        .and_then(|(client, statement)| {
                            client.execute(&statement, &[])
                                .map(|row_count| {
                                    assert_eq!(row_count, 0);
                                    client
                                })
                        })
                        .and_then(|client| {
                            Loop::Continue((client, schema_it.next())
                        }),
                    None => Either::B(Loop::Break((client, None))),
                }*/
            })
            .map(|x: (tokio_postgres::Client, std::slice::Iter<_>)| {
            })

            */
            /*ok(client
                .simple_query("
                    CREATE TABLE maps (
                        floor_id varchar(255) PRIMARY KEY,
                        name varchar(255),
                        blueprint bytea
                    );
                    CREATE TABLE users (
                        name varchar(255),
                        mac_address macaddr
                    );
                    CREATE TABLE beacons (
                        mac_address macaddr PRIMARY KEY,
                        name varchar(255),
                        map_id INTEGER REFERENCES maps(id)
                    );
                ")
                .map(|stream| {
                    println!("hello hello");
                    match stream {
                        tokio_postgres::SimpleQueryMessage::CommandComplete(b) => { println!("stream is {:?}", b); },
                        _ => {},
                    }
                })
                .map_err(|e| {
                    eprintln!("db error: {}", e);
                })
            )*/

                //ok(client)
            ok(())
        })
        .map(|_hello| {
            //let () = hello;
            //println!("hello {:?}", hello);
        })
        .map_err(|e| {
            eprintln!("db error: {}", e);
        });

        actix::spawn(fut);
}
