extern crate actix;
extern crate tokio;
extern crate futures;
extern crate bytes;

use bytes::{ BytesMut, Bytes };
use std::io;
use std::net::SocketAddr;
use futures::{ stream, Stream, Sink, future::ok, };
use futures::stream::SplitSink;
use tokio::net::{ UdpSocket, UdpFramed };
use tokio::codec::BytesCodec;
use actix::prelude::*;
use actix::{ Actor, Context, StreamHandler, fut::Either, };
use actix::io::{ WriteHandler, SinkWrite, };
use crate::beacon_manager::*;

pub struct BeaconUDP {
    sink: SinkWrite<SplitSink<UdpFramed<BytesCodec>>>,
    beacon_ips: Vec<SocketAddr>,
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
            "ack" => { println!("beacon {} ack'd", msg.addr); }
            other => {
                println!("Received: ({:?}, {:?})", other, msg.addr);
                // process data or error
            }
        }
    }
}

impl Handler<BeaconCommand> for BeaconUDP {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: BeaconCommand, context: &mut Context<Self>) -> Self::Result {

        let broad: SocketAddr = "127.0.0.255:8082".parse().unwrap();
        match msg {
            BeaconCommand::StartEmergency => {
                self.sink.write((Bytes::from("start"), broad));
            },
            BeaconCommand::EndEmergency => {
                self.sink.write((Bytes::from("end"), broad));
            },
            _ => {
            }
        }

        Ok(())
    }
}

impl BeaconUDP {
    pub fn new(addr: SocketAddr) -> Addr<BeaconUDP> {
        let sock = UdpSocket::bind(&addr).unwrap();
        sock.set_broadcast(true).expect("could not set broadcast");

        let (sink, stream) = UdpFramed::new(sock, BytesCodec::new()).split();
        BeaconUDP::create(|context| {
            context.add_stream(stream.map(|(data, sender)| Frame { data, addr: sender }));
            let sw = SinkWrite::new(sink, context);
            //tokio::spawn(fut);
            BeaconUDP {
                sink: sw,
                beacon_ips: Vec::new(),
            }
        })
    }
}
