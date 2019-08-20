use tokio_postgres::{NoTls};
use futures::{ Future, };

pub fn create_db() {
    /*let fut = tokio_postgres::connect("dname=ak port=5432 host=localhost password=postgres user=postgres", NoTls)
        .and_then(|_| {
            // successfully connected with ak, we need to drop this connection, connect with the
            // default db and then drop ak
            */
            let fut = tokio_postgres::connect("port=5432 host=localhost password=postgres user=postgres", NoTls)
                .map(|(client, connection)| {
                    let connection = connection.map_err(|e| eprintln!("db connection error: {}", e));
                    tokio::spawn(connection);
                    client
                })
                .and_then(|mut client| {
                    client.prepare("DROP DATABASE ak")
                        .map(|statement| (client, statement))
                })
                .and_then(|(mut client, statement)| {
                    client.execute(&statement, &[])
                        .map(|row_count| {
                            println!("successfully dropped old ak db");
                            assert_eq!(row_count, 0);
                            client
                        })
                })
                .and_then(|mut client| {
                    client.prepare("CREATE DATABASE ak")
                        .map(|statement| (client, statement))
                })
                .and_then(|(mut client, statement)| {
                    client.execute(&statement, &[])
                        .map(|row_count| {
                            println!("successfully dropped old ak db");
                            assert_eq!(row_count, 0);
                            client
                        })
                })
                .map(|_client| {
                })
                .map_err(|e| {
                    eprintln!("db error: {}", e);
                });
                //.map_err(|e| {
                //    eprintln!("db error: {}", e);
                //});
                //.map(|_| ok(()))
                /*
        })
        .map_err(|e| {
            // its fine if the db doesnt exist, other errors are bad
            match e.code() {
                code => println!("code is {:?}", code),
            }
            eprintln!("db error: {}", e);
        })
        //.map(|_| ok(()))
        .and_then(|_| {
            tokio_postgres::connect("dbname=ak port=5432 host=localhost password=postgres user=postgres", NoTls)
        })
        .map(|(client, connection)| {
            let connection = connection.map_err(|e| eprintln!("db connection error: {}", e));
            actix::spawn(connection);
            client
        })
        /*.and_then(|mut client| {
            client.execute("CREATE DATABASE ak", &[])
        })
        .map(|rows| {
            println!("successfully created ak db");
            assert_eq!(rows.len(), 0);
        })*/
        .map_err(|e| {
            eprintln!("db error: {}", e);
        });*/

        // By default, tokio_postgres uses the tokio crate as its runtime.
        actix::spawn(fut);
}
