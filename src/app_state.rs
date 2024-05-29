use std::collections::HashMap;
use sqlx::PgPool;
use crate::config::Config;

pub struct AppState {
    pub db_pool: PgPool,
    pub cfg: Config,
    pub airport_neighbours: HashMap<String, Vec<String>>
}