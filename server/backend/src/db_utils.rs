use futures::Future;
use tokio_postgres::NoTls;

pub const DEFAULT_CONNECTION: &str = "dbname=ak host=localhost password=postgres user=postgres";

pub fn default_connect() -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    connect(DEFAULT_CONNECTION)
}

pub fn connect(params: &str) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    tokio_postgres::connect(params, NoTls)
        .map(|(client, connection)| {
            let connection = connection.map_err(|e| eprintln!("connect db error: {}", e));
            tokio::spawn(connection);
            client
        })
}

