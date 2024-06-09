use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use sqlx::PgPool;

#[derive(Debug)]
pub struct NotRegisteredError;

impl Display for NotRegisteredError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("This passenger is not registered for this flight")
    }
}

impl Error for NotRegisteredError {}

pub async fn check_in_passanger(ticket_no: String, flight_id: i32, pool: &PgPool) -> Result<String, Box<dyn Error>> {
    let boarded_count = sqlx::query!(
        "
        SELECT COUNT(ticket_no) FROM boarding_passes
        WHERE flight_id = $1
        ",
        flight_id
    )
        .fetch_one(pool)
        .await?
        .count
        .unwrap_or(0);

    let boarding_number = boarded_count + 1;

    let fare_condition = match sqlx::query!(
        "
        SELECT fare_conditions FROM ticket_flights
        WHERE ticket_no = $1 AND flight_id = $2;
        ",
        ticket_no, flight_id
    )
        .fetch_optional(pool)
        .await?
    {
        Some(r) => { r.fare_conditions }
        None => { return Err(Box::new(NotRegisteredError)); }
    };

    let aircraft_code = sqlx::query!(
        "
        SELECT aircraft_code FROM flights_v
        WHERE flight_id = $1
        ",
        flight_id
    )
        .fetch_one(pool)
        .await?
        .aircraft_code
        .unwrap();

    let place = sqlx::query!(
        "
        WITH occupied_seats_query AS (
            SELECT COALESCE(array_agg(seat_no), '{}'::VARCHAR[]) AS occupied_seats
            FROM boarding_passes
            WHERE flight_id = $1
        )
        SELECT seat_no
        FROM seats_comfort, occupied_seats_query
        WHERE aircraft_code = $2 AND fare_conditions = $3
                AND NOT (seats_comfort.seat_no = ANY(occupied_seats_query.occupied_seats));
        ",
        flight_id, aircraft_code, fare_condition
    )
        .fetch_one(pool)
        .await?
        .seat_no;

    sqlx::query!(
        "
        INSERT INTO boarding_passes (ticket_no, flight_id, boarding_no, seat_no)
        VALUES      ($1, $2, $3, $4);
        ",
        ticket_no, flight_id, boarding_number as i32, place
    )
        .execute(pool)
        .await?;

    Ok(place)
}