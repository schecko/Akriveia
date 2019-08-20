use tokio_postgres::{ NoTls, error::SqlState, };
use futures::{ Future, future::Either };

pub fn create_db() {
    println!("creating db");
    let fut = tokio_postgres::connect("dbname=ak host=localhost password=postgres user=postgres", NoTls)
        .then(|res| {
            // make a connection to the default database to create the ak db
            let connect = tokio_postgres::connect("host=localhost password=postgres user=postgres", NoTls)
                .map(|(client, connection)| {
                    let connection = connection.map_err(|e| eprintln!("db connection error: {}", e));
                    tokio::spawn(connection);
                    client
                });

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
        .and_then(|_| {
            tokio_postgres::connect("dbname=ak host=localhost password=postgres user=postgres", NoTls)
        })
        .map(|(client, connection)| {
            let connection = connection.map_err(|e| eprintln!("db connection error: {}", e));
            actix::spawn(connection);
            client
        })
        .and_then(|mut client| {
            client
                .prepare("
                    CREATE TABLE beacons (
                        mac_address macaddr PRIMARY KEY
                    )
                ")
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
        .map_err(|e| {
            eprintln!("db error: {}", e);
        });

        actix::spawn(fut);
}
