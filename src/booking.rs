use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use rand::Rng;
use sqlx::types::Decimal;

use crate::handlers::{CreateBookingParameters, CreateBookingResult};

#[derive(Debug)]
struct NoFreeSpace;

impl Display for NoFreeSpace {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str("Flight has no free seats")
    }
}

impl Error for NoFreeSpace {}

pub async fn create_booking_entries(parameters: CreateBookingParameters, transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>) -> Result<CreateBookingResult, Box<dyn Error>> {
    let prices: HashMap<i32, i32> = sqlx::query!(
        "
        SELECT flights_v.flight_id, prices.amount
        FROM flights_v
        JOIN prices ON flights_v.flight_no = prices.flight_no
        WHERE flights_v.flight_id = ANY($1::INT[])
          AND fare_conditions = $2
        ",
        parameters.flight_ids.as_slice(),
        String::from(parameters.fare_conditions)
    )
        .fetch_all(&mut **transaction)
        .await?
        .iter()
        .map(|x| (x.flight_id.unwrap(), x.amount))
        .collect();

    let total_price: i32 = prices.iter().map(|(_, price)| price).sum();

    let mut rng = rand::thread_rng();

    let book_ref = format!("_FFF{:X}", rng.gen_range(0..256));

    let ticket_no = format!("_9999999999{:X}", rng.gen_range(0..256));

    println!("{}", book_ref);

    sqlx::query!(
        "
        INSERT INTO bookings (book_ref, book_date, total_amount)
        VALUES      ($1, bookings.now(), $2);
        ",
        book_ref, Decimal::from(total_price)
    )
        .execute(&mut **transaction)
        .await?;

    sqlx::query!(
        "
        INSERT INTO tickets (ticket_no, book_ref, passenger_id, passenger_name)
        VALUES      ($1, $2, $3, $4);
        ",
        ticket_no, book_ref, parameters.passenger_id, parameters.passenger_name
    )
        .execute(&mut **transaction)
        .await?;

    for flight_id in parameters.flight_ids {
        let price = Decimal::from(*prices.get(&flight_id).unwrap_or(&0));

        sqlx::query!(
        "
        INSERT INTO ticket_flights (ticket_no, flight_id, fare_conditions, amount)
        VALUES      ($1, $2, $3, $4);
        ",
        ticket_no, flight_id, String::from(parameters.fare_conditions), price
    )
            .execute(&mut **transaction)
            .await?;
    }

    Ok(CreateBookingResult {
        booking_id: book_ref,
        ticker_no: ticket_no,
        total_price,
    })
}