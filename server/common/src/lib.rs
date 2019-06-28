extern crate serde_derive;

use serde_derive::{Deserialize, Serialize};

pub const EMERGENCY: &str = "/emergency";
pub const END_EMERGENCY: &str = "/endemergency";
pub const PING: &str = "/hello";
pub const DIAGNOSTICS: &str = "/diagnostics";

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloFrontEnd {
    pub data: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    RSSI(i64),
    TOF(i64)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagData {
    pub name: String,
    pub mac_address: String,
    pub distance: DataType,
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
