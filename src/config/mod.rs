use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod security;
pub use security::{PasswordManager, SecurePassword};

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub connections: Vec<Connection>,
    pub projects: Vec<Project>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub connection_groups: Vec<ConnectionGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_connection_group: Option<String>,
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
    pub secure_password: Option<SecurePassword>,
    pub database_name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionGroup {
    pub name: String,
    pub connections: Vec<String>,
    pub description: Option<String>,
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
            let config: Config = serde_yaml::from_str(&content)?;
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

        let content = serde_yaml::to_string(self)?;
        std::fs::write(&config_path, content)?;

        Ok(())
    }

    fn config_path() -> anyhow::Result<PathBuf> {
        let home_dir =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

        Ok(home_dir.join(".config").join("lazydb").join("config.yaml"))
    }

    pub fn add_connection(&mut self, connection: Connection) {
        self.connections.push(connection);
    }

    pub fn add_project(&mut self, project: Project) {
        self.projects.push(project);
    }

    pub fn add_connection_group(&mut self, group: ConnectionGroup) {
        self.connection_groups.push(group);
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        for connection in &self.connections {
            connection.validate()?;
        }

        for group in &self.connection_groups {
            group.validate(&self.connections)?;
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            connections: Vec::new(),
            projects: Vec::new(),
            connection_groups: Vec::new(),
            default_connection_group: None,
        }
    }
}

impl Connection {
    pub fn new(
        id: String,
        name: String,
        database_type: DatabaseType,
        host: String,
        port: u16,
        username: String,
        database_name: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            database_type,
            host,
            port,
            username,
            secure_password: None,
            database_name,
            tags: Vec::new(),
        }
    }

    pub fn set_password(&mut self, password: &str) -> anyhow::Result<()> {
        self.secure_password = Some(PasswordManager::encrypt_password(password)?);
        Ok(())
    }

    pub fn verify_password(&self, password: &str) -> anyhow::Result<bool> {
        match &self.secure_password {
            Some(secure_password) => PasswordManager::verify_password(password, secure_password),
            None => Ok(false),
        }
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        if self.id.is_empty() {
            return Err(anyhow::anyhow!("Connection ID cannot be empty"));
        }

        if self.name.is_empty() {
            return Err(anyhow::anyhow!("Connection name cannot be empty"));
        }

        if self.host.is_empty() {
            return Err(anyhow::anyhow!("Host cannot be empty"));
        }

        if self.port == 0 {
            return Err(anyhow::anyhow!("Port must be greater than 0"));
        }

        if self.username.is_empty() {
            return Err(anyhow::anyhow!("Username cannot be empty"));
        }

        Ok(())
    }
}

impl ConnectionGroup {
    pub fn new(name: String, connections: Vec<String>) -> Self {
        Self {
            name,
            connections,
            description: None,
        }
    }

    pub fn validate(&self, all_connections: &[Connection]) -> anyhow::Result<()> {
        if self.name.is_empty() {
            return Err(anyhow::anyhow!("Connection group name cannot be empty"));
        }

        for connection_id in &self.connections {
            if !all_connections.iter().any(|c| &c.id == connection_id) {
                return Err(anyhow::anyhow!(
                    "Connection group '{}' references non-existent connection '{}'",
                    self.name,
                    connection_id
                ));
            }
        }

        Ok(())
    }
}
