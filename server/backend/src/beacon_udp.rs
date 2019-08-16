extern crate actix;
extern crate tokio;
extern crate futures;
extern crate bytes;

use bytes::{ BytesMut, Bytes };
use std::io;
use std::net::SocketAddr;
use futures::{ stream, Stream, Sink, Future, future::ok };
use futures::stream::SplitSink;
use tokio::net::{ UdpSocket, UdpFramed };
use tokio::codec::BytesCodec;
use actix::prelude::*;
use actix::{ Actor, Context, StreamHandler };
use crate::beacon_manager::*;


pub struct BeaconUDP {
    sink: SplitSink<UdpFramed<BytesCodec>>,
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
                let stream_data = self.beacon_ips.iter()
                    .map(|&ip| (Bytes::from("start"), ip));
                let stream = stream::iter_ok::<_, io::Error>(stream_data);
                (&mut self.sink).send_all(stream);
            },
            BeaconCommand::EndEmergency => {
                let stream_data = self.beacon_ips.iter()
                    .map(|&ip| (Bytes::from("end"), ip));
                let stream = stream::iter_ok::<_, io::Error>(stream_data);
                (&mut self.sink).send_all(stream);
            },
            _ => {
            }
        }
        Ok(common::SystemCommandResponse{ emergency: true })


    }
}

impl BeaconUDP {
    pub fn new(addr: SocketAddr) -> Addr<BeaconUDP> {
        let sock = UdpSocket::bind(&addr).unwrap();
        println!("{:?}", sock);
        let (sink, stream) = UdpFramed::new(sock, BytesCodec::new()).split();
        BeaconUDP::create(|context| {
            context.add_stream(stream.map(|(data, sender)| Frame { data, addr: sender }));
            BeaconUDP {
                sink,
                beacon_ips: Vec::new(),
            }
        })
    }
}
