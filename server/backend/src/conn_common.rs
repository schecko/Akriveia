
use common::MacAddress;
use regex::Regex;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum MessageError {
    ParseFormat,
    ParseFloat,
    ParseMac,
}

impl fmt::Display for MessageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageError::ParseFormat => write!(f, "Invalid message format"),
            MessageError::ParseFloat => write!(f, "Failed to parse float"),
            MessageError::ParseMac => write!(f, "Failed to parse mac address"),
        }
    }
}

impl Error for MessageError {
     fn cause(&self) -> Option<&dyn Error> {
         None
     }
 }

impl From<eui48::ParseError> for MessageError {
    fn from(_item: eui48::ParseError) -> Self {
        MessageError::ParseMac
    }
}

impl From<std::num::ParseFloatError> for MessageError {
    fn from(_item: std::num::ParseFloatError) -> Self {
        MessageError::ParseFloat
    }
}

pub fn parse_message(message: &str) -> Result<common::TagData, MessageError> {

    let split: Vec<&str> = message.split("|").collect();
    if split.len() == 3 {
        let beacon_mac = MacAddress::parse_str(split[0])?;
        let tag_mac = MacAddress::parse_str(split[1])?;
        let distance = split[2];
        let reg = Regex::new(r"/[^$0-9]+/").unwrap();
        let distance_stripped = reg.replace_all(&distance, "");

        // remove the last character every time, idk why but there is always
        // a newline at the end of rssi_stripped. from_str_radix REQUIRES
        // all numeric characters.
        if distance_stripped.len() <= 0 {
            return Err(MessageError::ParseFormat)
        }
        let distance_numeric = distance_stripped[..distance_stripped.len() - 1].parse::<f64>()?;
        Ok(common::TagData {
            beacon_mac: beacon_mac.clone(),
            tag_distance: distance_numeric,
            tag_mac,
        })
    } else {
        // incomplete transmission
        Err(MessageError::ParseFormat)
    }
}
