use super::schema::Table;
use crate::config::ConnectionConfig;

#[derive(Debug, Clone)]
pub struct Connection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub expanded: bool,
    pub tables: Vec<Table>,
}

impl From<ConnectionConfig> for Connection {
    fn from(config: ConnectionConfig) -> Self {
        Self {
            name: config.name,
            host: config.host,
            port: config.port,
            database: config.database,
            expanded: false,
            tables: Vec::new(),
        }
    }
}
