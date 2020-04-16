use crate::utils::Json;
use serde_json::Map;
use serde::Serialize;

pub struct FlightData {}

impl FlightData {
    pub fn random() -> FlightData {
        FlightData {}
    }

    pub fn to_json(&self) -> FlightDataJson {
        FlightDataJson {}
    }
}

#[derive(Serialize)]
pub struct FlightDataJson {}

