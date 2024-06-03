use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub enum LocationType {
    #[serde(rename = "City")]
    CITY,

    #[serde(rename = "Airport")]
    AIRPORT,
}

#[derive(Serialize, Deserialize)]
pub enum BookingClass {
    #[serde(rename = "Economy")]
    ECONOMY,

    #[serde(rename = "Comfort")]
    COMFORT,

    #[serde(rename = "Business")]
    BUSINESS,
}

pub type AirportCode = String;
