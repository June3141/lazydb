use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub connections: Vec<Connection>,
    pub projects: Vec<Project>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub database_type: DatabaseType,
    pub host: String,
    pub port: u16,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    pub database_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub connection_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
    MongoDB,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }

    fn config_path() -> anyhow::Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
        
        Ok(config_dir.join("lazydb").join("config.json"))
    }

    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    pub fn add_project(&mut self, project: Project) {
        self.projects.push(project);
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            connections: Vec::new(),
            projects: Vec::new(),
        }
    }
}