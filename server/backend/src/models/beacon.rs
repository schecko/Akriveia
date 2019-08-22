
use futures::{ Stream, Future, };
use common::*;
use na;

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
                .map(|row| {
                    match row.0 {
                        Some(data) => {
                            assert_eq!(data.len(), 1);
                            let coordinates: Vec<f64> = data.get(1);
                            (client, Some(Beacon {
                                mac_address: data.get(0),
                                coordinates: na::Vector2::new(coordinates[0], coordinates[1]),
                                map_id: data.get(2),
                                name: data.get(3),
                                note: data.get(4),
                            }))
                        },
                        None => (client, None),
                    }
                })
        })
}
