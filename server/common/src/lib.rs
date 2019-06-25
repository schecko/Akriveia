extern crate serde_derive;

use serde_derive::{Deserialize, Serialize};

pub const EMERGENCY: &str = "/emergency";
pub const PING: &str = "/hello";
pub const DIAGNOSTICS: &str = "/diagnostics";

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloFrontEnd {
    pub data: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DataType {
    RSSI(i64),
    TOF(i64)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TagData {
    pub name: String,
    pub mac_address: String,
    pub distance: DataType,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiagnosticsData {
    pub tag_data: Vec<TagData>,
}
