use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Main configuration file (~/.config/lazydb/config.yaml)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Global settings
    #[serde(default)]
    pub settings: Settings,

    /// List of project file paths
    #[serde(default)]
    pub projects: Vec<String>,
}

/// Global settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Default project name to select
    #[serde(default)]
    pub default_project: Option<String>,

    /// UI theme
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Whether to show row count
    #[serde(default = "default_true")]
    pub show_row_count: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_project: None,
            theme: default_theme(),
            show_row_count: true,
        }
    }
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_true() -> bool {
    true
}

/// Project file (projects/*.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    /// Project metadata
    pub project: ProjectConfig,

    /// List of connection configurations
    #[serde(default)]
    pub connections: Vec<ConnectionConfig>,
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name
    pub name: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
}

/// Connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Connection name
    pub name: String,

    /// Host name
    pub host: String,

    /// Port number
    #[serde(default = "default_port")]
    pub port: u16,

    /// Database name
    pub database: String,

    /// Username
    #[serde(default)]
    pub username: Option<String>,

    /// Password (direct value)
    #[serde(default)]
    pub password: Option<String>,

    /// Password (specified by environment variable name)
    #[serde(default)]
    pub password_env: Option<String>,
}

fn default_port() -> u16 {
    5432
}

impl ConnectionConfig {
    /// Get password (retrieves from environment variable if password_env is set)
    ///
    /// Priority:
    /// 1. If `password_env` is set and the environment variable exists, use that value
    /// 2. Otherwise, use the `password` field if set
    /// 3. Otherwise, return None
    pub fn get_password(&self) -> Option<String> {
        // Try environment variable first if specified
        if let Some(env_name) = &self.password_env {
            if let Ok(value) = std::env::var(env_name) {
                return Some(value);
            }
        }
        // Fall back to direct password
        self.password.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_deserialize() {
        let yaml = r#"
settings:
  default_project: my-project
  theme: dark
  show_row_count: true
projects:
  - projects/my-project.yaml
  - ~/work/other-project.yaml
"#;
        let config: Config = serde_norway::from_str(yaml).unwrap();
        assert_eq!(
            config.settings.default_project,
            Some("my-project".to_string())
        );
        assert_eq!(config.projects.len(), 2);
    }

    #[test]
    fn test_project_file_deserialize() {
        let yaml = r#"
project:
  name: My Project
  description: Test project

connections:
  - name: Production
    host: prod.example.com
    port: 5432
    database: mydb
    username: dbuser
    password_env: PROD_PASSWORD

  - name: Development
    host: localhost
    database: mydb_dev
    username: dev
    password: dev123
"#;
        let project_file: ProjectFile = serde_norway::from_str(yaml).unwrap();
        assert_eq!(project_file.project.name, "My Project");
        assert_eq!(project_file.connections.len(), 2);
        assert_eq!(project_file.connections[0].port, 5432);
        assert_eq!(project_file.connections[1].port, 5432); // default
    }

    #[test]
    fn test_connection_config_password() {
        let conn = ConnectionConfig {
            name: "test".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: Some("user".to_string()),
            password: Some("direct_password".to_string()),
            password_env: None,
        };
        assert_eq!(conn.get_password(), Some("direct_password".to_string()));
    }
}
