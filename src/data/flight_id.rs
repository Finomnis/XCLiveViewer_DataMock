use uuid::Uuid;

pub type FlightId = String;

pub fn generate_flight_id() -> FlightId {
    Uuid::new_v4().to_hyphenated().to_string()
}
