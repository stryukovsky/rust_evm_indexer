use postgres::{Client, Row};

use super::ModelError;

const TABLE_NAME: &'static str = "indexer_api_network";

pub struct Network {
    pub id: i64,
    pub chain_id: i64,
    pub name: String,
    pub rpc_url: String,
    pub max_step: u64,
    pub network_type: String,
    pub need_poa: bool,
    pub explorer_url: String,
}

impl Network {
    pub fn load_from_db(client: &mut Client, network_id: i64) -> Result<Self, ModelError> {
        let query = format!("SELECT * FROM {} WHERE id = $1", TABLE_NAME);
        match client.query(query.as_str(), &[&network_id]) {
            Ok(networks_rows) => {
                if networks_rows.len() != 1 {
                    return Err(ModelError {
                        reason: format!(
                            "Bad amount of networks received. Needs only 1 network, has {}.",
                            networks_rows.len(),
                        ),
                    });
                }
                let network_row = networks_rows.get(0).unwrap();
                Ok(Self::from_row(network_row))
            }
            Err(e) => {
                return Err(ModelError {
                    reason: e.to_string(),
                })
            }
        }
    }

    pub fn from_row(row: &Row) -> Self {
        Self {
            id: row.get(0),
            chain_id: row.get(1),
            name: row.get(2),
            rpc_url: row.get(3),
            max_step: row.get::<usize, i64>(4) as u64,
            network_type: row.get(5),
            need_poa: row.get(6),
            explorer_url: row.get(7),
        }
    }
}
