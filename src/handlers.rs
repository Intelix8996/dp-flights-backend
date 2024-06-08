use actix_web::{HttpResponse, Responder, web};
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::app_state::AppState;
use crate::booking::create_booking_entries;
use crate::find_flights::find_flights;
use crate::types::{AirportCode, BookingClass, LocationType};

pub async fn list_cities(state: web::Data<AppState>) -> impl Responder {
    match sqlx::query!("SELECT DISTINCT city FROM airports")
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(result) => {
            HttpResponse::Ok().json(
                result.into_iter().map(|x| x.city.unwrap()).collect::<Vec<String>>()
            )
        }
        Err(_) => {
            HttpResponse::InternalServerError().json("")
        }
    }
}

#[derive(Serialize)]
struct Airport {
    code: Option<String>,
    name: Option<String>
}

pub async fn list_all_airports(state: web::Data<AppState>) -> impl Responder {
    match sqlx::query_as!(
        Airport,
        "SELECT airport_name as name, airport_code as code FROM airports"
    )
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(result) => {
            HttpResponse::Ok().json(result)
        }
        Err(_) => {
            HttpResponse::InternalServerError().json("")
        }
    }
}

async fn get_airports_within_city(city: &str, pool: &PgPool) -> Vec<Airport> {
    sqlx::query_as!(
        Airport,
        "SELECT airport_name as name, airport_code as code FROM airports WHERE city=$1",
        city
    )
        .fetch_all(pool)
        .await
        .unwrap()
}

pub async fn list_airports_within_city(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(
        get_airports_within_city(path.as_str(), &state.db_pool).await
    )
}

#[derive(Serialize)]
struct InboundRoute {
    flight_no: Option<String>,
    arrival_time: Option<NaiveTime>,
    origin: Option<String>,
    days_of_week: Option<Vec<i32>>
}

#[derive(Serialize)]
struct OutboundRoute {
    flight_no: Option<String>,
    departure_time: Option<NaiveTime>,
    destination: Option<String>,
    days_of_week: Option<Vec<i32>>
}

pub async fn inbound_schedule(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    match sqlx::query_as!(
        InboundRoute,
        "
        SELECT DISTINCT
            flights_v.flight_no AS flight_no,
            days_of_week,
            scheduled_arrival::TIME AS arrival_time,
            flights_v.departure_airport AS origin
        FROM flights_v
        JOIN routes ON flights_v.flight_no = routes.flight_no
        WHERE flights_v.arrival_airport=$1
        ",
        path.as_str()
    )
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(result) => {
            HttpResponse::Ok().json(result)
        }
        Err(_) => {
            HttpResponse::InternalServerError().json("")
        }
    }
}
pub async fn outbound_schedule(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    match sqlx::query_as!(
        OutboundRoute,
        "
        SELECT DISTINCT
            flights_v.flight_no AS flight_no,
            days_of_week,
            scheduled_departure::TIME AS departure_time,
            flights_v.arrival_airport AS destination
        FROM flights_v
        JOIN routes ON flights_v.flight_no = routes.flight_no
        WHERE flights_v.departure_airport=$1
        ",
        path.as_str()
    )
        .fetch_all(&state.db_pool)
        .await
    {
        Ok(result) => {
            HttpResponse::Ok().json(result)
        }
        Err(_) => {
            HttpResponse::InternalServerError().json("")
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ListRoutesParameters {
    source_type: LocationType,
    source: String,

    destination_type: LocationType,
    destination: String,

    max_connections: u8,

    connection_time_min: u8,
    connection_time_max: u8,

    departure_date: NaiveDate,
    
    booking_class: BookingClass
}

async fn convert_location_to_airport_codes(location_type: LocationType, location: String, pool: &PgPool) -> Vec<AirportCode> {
    match location_type {
        LocationType::CITY => {
            get_airports_within_city(location.as_str(), pool)
                .await
                .into_iter()
                .map(|x| x.code.unwrap())
                .collect()
        }
        LocationType::AIRPORT => {
            vec![location]
        }
    }
}

pub async fn list_routes(parameters: web::Query<ListRoutesParameters>, state: web::Data<AppState>) -> impl Responder {
    let source_airports = convert_location_to_airport_codes(
        parameters.source_type.clone(),
        parameters.source.clone(),
        &state.db_pool
    ).await;

    let destination_airports = convert_location_to_airport_codes(
        parameters.destination_type.clone(),
        parameters.destination.clone(),
        &state.db_pool
    ).await;
    
    let flights = find_flights(
        source_airports,
        destination_airports,
        parameters.departure_date,
        parameters.max_connections as i32,
        parameters.connection_time_min as i32,
        parameters.connection_time_max as i32,
        parameters.booking_class,
        &state.db_pool
    ).await.unwrap();
    
    HttpResponse::Ok().json(flights)
}

#[derive(Serialize, Deserialize)]
pub struct CreateBookingParameters {
    pub passenger_name: String,
    pub passenger_id: String,

    pub flight_ids: Vec<i32>,
    pub fare_conditions: BookingClass
}

#[derive(Serialize, Deserialize)]
pub struct CreateBookingResult {
    pub booking_id: String,
    pub ticker_no: String,

    pub total_price: i32,
}

pub async fn create_booking(parameters: web::Json<CreateBookingParameters>, state: web::Data<AppState>) -> impl Responder {

    let pool = &state.db_pool;

    let mut transaction = pool.begin().await.unwrap();
    
    let result = match create_booking_entries(parameters.into_inner(), &mut transaction).await {
        Ok(r) => {
            transaction.commit().await.unwrap();
            r
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(e.to_string());
        }
    };

    HttpResponse::Ok().json(result)
}