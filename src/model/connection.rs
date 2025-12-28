use super::schema::Table;
use crate::config::ConnectionConfig;

#[derive(Debug, Clone)]
pub struct Connection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub expanded: bool,
    pub tables: Vec<Table>,
}

impl From<ConnectionConfig> for Connection {
    fn from(config: ConnectionConfig) -> Self {
        let password = config.get_password().unwrap_or_default();
        Self {
            name: config.name,
            host: config.host,
            port: config.port,
            database: config.database,
            username: config.username.unwrap_or_default(),
            password,
            expanded: false,
            tables: Vec::new(),
        }
    }
}
