extern crate actix;
extern crate tokio;
extern crate futures;
extern crate bytes;

use bytes::BytesMut;
use std::io;
use std::net::SocketAddr;
use futures::{ Stream, Sink, Future };
use futures::stream::SplitSink;
use tokio::net::{ UdpSocket, UdpFramed };
use tokio::codec::BytesCodec;
use actix::prelude::*;
use actix::{ Actor, Context, StreamHandler };


pub struct BeaconUDP {
    sink: SplitSink<UdpFramed<BytesCodec>>
}

impl Actor for BeaconUDP {
    type Context = Context<Self>;
}

#[derive(Message)]
struct Packet {
    data: BytesMut,
    addr: SocketAddr
}

impl StreamHandler<Packet, io::Error> for BeaconUDP {
    fn handle(&mut self, msg: Packet, _: &mut Context<Self>) {
        println!("Received: ({:?}, {:?})", msg.data, msg.addr);
        (&mut self.sink).send(("PING\n".into(), msg.addr)).wait().unwrap();
    }
}

impl BeaconUDP {
    pub fn new(addr: SocketAddr) -> Addr<BeaconUDP> {
        let sock = UdpSocket::bind(&addr).unwrap();
        println!("{:?}", sock);
        let (sink, stream) = UdpFramed::new(sock, BytesCodec::new()).split();
        BeaconUDP::create(|context| {
            context.add_stream(stream.map(|(data, sender)| Packet{ data, addr: sender }));
            BeaconUDP { sink }
        })
    }
}
