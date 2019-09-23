extern crate actix;
extern crate tokio;
extern crate futures;
extern crate bytes;

use bytes::{ BytesMut, Bytes };
use std::io;
use std::net::SocketAddr;
use futures::{ stream, Stream, Sink, };
use futures::stream::SplitSink;
use tokio::net::{ UdpSocket, UdpFramed };
use tokio::codec::BytesCodec;
use actix::prelude::*;
use actix::{ Actor, Context, StreamHandler, };
use actix::io::WriteHandler;
use actix::io::SinkWrite;
use crate::beacon_manager::*;

pub struct BeaconUDP {
    //sink: SplitSink<UdpFramed<BytesCodec>>,
    sw: SinkWrite<SplitSink<UdpFramed<BytesCodec>>>,
    beacon_ips: Vec<SocketAddr>,
}

impl Actor for BeaconUDP {
    type Context = Context<Self>;
}

#[derive(Message)]
struct Frame {
    data: BytesMut,
    addr: SocketAddr
}

impl WriteHandler<io::Error> for BeaconUDP {
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
    type Result = Result<common::SystemCommandResponse, io::Error>;

    fn handle(&mut self, msg: BeaconCommand, _: &mut Context<Self>) -> Self::Result {

        match msg {
            BeaconCommand::StartEmergency => {
                //let _ = (&mut self.sink).send((Bytes::from("start"), "127.0.0.255:8082".parse().unwrap()));
                Ok(common::SystemCommandResponse{ emergency: true })
            },
            BeaconCommand::EndEmergency => {
                //let _ = (&mut self.sink).send((Bytes::from("end"), "127.0.0.255:8082".parse().unwrap()));
                Ok(common::SystemCommandResponse{ emergency: false })
            },
            _ => {
                Ok(common::SystemCommandResponse{ emergency: true })
            }
        }
    }
}

impl BeaconUDP {
    pub fn new(addr: SocketAddr) -> Addr<BeaconUDP> {
        let sock = UdpSocket::bind(&addr).unwrap();
        sock.set_broadcast(true).expect("could not set broadcast");

        let (sink, stream) = UdpFramed::new(sock, BytesCodec::new()).split();
        BeaconUDP::create(|context| {
            context.add_stream(stream.map(|(data, sender)| Frame { data, addr: sender }));
            let mut sw = SinkWrite::new(sink, context);
            let broad: SocketAddr = "255.255.255.255:0".parse().unwrap();
            sw.write((Bytes::from("end"), broad));
            //let fut = (self.sink).send((Bytes::from("end"), broad)).map(|_x| {}).map_err(|_e| {});
            //tokio::spawn(fut);
            BeaconUDP {
                sw,
                beacon_ips: Vec::new(),
            }
        })
    }
}
