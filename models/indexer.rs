use postgres::{Client, Row};
use serde_json::Value;

use std::{convert::From, fmt::Display};

use super::ModelError;

const TABLE_NAME: &'static str = "indexer_api_indexer";

pub struct Indexer {
    pub id: i64,
    pub name: String,
    pub last_block: u64,
    pub strategy: String,
    pub short_sleep_seconds: i64,
    pub long_sleep_seconds: i64,
    pub strategy_params: Option<Value>,
    pub network_id: i64,
    pub status: String,
    pub indexer_type: String,
}

impl Indexer {
    pub fn update_last_block(
        &mut self,
        client: &mut Client,
        new_last_block: i64,
    ) -> Result<(), ModelError> {
        let query = format!("UPDATE {} SET last_block = $1 WHERE name = $2", TABLE_NAME);
        let result = client.execute(query.as_str(), &[&new_last_block, &self.name]);
        match result {
            Ok(_) => {
                self.last_block = new_last_block as u64;
                Ok(())
            },
            Err(e) => {
                Err(ModelError { reason: e.to_string() })
            }
        }
    }

    pub fn load_from_db(client: &mut Client, name: &String) -> Result<Self, ModelError> {
        let query = format!("SELECT * FROM {} WHERE name = $1", TABLE_NAME);
        match client.query(query.as_str(), &[&name]) {
            Ok(indexers_rows) => {
                if indexers_rows.len() != 1 {
                    return Err(ModelError {
                        reason: String::from(
                            "Bad amount of indexers received. Needs only 1 indexer to operate",
                        ),
                    });
                }
                let indexer_row = indexers_rows.get(0).unwrap();
                Ok(Self::from_row(indexer_row))
            }
            Err(e) => {
                return Err(ModelError {
                    reason: e.to_string(),
                })
            }
        }
    }

    pub fn refresh(&mut self, client: &mut Client) -> Result<(), ModelError> {
        match Self::load_from_db(client, &self.name) {
            Ok(fresh_from_db) => {
                *self = fresh_from_db;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get(0),
            name: row.get(1),
            last_block: row.get::<usize, i64>(2) as u64,
            strategy: row.get(3),
            short_sleep_seconds: row.get(4),
            long_sleep_seconds: row.get(5),
            strategy_params: row.get(6),
            network_id: row.get(7),
            status: row.get(8),
            indexer_type: row.get(9),
        }
    }
}

impl Display for Indexer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Indexer {}(#{}, {}) on network {} with last block at {} and strategy {} is {}", self.name, self.id,  self.indexer_type, self.network_id, self.last_block, self.strategy,  self.status)
    }
}

impl From<Row> for Indexer {
    fn from(value: Row) -> Self {
        Self::from_row(&value)
    }
}
