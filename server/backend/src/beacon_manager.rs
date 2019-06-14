
// this "manager" exists for two main purposes, first the initial implementation will involve
// serial communcation with the beacons, which will be synchronous. Since Actix is a single threaded
// event loop this would not be a good time for our webserver, so instead the work will be shoved
// off to "SyncArbitors" which gives an api like normal actix, but gives each actor(or each beacon
// in this case) its own communcation thread. The second reason is to hopefully abstract this
// functionality a little bit, so that it will be a little bit easier to move to the wireless
// implementation.
use actix::prelude::*;

pub struct BeaconManager;

pub enum BeaconState {
    On,
    Off,
}

pub enum BeaconOps {
    FindBeacons,
    TriggerBeacons(BeaconState),
}

impl Message for BeaconOps {
    type Result = Result<u64, ()>;
}

impl Actor for BeaconManager {
    type Context = Context<Self>;
}


impl Handler<BeaconOps> for BeaconManager {
    type Result = Result<u64, ()>;

    fn handle(&mut self, msg: BeaconOps, _: &mut Context<Self>) -> Self::Result {
        match msg {
            BeaconOps::FindBeacons => {
                // find the beacons
            },
            BeaconOps::TriggerBeacons(BeaconState::On) => {
                // tell all of the beacons to activate
            }
            BeaconOps::TriggerBeacons(BeaconState::Off) => {
                // tell all of the beacons to sleep
            }
        }
        Ok(1)
    }
}

