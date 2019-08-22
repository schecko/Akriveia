extern crate serde_derive;
extern crate nalgebra as na;

use serde_derive::{ Deserialize, Serialize };
use std::time::{ SystemTime, UNIX_EPOCH };

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
    pub mac_address: String,
    pub coordinates: na::Vector2<f64>,
    pub map_id: String,
    pub name: String,
    pub note: String,
}

impl Beacon {
    pub fn new(mac: String) -> Beacon {
        Beacon {
            mac_address: mac,
            coordinates: na::Vector2::new(0.0, 0.0),
            map_id: "unknown".to_string(),
            name: "unknown".to_string(),
            note: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub floor_id: String,
    pub blueprint: Vec<u8>,
    pub blueprint_bounds: na::Vector2<i32>,
    pub name: String,
    pub scale: f64,
}

impl Map {
    pub fn new(floor_id: String) -> Map {
        Map {
            floor_id,
            blueprint: Vec::new(),
            blueprint_bounds: na::Vector2::new(0, 0),
            name: "unknown".to_string(),
            scale: 1.0,
        }
    }
}
