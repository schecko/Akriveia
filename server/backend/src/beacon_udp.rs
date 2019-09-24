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
use actix::io::WriteHandler;
use crate::beacon_manager::*;

pub struct BeaconUDP {
    sink: Option<SplitSink<UdpFramed<BytesCodec>>>,
    sink_buffer: Vec<(Bytes, SocketAddr)>,
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

enum InternalCommand {
    Flush,
}
impl Message for InternalCommand {
    type Result = Result<(), ()>;
}

impl Handler<InternalCommand> for BeaconUDP {
    type Result = ResponseActFuture<Self, (), ()>;

    fn handle(&mut self, msg: InternalCommand, _: &mut Context<Self>) -> Self::Result {
        match msg {
            InternalCommand::Flush => {
                println!("fuck 3fuck");
                let fut = if let Some(sink) = self.sink.take() {
                    println!("fuck 4fuck");
                    let drain: Vec<(Bytes, SocketAddr)> = self.sink_buffer.drain(..).collect();
                    let commands = stream::iter_ok::<_, io::Error>(drain);

                    Either::A(sink
                        .send_all(commands)
                        .into_actor(self)
                        .and_then(|(sink, source), actor, context| {
                            println!("wtf is this {:?}", source);
                            actor.sink = Some(sink);
                            if(actor.sink_buffer.len() > 0) {
                                context.notify(InternalCommand::Flush);
                            }
                            actix::fut::ok(())
                        })
                        .map_err(|e, _actor, _context| {
                        })
                    )
                } else {
                    println!("fuck 5fuck");
                   Either::B(actix::fut::ok(()))
                };
                Box::new(fut)
            },
        }
    }
}

pub enum UdpCommand {
    EndEmergency,
    GetEmergency,
    ScanBeacons,
    StartEmergency,
}
impl Message for UdpCommand {
    type Result = Result<(), ()>;
}
impl Handler<UdpCommand> for BeaconUDP {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: UdpCommand, context: &mut Context<Self>) -> Self::Result {

        let broad: SocketAddr = "127.0.0.255:8082".parse().unwrap();
        match msg {
            UdpCommand::StartEmergency => {
                println!("fuck fuck");
                self.sink_buffer.push((Bytes::from("start"), broad));
                context.notify(InternalCommand::Flush);
            },
            UdpCommand::EndEmergency => {
                println!("fuck 2fuck");
                self.sink_buffer.push((Bytes::from("end"), broad));
                context.notify(InternalCommand::Flush);
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
            //tokio::spawn(fut);
            BeaconUDP {
                sink: Some(sink),
                sink_buffer: Vec::new(),
                beacon_ips: Vec::new(),
            }
        })
    }
}
