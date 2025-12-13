use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// メイン設定ファイル (~/.config/lazydb/config.yaml)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// グローバル設定
    #[serde(default)]
    pub settings: Settings,

    /// プロジェクトファイルのパスリスト
    #[serde(default)]
    pub projects: Vec<String>,
}

/// グローバル設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// デフォルトで選択するプロジェクト名
    #[serde(default)]
    pub default_project: Option<String>,

    /// UIテーマ
    #[serde(default = "default_theme")]
    pub theme: String,

    /// 行数を表示するか
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

/// プロジェクトファイル (projects/*.yaml)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    /// プロジェクトメタ情報
    pub project: ProjectConfig,

    /// 接続情報リスト
    #[serde(default)]
    pub connections: Vec<ConnectionConfig>,
}

/// プロジェクトメタ情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// プロジェクト名
    pub name: String,

    /// 説明
    #[serde(default)]
    pub description: Option<String>,

    /// 作成日時
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
}

/// 接続設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// 接続名
    pub name: String,

    /// ホスト名
    pub host: String,

    /// ポート番号
    #[serde(default = "default_port")]
    pub port: u16,

    /// データベース名
    pub database: String,

    /// ユーザー名
    #[serde(default)]
    pub username: Option<String>,

    /// パスワード（直接指定）
    #[serde(default)]
    pub password: Option<String>,

    /// パスワード（環境変数名で指定）
    #[serde(default)]
    pub password_env: Option<String>,
}

fn default_port() -> u16 {
    5432
}

impl ConnectionConfig {
    /// パスワードを取得（password_env が設定されていれば環境変数から取得）
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
        let config: Config = serde_yml::from_str(yaml).unwrap();
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
        let project_file: ProjectFile = serde_yml::from_str(yaml).unwrap();
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
