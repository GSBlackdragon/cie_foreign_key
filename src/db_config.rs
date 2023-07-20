use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    pub sql_server_host: String,
    pub sql_server_port: u16,
    pub sql_server_user: String,
    pub sql_server_pass: String,
    pub sql_server_db: String,

    pub postgres_host: String,
    pub postgres_port: String,
    pub postgres_user: String,
    pub postgres_pass: String,
    pub postgres_db: String
}