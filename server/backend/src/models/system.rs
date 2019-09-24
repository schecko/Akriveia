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
// NOTE: this should be in the reverse order of the schema
const UNDO_SCHEMA: [&str; 4] = [
    "DROP USER responder",
    "DROP USER admin",
    "DROP ROLE ak_responder_role",
    "DROP ROLE ak_admin_role",
];

const SCHEMA: [&str; 27] = [
    "CREATE SCHEMA runtime",
    "CREATE SCHEMA system",
    "CREATE TABLE runtime.maps (
        m_id SERIAL PRIMARY KEY,
        m_blueprint BYTEA,
        m_bounds INTEGER[2] NOT NULL CHECK (0 <= all(m_bounds)),
        m_name VARCHAR(256) UNIQUE,
        m_scale DOUBLE PRECISION,
        m_note VARCHAR(1024)
    );",
    "CREATE TABLE runtime.users (
        u_id SERIAL PRIMARY KEY,
        u_coordinates DOUBLE PRECISION[2],
        u_emergency_contact INTEGER REFERENCES runtime.users(u_id),
        u_employee_id VARCHAR(256),
        u_last_active TIMESTAMPTZ NOT NULL,
        u_mac_address MACADDR,
        u_map_id INTEGER REFERENCES runtime.maps(m_id),
        u_name VARCHAR(256) UNIQUE,
        u_note VARCHAR(1024),
        u_work_phone VARCHAR(20),
        u_mobile_phone VARCHAR(20)
    );",
    "CREATE TABLE runtime.beacons (
        b_id SERIAL PRIMARY KEY,
        b_mac_address MACADDR UNIQUE,
        b_ip INET UNIQUE,
        b_coordinates DOUBLE PRECISION[2] NOT NULL,
        b_map_id INTEGER REFERENCES runtime.maps(m_id) NOT NULL,
        b_name VARCHAR(255) UNIQUE,
        b_note VARCHAR(1024)
    );",
    "CREATE TABLE system.networks (
        n_id SERIAL PRIMARY KEY,
        n_mac_address MACADDR,
        n_host_beacon_udp BOOLEAN NOT NULL,
        n_host_webserver BOOLEAN NOT NULL,
        n_ip INET,
        n_name VARCHAR(255) UNIQUE
    )",

    // create roles and users
    "CREATE ROLE ak_admin_role",
    "CREATE ROLE ak_responder_role",
    "CREATE USER admin WITH PASSWORD 'admin' SYSID 1 ROLE ak_admin_role",
    "CREATE USER responder WITH PASSWORD NULL SYSID 2 ROLE ak_responder_role",

    // update permissions for responders
    "GRANT CONNECT ON DATABASE ak TO ak_responder_role",
    "GRANT SELECT ON ALL TABLES IN SCHEMA runtime TO ak_responder_role",

    // update permissions for admins
    "GRANT CONNECT ON DATABASE ak TO ak_admin_role",
    "GRANT SELECT ON ALL TABLES IN SCHEMA runtime TO ak_admin_role",
    "GRANT UPDATE ON ALL TABLES IN SCHEMA runtime TO ak_admin_role",
    "GRANT INSERT ON ALL TABLES IN SCHEMA runtime TO ak_admin_role",
    "GRANT DELETE ON ALL TABLES IN SCHEMA runtime TO ak_admin_role",
    "GRANT SELECT ON ALL TABLES IN SCHEMA system TO ak_admin_role",
    "GRANT UPDATE ON ALL TABLES IN SCHEMA system TO ak_admin_role",
    "GRANT INSERT ON ALL TABLES IN SCHEMA system TO ak_admin_role",
    "GRANT DELETE ON ALL TABLES IN SCHEMA system TO ak_admin_role",
    "INSERT INTO system.networks(n_mac_address, n_host_beacon_udp, n_host_webserver, n_ip, n_name)
            VALUES('00:00:00:00:00:00', TRUE, TRUE, '127.0.0.1', 'localhost')
    ",
    // TODO remove after implementing frontend
    "INSERT INTO runtime.users(u_name, u_last_active, u_coordinates, u_mac_address)
            VALUES('test_user', 'epoch', ARRAY [ 0, 0 ], '80:00:00:00:00:00')
    ",
    "INSERT INTO runtime.maps(m_id, m_bounds, m_name, m_scale)
            VALUES(69, ARRAY [ 400, 400 ], 'test_map', 100)
    ",
    "INSERT INTO runtime.beacons(b_id, b_mac_address, b_ip, b_coordinates, b_map_id, b_name)
            VALUES(100, '00:00:00:00:00:01', '0.0.0.1', ARRAY [ 0, 0 ], 69, 'origin_beacon')
    ",
    "INSERT INTO runtime.beacons(b_id, b_mac_address, b_ip, b_coordinates, b_map_id, b_name)
            VALUES(130, '00:00:00:00:00:02', '0.0.0.2', ARRAY [ 3, 0 ], 69, 'top_left')
    ",
    "INSERT INTO runtime.beacons(b_id, b_mac_address, b_ip, b_coordinates, b_map_id, b_name)
            VALUES(103, '00:00:00:00:00:03', '0.0.0.3', ARRAY [ 0, 3 ], 69, 'bottom_right')
    ",
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

fn loop_db_commands(client: tokio_postgres::Client, commands: Vec<&str>, ignore_errors: bool) -> impl Future<Item=tokio_postgres::Client, Error=tokio_postgres::Error> + '_ {
    loop_fn((client, commands.into_iter()), move |(mut client, mut schema_it)| {
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
                                                if ignore_errors {
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

pub fn create_db() -> impl Future<Item=(), Error=()> {
    println!("creating db");
    ensure_ak()
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
            println!("successfully recreated ak database");
        })
        .map_err(|e| {
            eprintln!("db error: {}", e);
        })
}
