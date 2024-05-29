mod config;
mod app_state;
mod handlers;
mod neighbours;
mod prices;

use std::process::exit;
use std::time::Instant;
use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_web::web::route;
use dotenv::dotenv;
use serde::Serialize;
use sqlx::postgres::PgPoolOptions;
use crate::app_state::AppState;
use crate::config::Config;
use crate::handlers::{inbound_schedule, list_airports_within_city, list_all_airports, list_cities, list_routes, outbound_schedule};
use crate::neighbours::compute_neighbours;
use crate::prices::compute_prices;

#[derive(Serialize)]
struct ListCitiesResponse {
    cities: Vec<String>
}

#[derive(Serialize)]
struct Airport {
    code: Option<String>,
    name: Option<String>,
}

async fn index(state: web::Data<AppState>) -> impl Responder {

    match sqlx::query_as!(
        Airport,
        "
        SELECT airport_code as code, airport_name as name
        FROM airports
        GROUP BY code, name
        LIMIT 5
        "
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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = Config::init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .unwrap_or_else(|e| {
            println!("Error while connecting to DB: {}", e);
            exit(1);
        });

    let server_addr = config.server_addr.clone();

    let timer = Instant::now();
    let airport_neighbours = compute_neighbours(&pool).await;
    println!("Elapsed: {:?}", timer.elapsed());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                db_pool: pool.clone(),
                cfg: config.clone(),
                airport_neighbours: airport_neighbours.clone()
            }))
            .service(
                web::scope("/api")
                    .route("/cities", web::get().to(list_cities))
                    .route("/airports", web::get().to(list_all_airports))
                    .route("/city_airports/{city}", web::get().to(list_airports_within_city))
                    .route("/inbound/{airport_code}", web::get().to(inbound_schedule))
                    .route("/outbound/{airport_code}", web::get().to(outbound_schedule))
                    .route("/route/{from}/{to}/{depth}", web::get().to(list_routes))
                    .route("/compute_prices", web::post().to(compute_prices))
            )
    })
        .bind(server_addr.clone())
        .expect(format!("Can't bind {}", &server_addr).as_str())
        .run()
        .await
}
