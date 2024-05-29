use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String
}

impl Config {
    pub fn init() -> Config {
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let server_addr = std::env::var("SERVER_ADDR").expect("SERVER_ADDR must be set");

        Config {
            database_url,
            server_addr
        }
    }
}