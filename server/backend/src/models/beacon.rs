
use futures::{ Stream, Future, };
use common::*;
use tokio_postgres::row::Row;
use na;

fn row_to_beacon(row: Option<Row>) -> Option<Beacon> {
    match row {
        Some(data) => {
            let coordinates: Vec<f64> = data.get(1);
            for column in data.columns().iter() {
                println!("col {:?}", column);
            }
            Some(Beacon {
                mac_address: data.get(0),
                coordinates: na::Vector2::new(coordinates[0], coordinates[1]),
                map_id: data.get(2),
                name: data.get(3),
                note: data.get(4),
            })
        },
        None => None,
    }
}

pub fn get_beacon(mut client: tokio_postgres::Client, mac_address: String) -> impl Future<Item=(tokio_postgres::Client, Option<Beacon>), Error=tokio_postgres::Error> {
    client
        .prepare("
            SELECT FROM runtime.beacons
            WHERE mac_address = $1::MACADDR
        ")
        .and_then(move |statement| {
            client
                .query(&statement, &[&mac_address])
                .into_future()
                .map_err(|err| {
                    err.0
                })
                .map(|(row, _next)| {
                    (client, row_to_beacon(row))
                })
        })
}
