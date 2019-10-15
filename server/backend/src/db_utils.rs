use futures::Future;
use tokio_postgres::NoTls;
use common::LoginInfo;

pub const DEFAULT_CONNECTION: &str = "dbname=ak host=localhost password=postgres user=postgres";

pub fn default_connect() -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    connect(DEFAULT_CONNECTION)
}

pub fn connect_login(user: &LoginInfo) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    connect(&format!("dbname=ak host=localhost user={} password={}", user.name, user.pw))
}

pub fn connect(params: &str) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    tokio_postgres::connect(params, NoTls)
        .map(|(client, connection)| {
            let connection = connection.map_err(|e| eprintln!("connect db error: {}", e));
            tokio::spawn(connection);
            client
        })
}

