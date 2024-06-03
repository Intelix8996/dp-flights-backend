use std::time::Instant;
use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use crate::app_state::AppState;

async fn create_table(pool: &PgPool) {
    sqlx::query!("DROP TABLE IF EXISTS prices").execute(pool).await.unwrap();
    
    sqlx::query!(
        "
        CREATE TABLE prices
        (
            flight_no       CHAR(6)                 NOT NULL,
            fare_conditions VARCHAR(10)             NOT NULL,
            amount          INT                     NOT NULL,

            PRIMARY KEY (flight_no, fare_conditions)
        );
        "
    )
        .execute(pool)
        .await
        .unwrap();

    sqlx::query!(
        "
        INSERT INTO prices (flight_no, fare_conditions, amount)
        SELECT DISTINCT flight_no, fare_conditions, min(DISTINCT amount) AS price
        FROM ticket_flights
                 JOIN flights ON ticket_flights.flight_id = flights.flight_id
        WHERE fare_conditions != 'Comfort'
        GROUP BY flight_no, fare_conditions;
        "
    )
        .execute(pool)
        .await
        .unwrap();

    sqlx::query!(
        "
        INSERT INTO prices (flight_no, fare_conditions, amount)
        SELECT DISTINCT flight_no, 'Comfort' as fare_conditions, max(DISTINCT amount) AS comfort_price
        FROM ticket_flights
                 JOIN flights ON ticket_flights.flight_id = flights.flight_id
        GROUP BY flight_no, fare_conditions
        HAVING fare_conditions = 'Economy' AND count(DISTINCT amount) = 2;
        "
    )
        .execute(pool)
        .await
        .unwrap();
}

pub async fn compute_prices(state: web::Data<AppState>) -> impl Responder {
    let timer = Instant::now();
    
    create_table(&state.db_pool).await;
    
    HttpResponse::Ok().json(format!("Ok, create prices table in {:?}", timer.elapsed()))
}