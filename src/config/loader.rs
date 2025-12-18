use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::models::{Config, ConnectionConfig, ProjectConfig, ProjectFile, Settings};
use crate::model::QueryHistory;

/// 設定ファイルの読み込みを担当
pub struct ConfigLoader {
    /// 設定ディレクトリのベースパス (~/.config/lazydb)
    config_dir: PathBuf,
}

impl ConfigLoader {
    /// 新しい ConfigLoader を作成
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_dir()?;
        Ok(Self { config_dir })
    }

    /// カスタムの設定ディレクトリで ConfigLoader を作成（テスト用）
    pub fn with_config_dir(config_dir: PathBuf) -> Self {
        Self { config_dir }
    }

    /// 設定ディレクトリのパスを取得
    ///
    /// - Linux/macOS: ~/.config/lazydb/
    /// - Windows: %APPDATA%\lazydb\ (C:\Users\<User>\AppData\Roaming\lazydb\)
    fn get_config_dir() -> Result<PathBuf> {
        #[cfg(windows)]
        {
            // Windows: %APPDATA%\lazydb (dirs::config_dir returns %APPDATA%)
            let config_dir = dirs::config_dir()
                .context("Failed to get config directory")?
                .join("lazydb");
            Ok(config_dir)
        }

        #[cfg(not(windows))]
        {
            // Linux/macOS: ~/.config/lazydb (XDG Base Directory)
            let home = dirs::home_dir().context("Failed to get home directory")?;
            let config_dir = home.join(".config").join("lazydb");
            Ok(config_dir)
        }
    }

    /// 設定ディレクトリのパスを返す
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// メイン設定ファイルのパスを返す
    pub fn config_file_path(&self) -> PathBuf {
        self.config_dir.join("config.yaml")
    }

    /// メイン設定ファイルを読み込む
    pub fn load_config(&self) -> Result<Config> {
        let config_path = self.config_file_path();

        if !config_path.exists() {
            // 設定ファイルが存在しない場合はデフォルト値を返す
            return Ok(Config::default());
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

        let config: Config = serde_yml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

        Ok(config)
    }

    /// メイン設定ファイルを保存
    pub fn save_config(&self, config: &Config) -> Result<()> {
        // ディレクトリが存在しない場合は作成
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir).with_context(|| {
                format!(
                    "Failed to create config directory: {}",
                    self.config_dir.display()
                )
            })?;
        }

        let config_path = self.config_file_path();
        let content = serde_yml::to_string(config).context("Failed to serialize config")?;

        fs::write(&config_path, content)
            .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

        Ok(())
    }

    /// プロジェクトファイルのパスを解決
    ///
    /// - 相対パス: config_dir からの相対パスとして解決
    /// - ~/ で始まるパス: ホームディレクトリに展開
    /// - 絶対パス: そのまま使用
    pub fn resolve_project_path(&self, path: &str) -> Result<PathBuf> {
        let expanded = shellexpand::tilde(path);
        let path = Path::new(expanded.as_ref());

        if path.is_absolute() {
            Ok(path.to_path_buf())
        } else {
            Ok(self.config_dir.join(path))
        }
    }

    /// プロジェクトファイルを読み込む
    pub fn load_project_file(&self, path: &str) -> Result<ProjectFile> {
        let resolved_path = self.resolve_project_path(path)?;

        let content = fs::read_to_string(&resolved_path)
            .with_context(|| format!("Failed to read project file: {}", resolved_path.display()))?;

        let project_file: ProjectFile = serde_yml::from_str(&content).with_context(|| {
            format!("Failed to parse project file: {}", resolved_path.display())
        })?;

        Ok(project_file)
    }

    /// プロジェクトファイルを保存
    pub fn save_project_file(&self, path: &str, project_file: &ProjectFile) -> Result<()> {
        let resolved_path = self.resolve_project_path(path)?;

        // 親ディレクトリが存在しない場合は作成
        if let Some(parent) = resolved_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }

        let content =
            serde_yml::to_string(project_file).context("Failed to serialize project file")?;

        fs::write(&resolved_path, content).with_context(|| {
            format!("Failed to write project file: {}", resolved_path.display())
        })?;

        Ok(())
    }

    /// 全てのプロジェクトファイルを読み込む
    ///
    /// 読み込みに失敗したファイルは警告を返し、スキップする
    pub fn load_all_projects(&self, config: &Config) -> (Vec<ProjectFile>, Vec<String>) {
        let mut projects = Vec::new();
        let mut warnings = Vec::new();

        for path in &config.projects {
            match self.load_project_file(path) {
                Ok(project_file) => {
                    projects.push(project_file);
                }
                Err(e) => {
                    warnings.push(format!("Failed to load project '{}': {}", path, e));
                }
            }
        }

        (projects, warnings)
    }

    /// 設定ディレクトリと初期設定ファイルを作成
    ///
    /// 初回起動時（設定ファイルが存在しない場合）は、サンプルプロジェクトも作成する
    pub fn init_config_dir(&self) -> Result<bool> {
        let is_first_run = !self.config_file_path().exists();

        // ディレクトリ作成
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir).with_context(|| {
                format!(
                    "Failed to create config directory: {}",
                    self.config_dir.display()
                )
            })?;
        }

        // projects サブディレクトリ作成
        let projects_dir = self.config_dir.join("projects");
        if !projects_dir.exists() {
            fs::create_dir_all(&projects_dir).with_context(|| {
                format!(
                    "Failed to create projects directory: {}",
                    projects_dir.display()
                )
            })?;
        }

        // 初回起動時はサンプルデータを作成
        if is_first_run {
            self.create_sample_data()?;
        }

        Ok(is_first_run)
    }

    /// サンプルデータを作成（初回起動用）
    fn create_sample_data(&self) -> Result<()> {
        // サンプルプロジェクトファイルを作成
        let sample_project = ProjectFile {
            project: ProjectConfig {
                name: "Sample Project".to_string(),
                description: Some("This is a sample project to get you started.".to_string()),
                created_at: Some(chrono::Utc::now()),
            },
            connections: vec![
                ConnectionConfig {
                    name: "Local PostgreSQL".to_string(),
                    host: "localhost".to_string(),
                    port: 5432,
                    database: "postgres".to_string(),
                    username: Some("postgres".to_string()),
                    password: None,
                    password_env: Some("POSTGRES_PASSWORD".to_string()),
                },
                ConnectionConfig {
                    name: "Example MySQL".to_string(),
                    host: "localhost".to_string(),
                    port: 3306,
                    database: "example_db".to_string(),
                    username: Some("root".to_string()),
                    password: None,
                    password_env: Some("MYSQL_PASSWORD".to_string()),
                },
            ],
        };

        let sample_project_path = "projects/sample-project.yaml";
        self.save_project_file(sample_project_path, &sample_project)?;

        // メイン設定ファイルを作成（サンプルプロジェクトを参照）
        let config = Config {
            settings: Settings {
                default_project: Some("Sample Project".to_string()),
                theme: "dark".to_string(),
                show_row_count: true,
            },
            projects: vec![sample_project_path.to_string()],
        };
        self.save_config(&config)?;

        Ok(())
    }

    /// 履歴ファイルのパスを返す
    pub fn history_file_path(&self) -> PathBuf {
        self.config_dir.join("history.yaml")
    }

    /// クエリ履歴を読み込む
    pub fn load_history(&self) -> Result<QueryHistory> {
        let history_path = self.history_file_path();

        if !history_path.exists() {
            // 履歴ファイルが存在しない場合は空の履歴を返す
            return Ok(QueryHistory::new());
        }

        let content = fs::read_to_string(&history_path)
            .with_context(|| format!("Failed to read history file: {}", history_path.display()))?;

        let history: QueryHistory = serde_yml::from_str(&content)
            .with_context(|| format!("Failed to parse history file: {}", history_path.display()))?;

        Ok(history)
    }

    /// クエリ履歴を保存
    pub fn save_history(&self, history: &QueryHistory) -> Result<()> {
        // ディレクトリが存在しない場合は作成
        if !self.config_dir.exists() {
            fs::create_dir_all(&self.config_dir).with_context(|| {
                format!(
                    "Failed to create config directory: {}",
                    self.config_dir.display()
                )
            })?;
        }

        let history_path = self.history_file_path();
        let content = serde_yml::to_string(history).context("Failed to serialize history")?;

        fs::write(&history_path, content)
            .with_context(|| format!("Failed to write history file: {}", history_path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_loader() -> (ConfigLoader, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let loader = ConfigLoader::with_config_dir(temp_dir.path().to_path_buf());
        (loader, temp_dir)
    }

    #[test]
    fn test_resolve_relative_path() {
        let (loader, _temp_dir) = create_test_loader();
        let resolved = loader.resolve_project_path("projects/test.yaml").unwrap();
        assert!(resolved.ends_with("projects/test.yaml"));
    }

    #[test]
    fn test_resolve_absolute_path() {
        let (loader, _temp_dir) = create_test_loader();
        let resolved = loader
            .resolve_project_path("/absolute/path/test.yaml")
            .unwrap();
        assert_eq!(resolved, PathBuf::from("/absolute/path/test.yaml"));
    }

    #[test]
    fn test_resolve_tilde_path() {
        let (loader, _temp_dir) = create_test_loader();
        let resolved = loader.resolve_project_path("~/projects/test.yaml").unwrap();
        let home = dirs::home_dir().expect("Failed to get home directory");
        assert!(resolved.starts_with(&home));
    }

    #[test]
    fn test_load_config_default() {
        let (loader, _temp_dir) = create_test_loader();
        let config = loader.load_config().unwrap();
        assert!(config.projects.is_empty());
    }

    #[test]
    fn test_save_and_load_config() {
        let (loader, _temp_dir) = create_test_loader();

        let config = Config {
            settings: Default::default(),
            projects: vec!["projects/test.yaml".to_string()],
        };

        loader.save_config(&config).unwrap();
        let loaded = loader.load_config().unwrap();

        assert_eq!(loaded.projects.len(), 1);
        assert_eq!(loaded.projects[0], "projects/test.yaml");
    }

    #[test]
    fn test_init_config_dir_first_run() {
        let (loader, temp_dir) = create_test_loader();
        let is_first_run = loader.init_config_dir().unwrap();

        // 初回起動時は true を返す
        assert!(is_first_run);

        // ディレクトリとファイルが作成されている
        assert!(temp_dir.path().join("projects").exists());
        assert!(temp_dir.path().join("config.yaml").exists());

        // サンプルプロジェクトが作成されている
        assert!(temp_dir
            .path()
            .join("projects/sample-project.yaml")
            .exists());

        // 設定ファイルにサンプルプロジェクトが登録されている
        let config = loader.load_config().unwrap();
        assert_eq!(config.projects.len(), 1);
        assert_eq!(config.projects[0], "projects/sample-project.yaml");
        assert_eq!(
            config.settings.default_project,
            Some("Sample Project".to_string())
        );
    }

    #[test]
    fn test_init_config_dir_second_run() {
        let (loader, _temp_dir) = create_test_loader();

        // 初回起動
        let is_first_run = loader.init_config_dir().unwrap();
        assert!(is_first_run);

        // 2回目の起動
        let is_first_run = loader.init_config_dir().unwrap();
        assert!(!is_first_run);

        // ファイルは上書きされていない（サンプルプロジェクトはそのまま）
        let config = loader.load_config().unwrap();
        assert_eq!(config.projects.len(), 1);
    }

    #[test]
    fn test_sample_project_content() {
        let (loader, _temp_dir) = create_test_loader();
        loader.init_config_dir().unwrap();

        // サンプルプロジェクトの内容を確認
        let project = loader
            .load_project_file("projects/sample-project.yaml")
            .unwrap();

        assert_eq!(project.project.name, "Sample Project");
        assert!(project.project.description.is_some());
        assert_eq!(project.connections.len(), 2);

        // PostgreSQL 接続
        let pg_conn = &project.connections[0];
        assert_eq!(pg_conn.name, "Local PostgreSQL");
        assert_eq!(pg_conn.host, "localhost");
        assert_eq!(pg_conn.port, 5432);

        // MySQL 接続
        let mysql_conn = &project.connections[1];
        assert_eq!(mysql_conn.name, "Example MySQL");
        assert_eq!(mysql_conn.host, "localhost");
        assert_eq!(mysql_conn.port, 3306);
    }
}
