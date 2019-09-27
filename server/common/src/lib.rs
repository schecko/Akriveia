extern crate serde_derive;
extern crate nalgebra as na;
extern crate eui48;
extern crate ipnet;

use serde_derive::{ Deserialize, Serialize, };
pub use eui48::MacAddress;
pub use chrono::{ DateTime, Utc, };
pub use chrono::offset::TimeZone;
use std::net::{ IpAddr, Ipv4Addr, };
use ipnet::{ IpNet, Ipv4Net, };

pub fn beacon_url(id: &str) -> String {
    return format!("/beacon/{}", id);
}
pub fn beacons_url() -> String {
    return String::from("/beacons");
}
pub fn beacons_for_map_url(id: &str) -> String {
    return format!("/map/{}/beacons", id);
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
    pub tag_distance: f64,
    pub tag_mac: MacAddress,
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
pub struct TrackedUser {
    pub id: i32,
    pub coordinates: na::Vector2<f64>,
    pub emergency_contact: Option<i32>,
    pub employee_id: Option<String>,
    pub last_active: DateTime<Utc>,
    pub mac_address: MacAddress,
    pub map_id: Option<i32>,
    pub name: String,
    pub note: Option<String>,
    pub phone_number: Option<String>,

    // NOTE TEMPORARY
    pub beacon_sources: Vec<UserBeaconSourceLocations>,
}

impl TrackedUser {
    pub fn new() -> TrackedUser {
        TrackedUser {
            id: -1, // primary key
            coordinates: na::Vector2::new(0.0, 0.0),
            emergency_contact: None,
            employee_id: None,
            last_active: Utc.timestamp(0, 0),
            mac_address: MacAddress::nil(),
            map_id: None,
            name: String::new(),
            note: None,
            phone_number: None,

            beacon_sources: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Beacon {
    pub id: i32, // primary key
    pub coordinates: na::Vector2<f64>,
    pub ip: IpAddr,
    pub mac_address: MacAddress,
    pub map_id: Option<i32>,
    pub name: String,
    pub note: Option<String>,
}

impl Beacon {
    pub fn new() -> Beacon {
        Beacon {
            id: -1,
            coordinates: na::Vector2::new(0.0, 0.0),
            ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            mac_address: MacAddress::nil(),
            map_id: None,
            name: String::new(),
            note: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub id: i32, // primary key
    pub blueprint: Vec<u8>,
    pub bounds: na::Vector2<i32>,
    pub name: String,
    pub note: Option<String>,
    pub scale: f64, // pixels per meter
}

impl Map {
    pub fn new() -> Map {
        Map {
            id: -1,
            blueprint: Vec::new(),
            bounds: na::Vector2::new(0, 0),
            name: String::new(),
            note: None,
            scale: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub id: i32, // primary key
    pub beacon_port: Option<i16>,
    pub ip: Ipv4Net,
    pub mac: MacAddress,
    pub name: String,
    pub webserver_port: Option<i16>,
}

impl NetworkInterface {
    pub fn new() -> NetworkInterface {
        NetworkInterface {
            id: -1,
            beacon_port: None,
            ip: Ipv4Net::new(Ipv4Addr::new(0, 0, 0, 0), 32).unwrap(),
            mac: MacAddress::nil(),
            name: String::new(),
            webserver_port: None,
        }
    }
}
