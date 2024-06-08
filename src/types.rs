use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum LocationType {
    #[serde(rename = "City")]
    CITY,

    #[serde(rename = "Airport")]
    AIRPORT,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
pub enum BookingClass {
    Economy,
    Comfort,
    Business,
}

impl From<BookingClass> for String {
    fn from(value: BookingClass) -> Self {
        match value {
            BookingClass::Economy => { "Economy".to_string() }
            BookingClass::Comfort => { "Comfort".to_string() }
            BookingClass::Business => { "Business".to_string() }
        }
    }
}

pub type AirportCode = String;
