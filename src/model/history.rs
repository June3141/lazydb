use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// クエリ実行結果のステータス
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryStatus {
    /// 実行成功
    Success,
    /// 実行失敗
    Error(String),
}

/// クエリ履歴エントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// 実行したクエリ文字列
    pub query: String,

    /// 実行日時
    pub executed_at: DateTime<Utc>,

    /// 実行時間（ミリ秒）
    #[serde(default)]
    pub execution_time_ms: Option<u64>,

    /// 取得した行数（成功時のみ）
    #[serde(default)]
    pub row_count: Option<usize>,

    /// 接続名
    pub connection_name: String,

    /// データベース名
    pub database: String,

    /// 実行ステータス
    pub status: QueryStatus,
}

impl HistoryEntry {
    /// 成功した実行から履歴エントリを作成
    pub fn success(
        query: impl Into<String>,
        connection_name: impl Into<String>,
        database: impl Into<String>,
        execution_time_ms: u64,
        row_count: usize,
    ) -> Self {
        Self {
            query: query.into(),
            executed_at: Utc::now(),
            execution_time_ms: Some(execution_time_ms),
            row_count: Some(row_count),
            connection_name: connection_name.into(),
            database: database.into(),
            status: QueryStatus::Success,
        }
    }

    /// 失敗した実行から履歴エントリを作成
    pub fn error(
        query: impl Into<String>,
        connection_name: impl Into<String>,
        database: impl Into<String>,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            query: query.into(),
            executed_at: Utc::now(),
            execution_time_ms: None,
            row_count: None,
            connection_name: connection_name.into(),
            database: database.into(),
            status: QueryStatus::Error(error_message.into()),
        }
    }

    /// クエリが成功したかどうか
    pub fn is_success(&self) -> bool {
        matches!(self.status, QueryStatus::Success)
    }
}

/// クエリ履歴の管理
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryHistory {
    /// 履歴エントリのリスト（新しいものが先頭）
    #[serde(default)]
    pub entries: Vec<HistoryEntry>,

    /// 最大履歴数
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
}

fn default_max_entries() -> usize {
    100
}

impl QueryHistory {
    /// 新しい空の履歴を作成
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: default_max_entries(),
        }
    }

    /// 最大履歴数を指定して作成
    #[allow(dead_code)]
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// 履歴エントリを追加
    pub fn add(&mut self, entry: HistoryEntry) {
        // 重複する最後のエントリを避ける（同じクエリが連続で実行された場合）
        if let Some(last) = self.entries.first() {
            if last.query == entry.query
                && last.connection_name == entry.connection_name
                && last.database == entry.database
            {
                // 同じクエリの場合は最新のものに更新
                self.entries[0] = entry;
                return;
            }
        }

        // 先頭に追加
        self.entries.insert(0, entry);

        // 最大数を超えた場合は古いものを削除
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
    }

    /// 履歴をクリア
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 履歴が空かどうか
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 履歴の数を取得
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 指定インデックスのエントリを取得
    pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
        self.entries.get(index)
    }

    /// 成功したクエリのみをフィルタして取得
    #[allow(dead_code)]
    pub fn successful_queries(&self) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.iter().filter(|e| e.is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_entry_success() {
        let entry = HistoryEntry::success("SELECT * FROM users", "Production", "mydb", 100, 50);
        assert!(entry.is_success());
        assert_eq!(entry.row_count, Some(50));
        assert_eq!(entry.execution_time_ms, Some(100));
    }

    #[test]
    fn test_history_entry_error() {
        let entry = HistoryEntry::error(
            "SELECT * FROM invalid",
            "Production",
            "mydb",
            "Table not found",
        );
        assert!(!entry.is_success());
        assert!(matches!(entry.status, QueryStatus::Error(_)));
    }

    #[test]
    fn test_query_history_add() {
        let mut history = QueryHistory::new();

        history.add(HistoryEntry::success("SELECT 1", "conn1", "db1", 10, 1));
        history.add(HistoryEntry::success("SELECT 2", "conn1", "db1", 20, 1));

        assert_eq!(history.len(), 2);
        assert_eq!(history.get(0).unwrap().query, "SELECT 2");
        assert_eq!(history.get(1).unwrap().query, "SELECT 1");
    }

    #[test]
    fn test_query_history_max_entries() {
        let mut history = QueryHistory::with_max_entries(3);

        for i in 0..5 {
            history.add(HistoryEntry::success(
                format!("SELECT {}", i),
                "conn",
                "db",
                10,
                1,
            ));
        }

        assert_eq!(history.len(), 3);
        // 最新の3つが残っている
        assert_eq!(history.get(0).unwrap().query, "SELECT 4");
        assert_eq!(history.get(2).unwrap().query, "SELECT 2");
    }

    #[test]
    fn test_query_history_dedup() {
        let mut history = QueryHistory::new();

        history.add(HistoryEntry::success(
            "SELECT * FROM users",
            "conn",
            "db",
            10,
            5,
        ));
        history.add(HistoryEntry::success(
            "SELECT * FROM users",
            "conn",
            "db",
            20,
            10,
        ));

        // 同じクエリは重複しない
        assert_eq!(history.len(), 1);
        // 最新の実行情報で更新されている
        assert_eq!(history.get(0).unwrap().row_count, Some(10));
    }

    #[test]
    fn test_query_history_clear() {
        let mut history = QueryHistory::new();
        history.add(HistoryEntry::success("SELECT 1", "conn", "db", 10, 1));
        history.clear();
        assert!(history.is_empty());
    }
}
