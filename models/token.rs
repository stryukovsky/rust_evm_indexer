use postgres::{Client, Row};

use super::{ModelError, Indexer, TokenType};

const TABLE_NAME: &'static str = "indexer_api_token";
const INDEXER_TOKEN_TABLE_NAME: &'static str = "indexer_api_indexer_watched_tokens";

use rust_decimal::Decimal;

pub struct Token {
    pub id: i64,
    pub address: String,
    pub name: String,
    pub strategy: String,
    pub token_type: TokenType,
    pub total_supply: Decimal,
    pub volume: Decimal,
    pub network_id: i64,
}

impl Token {
    pub fn load_from_db(
        client: &mut Client,
        network_id: i64,
        address: String,
    ) -> Result<Self, ModelError> {
        let query = format!(
            "SELECT * FROM {} WHERE network_id = $1 AND address = $2",
            TABLE_NAME
        );
        match client.query(query.as_str(), &[&network_id, &address]) {
            Ok(tokens_rows) => {
                if tokens_rows.len() != 1 {
                    Err(ModelError {
                        reason: format!(
                            "Too many tokens received, expected 1, having {}",
                            tokens_rows.len()
                        ),
                    })
                } else {
                    Ok(Self::from_row(tokens_rows.get(0).unwrap()))
                }
            }
            Err(e) => Err(ModelError {
                reason: e.to_string(),
            }),
        }
    }

    pub fn load_tokens_from_db_by_indexer(client: &mut Client, indexer: &Indexer) -> Result<Vec<Self>, ModelError> {
        let query = format!("SELECT * FROM {TABLE_NAME} INNER JOIN {INDEXER_TOKEN_TABLE_NAME} ON {TABLE_NAME}.id = {INDEXER_TOKEN_TABLE_NAME}.token_id WHERE {INDEXER_TOKEN_TABLE_NAME}.indexer_id = $1");
        match client.query(query.as_str(), &[&indexer.id]) {
            Ok(rows) => {
                let result = rows.iter().map(Self::from_row).collect();
                Ok(result)
            },
            Err(e) => {
                Err(ModelError { reason: e.to_string() })
            }
        }

    }

    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get(0),
            address: row.get(1),
            name: row.get(2),
            strategy: row.get(3),
            token_type: TokenType::from(&row.get::<usize, String>(4)),
            total_supply: row.get(5),
            volume: row.get(6),
            network_id: row.get(7),
        }
    }
}
