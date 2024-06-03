use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc};
use serde::de::Error;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::types::{AirportCode, BookingClass};

#[derive(Serialize, Deserialize)]
pub struct FlightRecord {
    path: Option<Vec<String>>,
    flight_ids: Option<Vec<i32>>,
    departure_time: Option<DateTime<Utc>>,
    arrival_time: Option<DateTime<Utc>>,
    connections: Option<i32>
}

pub async fn find_flights(
    sources: Vec<AirportCode>,
    destinations: Vec<AirportCode>,
    departure_date: NaiveDate,
    max_connections: i32,
    connection_time_min: i32,
    connection_time_max: i32,
    booking_class: BookingClass,
    pool: &PgPool) -> Result<Vec<FlightRecord>, sqlx::Error> {

    sqlx::query_as!(
        FlightRecord,
        "
        WITH RECURSIVE flights_recur AS (
            SELECT
                ARRAY[f1.departure_airport]::VARCHAR[] AS path,
                    f1.arrival_airport AS end_point,
                f1.scheduled_departure AS departure_time,
                f1.scheduled_arrival AS arrival_time,
                ARRAY[f1.flight_id]::INT[] as flight_id,
                    0 as len
            FROM flights_v f1
            WHERE f1.departure_airport = ANY($1::VARCHAR[])
              AND f1.status = 'Scheduled'
              AND f1.scheduled_departure
                BETWEEN $3
                AND $3 + INTERVAL '24h'

            UNION

            SELECT
                flights_recur.path || flights_recur.end_point,
                fn.arrival_airport,
                flights_recur.departure_time,
                fn.scheduled_arrival,
                flights_recur.flight_id || fn.flight_id,
                len + 1 AS i
            FROM flights_recur
                     JOIN flights_v fn ON (
                flights_recur.end_point = fn.departure_airport
                    AND fn.status = 'Scheduled'
                    AND NOT (fn.arrival_airport = ANY(flights_recur.path))
                    AND fn.scheduled_departure
                    BETWEEN (flights_recur.arrival_time + INTERVAL '1h')
                    AND (flights_recur.arrival_time + INTERVAL '24h')
                -- AND free_seats(fn.flight_id, $10) IS NOT NULL
                )
            WHERE len < $4
        )
        SELECT
            flights_recur.path || flights_recur.end_point AS path,
            flights_recur.flight_id as flight_ids,
            departure_time,
            arrival_time,
            len AS connections
        FROM flights_recur
        WHERE flights_recur.end_point = ANY($2::VARCHAR[]);
        ",
        sources.as_slice(),
        destinations.as_slice(),
        DateTime::<Utc>::from_naive_utc_and_offset(NaiveDateTime::from(departure_date), Utc),
        max_connections,
        // format!("{}h", connection_time_min).to_string(),
        // connection_time_max,
        // booking_class,
    )
        .fetch_all(pool)
        .await
}
