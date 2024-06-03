use std::time::Instant;
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use crate::app_state::AppState;

async fn create_table(pool: &PgPool) {
    sqlx::query!("DROP TABLE IF EXISTS seats_comfort").execute(pool).await.unwrap();

    sqlx::query!(
        "
        CREATE TABLE seats_comfort
        (
            aircraft_code   CHAR(3)         NOT NULL,
            seat_no         VARCHAR(4)      NOT NULL,
            fare_conditions VARCHAR(10)     NOT NULL,
        
            PRIMARY KEY (aircraft_code, seat_no)
        );
        "
    )
        .execute(pool)
        .await
        .unwrap();

    sqlx::query!(
        "
        INSERT INTO seats_comfort
        SELECT * FROM seats;
        "
    )
        .execute(pool)
        .await
        .unwrap();

    sqlx::query!(
        "
        UPDATE seats_comfort
        SET fare_conditions = 'Comfort'
        FROM
        (
            SELECT DISTINCT aircraft_code, seat_no
            FROM ticket_flights
                     JOIN flights ON ticket_flights.flight_id = flights.flight_id
                     JOIN boarding_passes ON ticket_flights.ticket_no = boarding_passes.ticket_no
                AND ticket_flights.flight_id = boarding_passes.flight_id
                     JOIN prices ON flights.flight_no = prices.flight_no AND ticket_flights.amount = prices.amount
            WHERE prices.fare_conditions = 'Comfort'
        ) AS comfort_seats_query
        WHERE seats_comfort.seat_no = comfort_seats_query.seat_no
                AND seats_comfort.aircraft_code = comfort_seats_query.aircraft_code
        "
    )
        .execute(pool)
        .await
        .unwrap();
}

pub async fn compute_seats(state: web::Data<AppState>) -> impl Responder {
    let timer = Instant::now();

    create_table(&state.db_pool).await;

    HttpResponse::Ok().json(format!("Ok, create seats table in {:?}", timer.elapsed()))
}