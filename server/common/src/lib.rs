extern crate serde_derive;
extern crate nalgebra as na;

use serde_derive::{ Deserialize, Serialize };
use std::time::{ SystemTime, UNIX_EPOCH };

pub const EMERGENCY: &str = "/emergency";
pub const END_EMERGENCY: &str = "/endemergency";
pub const PING: &str = "/hello";
pub const DIAGNOSTICS: &str = "/diagnostics";
pub const REALTIME_USERS: &str = "/realtime_users";
pub const BEACON: &str = "/beacon";

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct HelloFrontEnd {
    pub data: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    RSSI(i64),
    TOF(i64)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCommandResponse {
    pub emergency: bool,
}

impl SystemCommandResponse {
    pub fn new(emergency: bool) -> SystemCommandResponse {
        SystemCommandResponse {
            emergency,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagData {
    pub beacon_mac: String,
    pub tag_distance: DataType,
    pub tag_mac: String,
    pub tag_name: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticData {
    pub tag_data: Vec<TagData>,
}

impl DiagnosticData {
    pub fn new() -> DiagnosticData {
        DiagnosticData {
            tag_data: Vec::new(),
        }
    }
}

// NOTE temporary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBeaconSourceLocations {
    pub name: String,
    pub location: na::Vector2<f64>,
    pub distance_to_tag: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    //floor: String,
    pub last_active: SystemTime,
    pub location: na::Vector2<f64>,
    pub tag_mac: String,

    // NOTE TEMPORARY
    pub beacon_sources: Vec<UserBeaconSourceLocations>,
}

impl User {
    pub fn new(tag_mac: String) -> User {
        User {
            last_active: UNIX_EPOCH,
            location: na::Vector2::new(0., 0.),
            tag_mac,
            beacon_sources: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beacon {
    pub mac: String,
    pub name: String,
}

impl Beacon {
    pub fn new(mac: String) -> Beacon {
        Beacon {
            mac,
            name: "unknown".to_string(),
        }
    }
}
