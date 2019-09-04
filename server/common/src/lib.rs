extern crate serde_derive;
extern crate nalgebra as na;
extern crate eui48;

use serde_derive::{ Deserialize, Serialize };
use std::time::{ SystemTime, UNIX_EPOCH };
pub use eui48::MacAddress;

pub fn beacon_url(id: &str) -> String {
    return format!("/beacon/{}", id);
}
pub fn beacons_url() -> String {
    return String::from("/beacons");
}

pub fn user_url(id: &str) -> String {
    return format!("/user/{}", id);
}
pub fn users_url() -> String {
    return String::from("/users");
}
pub fn users_realtime_url() -> String {
    return String::from("/users/realtime");
}

pub fn map_url(id: &str) -> String {
    return format!("/map/{}", id);
}
pub fn maps_url() -> String {
    return String::from("/maps");
}

pub fn system_network_url(id: &str) -> String {
    return format!("/system/network/{}", id);
}
pub fn system_networks_url() -> String {
    return String::from("/system/networks");
}

pub fn system_emergency_url() -> String {
    return String::from("/system/emergency/");
}

pub fn system_diagnostics_url() -> String {
    return String::from("/system/diagnostics/");
}

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
    pub beacon_mac: MacAddress,
    pub tag_distance: DataType,
    pub tag_mac: MacAddress,
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
    pub id: i32,
    pub last_active: SystemTime,
    pub location: na::Vector2<f64>,
    pub tag_mac: MacAddress,

    // NOTE TEMPORARY
    pub beacon_sources: Vec<UserBeaconSourceLocations>,
}

impl User {
    pub fn new() -> User {
        User {
            id: -1,
            last_active: UNIX_EPOCH,
            location: na::Vector2::new(0.0, 0.0),
            tag_mac: MacAddress::nil(),
            beacon_sources: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beacon {
    pub id: i32,
    pub mac_address: MacAddress,
    pub coordinates: na::Vector2<f64>,
    pub map_id: Option<String>,
    pub name: String,
    pub note: String,
}

impl Beacon {
    pub fn new() -> Beacon {
        Beacon {
            id: -1,
            mac_address: MacAddress::nil(),
            coordinates: na::Vector2::new(0.0, 0.0),
            map_id: None,
            name: "".to_string(),
            note: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub id: i32,
    pub blueprint: Vec<u8>,
    pub bounds: na::Vector2<f64>,
    pub name: String,
    pub note: String,
    pub scale: f64,
}

impl Map {
    pub fn new() -> Map {
        Map {
            id: -1,
            blueprint: Vec::new(),
            bounds: na::Vector2::new(0.0, 0.0),
            name: "".to_string(),
            note: "".to_string(),
            scale: 1.0,
        }
    }
}
