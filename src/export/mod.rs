// TODO: Remove #[allow(dead_code)] once export is integrated with UI
#![allow(dead_code)]

mod csv;
mod error;
mod json;

pub use error::ExportError;

use std::path::Path;

use crate::model::QueryResult;

/// エクスポート形式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Csv,
    Json,
}

/// 文字エンコーディング
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Encoding {
    #[default]
    Utf8,
    ShiftJis,
    EucJp,
}

/// エクスポート設定
#[derive(Debug, Clone)]
pub struct ExportConfig {
    pub format: ExportFormat,
    pub encoding: Encoding,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Csv,
            encoding: Encoding::Utf8,
        }
    }
}

/// クエリ結果をファイルにエクスポートする
pub fn export_to_file(
    result: &QueryResult,
    path: &Path,
    config: &ExportConfig,
) -> anyhow::Result<()> {
    match config.format {
        ExportFormat::Csv => csv::export(result, path, config.encoding),
        ExportFormat::Json => json::export(result, path, config.encoding),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_query_result() -> QueryResult {
        QueryResult {
            columns: vec!["id".to_string(), "name".to_string(), "email".to_string()],
            rows: vec![
                vec![
                    "1".to_string(),
                    "Alice".to_string(),
                    "alice@example.com".to_string(),
                ],
                vec![
                    "2".to_string(),
                    "Bob".to_string(),
                    "bob@example.com".to_string(),
                ],
                vec![
                    "3".to_string(),
                    "Charlie".to_string(),
                    "charlie@example.com".to_string(),
                ],
            ],
            execution_time_ms: 100,
            total_rows: 3,
        }
    }

    #[test]
    fn test_export_csv_utf8() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("output.csv");
        let result = create_test_query_result();
        let config = ExportConfig {
            format: ExportFormat::Csv,
            encoding: Encoding::Utf8,
        };

        export_to_file(&result, &path, &config).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let expected = "id,name,email\n1,Alice,alice@example.com\n2,Bob,bob@example.com\n3,Charlie,charlie@example.com\n";
        assert_eq!(content, expected);
    }

    #[test]
    fn test_export_csv_with_special_characters() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("output.csv");
        let result = QueryResult {
            columns: vec!["id".to_string(), "description".to_string()],
            rows: vec![
                vec!["1".to_string(), "Hello, World".to_string()],
                vec!["2".to_string(), "Line with \"quotes\"".to_string()],
                vec!["3".to_string(), "Line\nwith\nnewlines".to_string()],
            ],
            execution_time_ms: 50,
            total_rows: 3,
        };
        let config = ExportConfig {
            format: ExportFormat::Csv,
            encoding: Encoding::Utf8,
        };

        export_to_file(&result, &path, &config).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        // CSVでは、カンマ・引用符・改行を含む値はダブルクォートで囲む
        assert!(content.contains("\"Hello, World\""));
        assert!(content.contains("\"Line with \"\"quotes\"\"\""));
        assert!(content.contains("\"Line\nwith\nnewlines\""));
    }

    #[test]
    fn test_export_json_utf8() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("output.json");
        let result = create_test_query_result();
        let config = ExportConfig {
            format: ExportFormat::Json,
            encoding: Encoding::Utf8,
        };

        export_to_file(&result, &path, &config).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);

        // 最初の行を検証
        assert_eq!(array[0]["id"], "1");
        assert_eq!(array[0]["name"], "Alice");
        assert_eq!(array[0]["email"], "alice@example.com");
    }

    #[test]
    fn test_export_json_with_japanese() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("output.json");
        let result = QueryResult {
            columns: vec!["id".to_string(), "名前".to_string()],
            rows: vec![
                vec!["1".to_string(), "田中太郎".to_string()],
                vec!["2".to_string(), "鈴木花子".to_string()],
            ],
            execution_time_ms: 50,
            total_rows: 2,
        };
        let config = ExportConfig {
            format: ExportFormat::Json,
            encoding: Encoding::Utf8,
        };

        export_to_file(&result, &path, &config).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        let array = parsed.as_array().unwrap();

        assert_eq!(array[0]["名前"], "田中太郎");
        assert_eq!(array[1]["名前"], "鈴木花子");
    }

    #[test]
    fn test_export_csv_shift_jis() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("output_sjis.csv");
        let result = QueryResult {
            columns: vec!["id".to_string(), "名前".to_string()],
            rows: vec![vec!["1".to_string(), "田中太郎".to_string()]],
            execution_time_ms: 50,
            total_rows: 1,
        };
        let config = ExportConfig {
            format: ExportFormat::Csv,
            encoding: Encoding::ShiftJis,
        };

        export_to_file(&result, &path, &config).unwrap();

        // Shift_JISでエンコードされていることを確認
        let bytes = fs::read(&path).unwrap();
        // UTF-8として読めないはず（日本語部分）
        // Shift_JISの「田中太郎」をデコードして確認
        let (decoded, _, _) = encoding_rs::SHIFT_JIS.decode(&bytes);
        assert!(decoded.contains("田中太郎"));
    }

    #[test]
    fn test_export_csv_euc_jp() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("output_eucjp.csv");
        let result = QueryResult {
            columns: vec!["id".to_string(), "名前".to_string()],
            rows: vec![vec!["1".to_string(), "田中太郎".to_string()]],
            execution_time_ms: 50,
            total_rows: 1,
        };
        let config = ExportConfig {
            format: ExportFormat::Csv,
            encoding: Encoding::EucJp,
        };

        export_to_file(&result, &path, &config).unwrap();

        // EUC-JPでエンコードされていることを確認
        let bytes = fs::read(&path).unwrap();
        let (decoded, _, _) = encoding_rs::EUC_JP.decode(&bytes);
        assert!(decoded.contains("田中太郎"));
    }

    #[test]
    fn test_export_empty_result() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("empty.csv");
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![],
            execution_time_ms: 10,
            total_rows: 0,
        };
        let config = ExportConfig {
            format: ExportFormat::Csv,
            encoding: Encoding::Utf8,
        };

        export_to_file(&result, &path, &config).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        // ヘッダーのみ
        assert_eq!(content, "id,name\n");
    }

    #[test]
    fn test_export_json_empty_result() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("empty.json");
        let result = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![],
            execution_time_ms: 10,
            total_rows: 0,
        };
        let config = ExportConfig {
            format: ExportFormat::Json,
            encoding: Encoding::Utf8,
        };

        export_to_file(&result, &path, &config).unwrap();

        let content = fs::read_to_string(&path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert!(parsed.is_array());
        assert_eq!(parsed.as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_export_config_default() {
        let config = ExportConfig::default();
        assert_eq!(config.format, ExportFormat::Csv);
        assert_eq!(config.encoding, Encoding::Utf8);
    }

    #[test]
    fn test_export_to_nested_directory() {
        let temp_dir = TempDir::new().unwrap();
        let nested_path = temp_dir
            .path()
            .join("subdir")
            .join("deep")
            .join("output.csv");

        // 親ディレクトリを作成
        fs::create_dir_all(nested_path.parent().unwrap()).unwrap();

        let result = create_test_query_result();
        let config = ExportConfig::default();

        export_to_file(&result, &nested_path, &config).unwrap();

        assert!(nested_path.exists());
    }
}
