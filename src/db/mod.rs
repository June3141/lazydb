use crate::config::{Connection, DatabaseType};
use async_trait::async_trait;

#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    async fn connect(&self) -> anyhow::Result<()>;
    async fn disconnect(&self) -> anyhow::Result<()>;
    async fn execute_query(&self, query: &str) -> anyhow::Result<QueryResult>;
    async fn get_tables(&self) -> anyhow::Result<Vec<String>>;
}

#[derive(Debug)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

pub struct ConnectionManager {
    // Store connections as strings for now
    connections: std::collections::HashMap<String, String>,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: std::collections::HashMap::new(),
        }
    }

    pub async fn add_connection(&mut self, connection: &Connection) -> anyhow::Result<()> {
        match connection.database_type {
            DatabaseType::PostgreSQL => {
                // TODO: Implement PostgreSQL connection
                anyhow::bail!("PostgreSQL support not implemented yet");
            }
            DatabaseType::MySQL => {
                // TODO: Implement MySQL connection
                anyhow::bail!("MySQL support not implemented yet");
            }
            DatabaseType::SQLite => {
                // TODO: Implement SQLite connection
                anyhow::bail!("SQLite support not implemented yet");
            }
            DatabaseType::MongoDB => {
                // TODO: Implement MongoDB connection
                anyhow::bail!("MongoDB support not implemented yet");
            }
        }
    }
}
