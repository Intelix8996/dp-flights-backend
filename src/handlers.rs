use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Instant;
use actix_web::{HttpResponse, Responder, web};
use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use crate::app_state::AppState;

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
pub async fn list_airports_within_city(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    match sqlx::query_as!(
        Airport,
        "SELECT airport_name as name, airport_code as code FROM airports WHERE city=$1",
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

fn dfs(depth_limit: usize, u: String, v: String, visited: &mut HashSet<String>, current_path: &mut VecDeque<String>, paths: &mut Vec<Vec<String>>, airport_neighbours: &HashMap<String, Vec<String>>) {
    if current_path.len() > depth_limit {
        return;
    }

    if visited.contains(u.as_str()) {
        return;
    }

    // println!("DFS Iter {} {} {:?} {:?} {}", u, v, visited, current_path, paths.len());

    visited.insert(u.clone());

    current_path.push_back(u.clone());

    if *u == *v {
        paths.push(current_path.clone().into_iter().collect());
        visited.remove(u.as_str());
        current_path.pop_back();
        return;
    }

    let neighbours = airport_neighbours.get(u.as_str()).unwrap();
    for neighbour in neighbours {
        dfs(depth_limit, neighbour.clone(), v.clone(), visited, current_path, paths, airport_neighbours);
    }

    current_path.pop_back();
    visited.remove(u.as_str());
}

fn find_all_paths(depth_limit: usize, from: String, to: String, paths: &mut Vec<Vec<String>>, airport_neighbours: &HashMap<String, Vec<String>>) {
    let mut visited = HashSet::new();
    let mut current_path = VecDeque::new();

    dfs(depth_limit, from, to, &mut visited, &mut current_path, paths, airport_neighbours);
}

pub async fn list_routes(path: web::Path<(String, String, usize)>, state: web::Data<AppState>) -> impl Responder {
    let (from, to, depth) = path.into_inner();
    
    let mut paths = Vec::new();

    let timer = Instant::now();
    find_all_paths(depth, from.clone(), to.clone(), &mut paths, &state.airport_neighbours);
    println!("Depth {}, path count: {}, elapsed: {:?}", depth, paths.len(), &timer.elapsed());
    
    // for i in 1..15 {
    //     println!("Start");
    //     let timer = Instant::now();
    //     find_all_paths(i.clone(), from.clone(), to.clone(), &mut paths, &airport_neighbours);
    //     println!("Depth {}, path count: {}, elapsed: {:?}", &i, paths.len(), &timer.elapsed());
    //     paths.clear();
    // }
    
    HttpResponse::Ok().json(paths)
}