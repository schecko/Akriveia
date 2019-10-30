// Unfortunately, due to using UDP sockets and broadcasts, it is difficult to associate a single
// message from the manager with a set of responses to the j
extern crate actix;
extern crate tokio;
extern crate futures;
extern crate bytes;

use actix::io::{ WriteHandler, SinkWrite, };
use actix::prelude::*;
use actix::{ Actor, Context, StreamHandler, };
use bytes::{ BytesMut, Bytes };
use crate::beacon_manager::*;
use crate::conn_common;
use futures::stream::SplitSink;
use futures::{ Stream, };
use ipnet::Ipv4Net;
use std::io;
use std::net::{ IpAddr, };
use std::net::SocketAddr;
use tokio::codec::BytesCodec;
use tokio::net::{ UdpSocket, UdpFramed };
use common::*;

pub struct BeaconUDP {
    bound_ip: Ipv4Net,
    bound_port: u16,
    manager: Addr<BeaconManager>,
    sink: SinkWrite<SplitSink<UdpFramed<BytesCodec>>>,
}

impl WriteHandler<io::Error> for BeaconUDP {
    fn error(&mut self, err: io::Error, _context: &mut Self::Context) -> Running {
        println!("beacon udp encountered an error {}", err);
        Running::Stop
    }

    fn finished(&mut self, _context: &mut Self::Context) {
        // override the finish method of the trait, because the default will stop the actor...
        println!("finish sending data");
    }
}

impl Actor for BeaconUDP {
    type Context = Context<Self>;
}

#[derive(Message)]
struct Frame {
    data: BytesMut,
    addr: SocketAddr
}

impl StreamHandler<Frame, io::Error> for BeaconUDP {
    fn handle(&mut self, msg: Frame, _: &mut Context<Self>) {
        match String::from_utf8_lossy(&msg.data).into_owned().as_str() {
            // TODO update this with new format
            "start_ack" => {
                self.manager
                    .do_send(BMResponse::Start(msg.addr.ip(), MacAddress8::nil()));
            }
            "end_ack" => {
                self.manager
                    .do_send(BMResponse::End(msg.addr.ip(), MacAddress8::nil()));
            }
            "ping_ack" => {
                self.manager
                    .do_send(BMResponse::Ping(msg.addr.ip(), MacAddress8::nil()));
            }
            other => {
                // process data or error
                match conn_common::parse_message(other) {
                    Ok(msg) => {
                        self.manager
                            .do_send(TagDataMessage { data: msg });
                    },
                    Err(e) => {
                        println!("failed to parse message from udp beacon: {}", e);
                    }
                }
                println!("Received: ({:?}, {:?})", other, msg.addr);
            }
        }
    }
}

impl Handler<BeaconCommand> for BeaconUDP {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: BeaconCommand, _context: &mut Context<Self>) -> Self::Result {

        let broadcast = SocketAddr::new(IpAddr::V4(self.bound_ip.broadcast()), self.bound_port);
        let (command, ip) = match msg {
            BeaconCommand::StartEmergency => ("start", broadcast),
            BeaconCommand::EndEmergency => ("end", broadcast),
            BeaconCommand::Ping(opt_ip) => {
                if let Some(ip) = opt_ip {
                    let addr = SocketAddr::new(ip.into(), self.bound_port);
                    ("ping", addr)
                } else {
                    ("ping", broadcast)
                }
            },
            BeaconCommand::Reboot(opt_ip) => {
                if let Some(ip) = opt_ip {
                    let addr = SocketAddr::new(ip.into(), self.bound_port);
                    ("reboot", addr)
                } else {
                    ("reboot", broadcast)
                }
            },
        };

        self.sink
            .write((Bytes::from(command), ip))
            .map(|_s| {})
            .map_err(|_e| {})
    }
}

impl BeaconUDP {
    pub fn new(manager: Addr<BeaconManager>, ip: Ipv4Net, port: u16) -> Addr<BeaconUDP> {
        let bind_addr = SocketAddr::new(IpAddr::V4(ip.addr()), port);

        let sock = UdpSocket::bind(&bind_addr).unwrap();
        sock.set_broadcast(true).expect("could not set broadcast");

        let (sink, stream) = UdpFramed::new(sock, BytesCodec::new()).split();
        BeaconUDP::create(move |context| {
            context.add_stream(stream.map(|(data, sender)| Frame { data, addr: sender }));
            let sw = SinkWrite::new(sink, context);
            BeaconUDP {
                bound_ip: ip,
                bound_port: port,
                manager,
                sink: sw,
            }
        })
    }
}
