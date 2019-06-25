extern crate serde_derive;

use serde_derive::{Deserialize, Serialize};

pub const EMERGENCY: &str = "/emergency";
pub const PING: &str = "/hello";

#[derive(Debug, Serialize, Deserialize)]
pub struct HelloFrontEnd {
    pub data: u32,
}
