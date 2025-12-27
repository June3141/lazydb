//! PostgresProvider struct and connection methods

use postgres::{Client, NoTls};
use std::sync::Mutex;

use crate::config::ConnectionConfig;

use super::ProviderError;

/// PostgreSQL database provider
pub struct PostgresProvider {
    pub(super) client: Mutex<Client>,
}

impl PostgresProvider {
    /// Create a new PostgresProvider from connection configuration
    pub fn new(config: &ConnectionConfig) -> Result<Self, ProviderError> {
        let password = config.get_password().unwrap_or_default();
        let username = config.username.as_deref().unwrap_or("postgres");

        let connection_string = format!(
            "host={} port={} dbname={} user={} password={}",
            config.host, config.port, config.database, username, password
        );

        let client = Client::connect(&connection_string, NoTls)
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            client: Mutex::new(client),
        })
    }

    /// Create a new PostgresProvider from connection parameters.
    ///
    /// Uses the `postgres::Config` builder API to safely handle passwords
    /// containing special characters (like `@`, `#`, spaces, or quotes).
    pub fn connect(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, ProviderError> {
        let mut config = postgres::Config::new();
        config
            .host(host)
            .port(port)
            .dbname(database)
            .user(username)
            .password(password);

        let client = config
            .connect(NoTls)
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            client: Mutex::new(client),
        })
    }
}
