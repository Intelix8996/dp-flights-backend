use actix_web::{HttpResponse, Responder, web};
use sqlx::PgPool;
use crate::app_state::AppState;

async fn create_table(pool: &PgPool) {
    sqlx::query!("DROP TABLE IF EXISTS prices").execute(pool).await.unwrap();
    
    sqlx::query!(
        "
        CREATE TABLE prices
        (
            id              BIGSERIAL PRIMARY KEY   NOT NULL,
            flight_no       CHAR(6)                 NOT NULL,
            fare_conditions VARCHAR(10)             NOT NULL,
            amount          INT                     NOT NULL
        )
        "
    )
        .execute(pool)
        .await
        .unwrap();
}

pub async fn compute_prices(state: web::Data<AppState>) -> impl Responder {
    create_table(&state.db_pool).await;
    
    HttpResponse::Ok().json("Ok, create table")
}