use std::collections::HashMap;
use sqlx::PgPool;

async fn get_airports(db_pool: &PgPool) -> Vec<String> {
    match sqlx::query!(
        "SELECT DISTINCT airport_code as code FROM airports"
    )
        .fetch_all(db_pool)
        .await
    {
        Ok(result) => {
            result.into_iter().map(|e| e.code.unwrap()).collect()
        }
        Err(_) => {
            panic!("((")
        }
    }
}

async fn get_neighbours(current: String, db_pool: &PgPool) -> Vec<String> {
    match sqlx::query!(
        "
        SELECT DISTINCT
            arrival_airport
        FROM routes
        WHERE departure_airport=$1
        ",
        current.as_str()
    )
        .fetch_all(db_pool)
        .await
    {
        Ok(result) => {
            let r = result.into_iter().map(|x| x.arrival_airport.unwrap()).collect();

            println!("Find from {} to -> {:?}", &current, &r);

            r
        }
        Err(_) => {
            panic!("On no)")
        }
    }
}

pub async fn compute_neighbours(db_pool: &PgPool) -> HashMap<String, Vec<String>> {
    // let airports: Vec<String> = get_airports(db_pool).await;
    // 
    // let mut airport_neighbours: HashMap<String, Vec<String>> = HashMap::new();
    // 
    // for airport in airports {
    //     airport_neighbours.insert(
    //         airport.clone(),
    //         get_neighbours(airport.clone(), db_pool).await
    //     );
    // }
    // 
    // airport_neighbours

    let mut airport_neighbours: HashMap<String, Vec<String>> = HashMap::new();
    
    let records = match sqlx::query!(
        "
        SELECT DISTINCT
        departure_airport, array_agg(DISTINCT arrival_airport) as arrival_airport
        FROM routes
        GROUP BY departure_airport
        "
    )
        .fetch_all(db_pool)
        .await
    {
        Ok(result) => {
            result
        }
        Err(_) => {
            panic!("On no)")
        }
    };
    
    for record in records {
        airport_neighbours.insert(
            record.departure_airport.unwrap(),
            record.arrival_airport.unwrap()
        );
    }
    
    airport_neighbours
}