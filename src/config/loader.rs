use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::models::{Config, ProjectFile};

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
    pub fn init_config_dir(&self) -> Result<()> {
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

        // config.yaml が存在しない場合は作成
        let config_path = self.config_file_path();
        if !config_path.exists() {
            let default_config = Config::default();
            self.save_config(&default_config)?;
        }

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
    fn test_init_config_dir() {
        let (loader, temp_dir) = create_test_loader();
        loader.init_config_dir().unwrap();

        assert!(temp_dir.path().join("projects").exists());
        assert!(temp_dir.path().join("config.yaml").exists());
    }
}
