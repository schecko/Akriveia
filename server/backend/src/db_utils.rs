use futures::Future;
// tokio_postgres is the driver to connect to PostGreSQL
// What is the difference then?
// What is a postgresql client?
use tokio_postgres::NoTls;

pub const DEFAULT_CONNECTION: &str = "dbname=ak host=localhost password=postgres user=postgres";

pub fn default_connect() -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    connect(DEFAULT_CONNECTION)
}

//connecting using a new client
pub fn connect(params: &str) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> {
    tokio_postgres::connect(params, NoTls)
        .map(|(client, connection)| {
            let connection = connection.map_err(|e| eprintln!("connect db error: {}", e));
            tokio::spawn(connection);
            client
        })
}
// What is the difference between .map_err vs .unwrap_err ?

