extern crate postgres;
extern crate serde_json;
extern crate web3;
extern crate log;

use std::env;
use postgres::{Client, NoTls};
mod indexer;
mod models;
use indexer::start;

pub struct DBClientError {
    pub reason: String,
}

pub fn get_env(key: &str) -> Result<String, String> {
    match env::var(key) {
        Ok(env_value) => {
            Ok(env_value)
        },
        Err(e) => {
            Err(format!("On fetching env {} occurred error: {}", key, e.to_string()))
        }
    }
}

pub fn init_db_client() -> Result<Client, String> {
    let postgres_db = get_env("POSTGRES_DB")?;
    let postgres_user = get_env("POSTGRES_USER")?;
    let postgres_password = get_env("POSTGRES_PASSWORD")?;
    let postgres_host = get_env("POSTGRES_HOST")?;
    let postgres_port = get_env("POSTGRES_PORT").unwrap_or(String::from("5432"));
    let connection_string = format!("postgresql://{}:{}@{}:{}/{}", postgres_user, postgres_password, postgres_host, postgres_port, postgres_db);
    match Client::connect(connection_string.as_str(), NoTls) {
        Ok(client) => {
            log::info!("DB Client initialized with connection url {}", connection_string);
            Ok(client)
        },
        Err(e) => {
            Err(format!("During connection {e} occurred"))
        }
    }
}

pub fn main() {
    env_logger::init();
    match init_db_client() {
        Ok(mut client) => {
            start(&mut client);
        },
        Err(e) => {
            log::warn!("{e}");
        }
    }
}
