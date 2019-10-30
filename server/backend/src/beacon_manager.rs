
// this "manager" exists for two main purposes, first the initial implementation will involve
// serial communcation with the beacons, which will be synchronous. Since Actix is a single threaded
// event loop this would not be a good time for our webserver, so instead the work will be shoved
// off to "SyncArbitors" which gives an api like normal actix, but gives each actor(or each beacon
// in this case) its own communcation thread. The second reason is to hopefully abstract this
// functionality a little bit, so that it will be a little bit easier to move to the wireless
// implementation.

use actix::prelude::*;
use actix_web::Result;
use crate::beacon_udp::*;
use crate::dummy_udp::*;
use crate::data_processor::*;
use crate::db_utils;
use crate::models::network_interface;
use std::io;
use std::time::Duration;
use std::collections::{ BTreeSet, };
use common::*;
use multi_map::MultiMap;
use std::net::{ IpAddr, };

pub struct BeaconManager {
    state: BeaconState,
    data_processor: Addr<DataProcessor>,
    diagnostic_data: common::DiagnosticData,
    udp_connections: Vec<Addr<BeaconUDP>>,
    dummy_udp_connections: Vec<Addr<DummyUDP>>,
    pinger: SpawnHandle,
    self_health: SpawnHandle,
    beacons: MultiMap<MacAddress8, IpAddr, RealtimeBeacon>,
    // the outer vec should be a fixed size length
    // array, but rust arrays are silly to work with.
    // Assert the length of the outer array is equal to BeaconState::count().
    beacons_state: Vec<BTreeSet<MacAddress8>>,
}

impl Actor for BeaconManager {
    type Context = Context<Self>;
}

const USE_DUMMY_BEACONS: bool = true;
const USE_UDP_BEACONS: bool = true;
const PING_INTERVAL: Duration = Duration::from_millis(10000);
const EMERGENCY_PING_INTERVAL: Duration = Duration::from_millis(1000);
const RESPONSE_THRESHOLD: Duration = Duration::from_millis(200);

pub enum BMCommand {
    EndEmergency,
    GetEmergency,
    ScanBeacons,
    StartEmergency,
    Ping(Option<MacAddress8>),
    Reboot(Option<MacAddress8>),
}

pub enum BMResponse {
    Start(IpAddr, MacAddress8),
    End(IpAddr, MacAddress8),
    Ping(IpAddr, MacAddress8),
    Reboot(IpAddr, MacAddress8),
}

impl Message for BMCommand {
    type Result = Result<common::SystemCommandResponse, io::Error>;
}

#[derive(Clone, Copy)]
pub enum BeaconCommand {
    EndEmergency,
    StartEmergency,
    Ping(Option<IpAddr>),
    Reboot(Option<IpAddr>),
}

impl Message for BeaconCommand {
    type Result = Result<(), ()>;
}

impl Message for BMResponse {
    type Result = Result<(), ()>;
}

impl BeaconManager {
    pub fn new(dp: Addr<DataProcessor>) -> Addr<BeaconManager> {
        let mut state_vec = Vec::new();
        state_vec.reserve_exact(BeaconState::count());
        BeaconManager::create(move |context| {
            let mut manager = BeaconManager {
                data_processor: dp,
                diagnostic_data: common::DiagnosticData::new(),
                emergency: false, // TODO get from db!
                udp_connections: Vec::new(),
                dummy_udp_connections: Vec::new(),
                pinger: Default::default(),
                self_health: Default::default(),
                beacons: MultiMap::new(),
                beacons_state: state_vec,
            };
            manager.ping_self(context, PING_INTERVAL);
            manager
        })
    }

    fn check_health(&mut self, context: &mut Context<Self>) {
        self.beacons.iter().filter(|beacon| Utc::now - beacon.last_active < RESPONSE_THRESHOLD).for_each(|beacon| {
            context.notify(BMCommand(Ping(Some(beacon.mac_address))));
        })
    }

    fn ping_self(&mut self, ctx: &mut Context<Self>, dur: Duration) {
        ctx.cancel_future(self.pinger);
        self.pinger = ctx.run_interval(dur, |_actor, context| {
            context.notify(BMCommand::Ping(None));
        });

        match self.state {
            BeaconState::Idle => {
                self.beacons_state
                    .iter()
                    .filter(|state_type| state_type != BeaconState::Idle)
                    .iter()
                    .for_each(|mac| {
                        let beacon = self.beacons.get(&mac).unwrap();
                        context.notify(BMCommand::Ping(beacon.ip));
                    });
            },
            BeaconState::Active => {
                self.beacons_state
                    .iter()
                    .filter(|state_type| state_type != BeaconState::Active)
                    .iter()
                    .for_each(|mac| {
                        let beacon = self.beacons.get(&mac).unwrap();
                        context.notify(BMCommand::StartEmergency(beacon.ip));
                    });
            },
            _ => {
                panic!("Beacon manager can only be in active or idle state");
            }
        }
    }

    fn find_beacons(&mut self, context: &mut Context<Self>) {
        if USE_DUMMY_BEACONS { self.find_beacons_dummy(context); }
        if USE_UDP_BEACONS { self.find_beacons_udp(context); }
    }

    fn find_beacons_udp(&mut self, context: &mut Context<Self>) {
        let fut = db_utils::default_connect()
            .and_then(|client| {
                network_interface::select_network_interfaces(client)
                    .map(|(_client, ifaces)| {
                        ifaces
                    })
            })
            .into_actor(self)
            .and_then(|ifaces, actor, context| {
                for iface in ifaces {
                    match iface.beacon_port {
                        Some(port) => {
                            actor.udp_connections.push(BeaconUDP::new(context.address(), iface.ip.clone(), port as u16));
                        },
                        None => {},
                    }
                }
                fut::result(Ok(()))
            })
            .map_err(|err, _, _| {
                println!("failed to create udp connection {}", err);
            });
        context.spawn(fut);
    }

    fn find_beacons_dummy(&mut self, context: &mut Context<Self>) {
        self.dummy_udp_connections.push(DummyUDP::new(context.address()));
    }

    fn mass_send(&self, msg: BeaconCommand) {
        for connection in &self.udp_connections {
            connection.do_send(msg);
        }

        for connection in &self.dummy_udp_connections {
            connection.do_send(msg);
        }
    }
}

impl Handler<BMCommand> for BeaconManager {
    type Result = Result<common::SystemCommandResponse, io::Error>;

    fn handle(&mut self, msg: BMCommand, context: &mut Context<Self>) -> Self::Result {
        match msg {
            BMCommand::GetEmergency => { },
            BMCommand::ScanBeacons => {
                println!("find beacons called!");
                self.find_beacons(context);
            },
            BMCommand::StartEmergency => {
                self.state = BeaconState::Active;
                self.diagnostic_data = common::DiagnosticData::new();
                self.emergency = true;
                self.ping_self(context, EMERGENCY_PING_INTERVAL);
                self.mass_send(BeaconCommand::StartEmergency);
            }
            BMCommand::EndEmergency => {
                self.state = BeaconState::Idle;
                self.mass_send(BeaconCommand::EndEmergency);
                self.emergency = false;
                self.data_processor.do_send(DPMessage::ResetData);
                self.ping_self(context, PING_INTERVAL);
            },
            BMCommand::Ping(opt_mac) => {
                if let Some(mac) = opt_mac {
                    let opt_beacon = self.beacons.get(&mac);
                    if let Some(beacon) = opt_beacon {
                        self.mass_send(BeaconCommand::Ping(Some(beacon.ip)));
                    } else {
                        self.beacons_state[usize::from(BeaconState::Unknown)].insert(mac);
                    }
                    self.self_health = ctx.run_later(RESPONSE_THRESHOLD, move |actor, _context| {
                        actor.check_health(context, Some(beacon.mac.clone()));
                    });
                } else {
                    self.mass_send(BeaconCommand::Ping(None));
                }
            },
            BMCommand::Reboot(opt_mac) => {
                if let Some(mac) = opt_mac {
                    let opt_beacon = self.beacons.get(&mac);
                    if let Some(beacon) = opt_beacon {
                        self.mass_send(BeaconCommand::Reboot(Some(beacon.ip)));
                    } else {
                        self.beacons_state[usize::from(BeaconState::Unknown)].insert(mac);
                    }
                } else {
                    self.mass_send(BeaconCommand::Reboot(None));
                }
            },
        }
        Ok(common::SystemCommandResponse::new(self.emergency))
    }
}

impl Handler<BMResponse> for BeaconManager {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: BMResponse, _context: &mut Context<Self>) -> Self::Result {
        match msg {
            BMResponse::Start(ip, mac) => {
                match self.beacons.get(&mac) {
                    Some(beacon) => {
                        self.beacons_state[usize::from(beacon.state)].remove(&mac)
                            .expect("Beacon {} state was set to {}, but was not in the proper array of manager.", beacon.mac, beacon.state);
                        self.beacons_state[usize::from(BeaconState::Active)].insert(mac);
                        beacon.state = BeaconState::Active;
                        beacon.last_active = Utc::now();
                    },
                    None => {
                        self.beacons_state[usize::from(BeaconState::Unknown)].insert(mac);
                    }
                }
            }
            BMResponse::End(ip, mac) => {
                match self.beacons.get(&mac) {
                    Some(beacon) => {
                        self.beacons_state[usize::from(beacon.state)].remove(&mac)
                            .expect("Beacon {} state was set to {}, but was not in the proper array of manager.", beacon.mac, beacon.state);
                        self.beacons_state[usize::from(BeaconState::Idle)].insert(mac);
                        beacon.state = BeaconState::Idle;
                        beacon.last_active = Utc::now();
                    },
                    None => {
                        self.beacons_state[usize::from(BeaconState::Unknown)].insert(mac);
                    }
                }
            },
            BMResponse::Ping(ip, mac) => {
                match self.beacons.get(&mac) {
                    Some(beacon) => {
                        self.beacons_state[usize::from(beacon.state)].remove(&mac)
                            .expect("Beacon {} state was set to {}, but was not in the proper array of manager.", beacon.mac, beacon.state);
                        self.beacons_state[usize::from(BeaconState::Active)].insert(mac);
                        beacon.state = BeaconState::Active;
                        beacon.last_active = Utc::now();
                    },
                    None => {
                        self.beacons_state[usize::from(BeaconState::Unknown)].insert(mac);
                    }
                }
            },
            BMResponse::Reboot(ip, mac) => {
                match self.beacons.get(&mac) {
                    Some(beacon) => {
                        self.beacons_state[usize::from(beacon.state)].remove(&mac)
                            .expect("Beacon {} state was set to {}, but was not in the proper array of manager.", beacon.mac, beacon.state);
                        self.beacons_state[usize::from(BeaconState::Rebooting)].insert(mac);
                        beacon.state = BeaconState::Rebooting;
                        beacon.last_active = Utc::now();
                    },
                    None => {
                        self.beacons_state[usize::from(BeaconState::Unknown)].insert(mac);
                    }
                }
            },

        }
        Ok(())
    }
}

pub struct GetDiagnosticData;
impl Message for GetDiagnosticData {
    type Result = Result<common::DiagnosticData, io::Error>;
}
impl Handler<GetDiagnosticData> for BeaconManager {
    type Result = Result<common::DiagnosticData, io::Error>;

    fn handle(&mut self, _msg: GetDiagnosticData, _context: &mut Context<Self>) -> Self::Result {
        let res = self.diagnostic_data.clone();
        self.diagnostic_data.tag_data = Vec::new();
        Ok(res)
    }
}

pub struct TagDataMessage {
    pub data: common::TagData,
}
impl Message for TagDataMessage {
    type Result = Result<(), ()>;
}
impl Handler<TagDataMessage> for BeaconManager {
    type Result = Result<(), ()>;

    fn handle(&mut self, msg: TagDataMessage, _context: &mut Context<Self>) -> Self::Result {
        self.diagnostic_data.tag_data.push(msg.data.clone());
        self.data_processor.do_send(InLocationData(msg.data));
        Ok(())
    }
}
