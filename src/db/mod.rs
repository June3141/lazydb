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
    // 今のところは接続を一時的に文字列として保存
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
                // TODO: PostgreSQL接続の実装
                anyhow::bail!("PostgreSQL support not implemented yet");
            }
            DatabaseType::MySQL => {
                // TODO: MySQL接続の実装
                anyhow::bail!("MySQL support not implemented yet");
            }
            DatabaseType::SQLite => {
                // TODO: SQLite接続の実装
                anyhow::bail!("SQLite support not implemented yet");
            }
            DatabaseType::MongoDB => {
                // TODO: MongoDB接続の実装
                anyhow::bail!("MongoDB support not implemented yet");
            }
        }
    }
}