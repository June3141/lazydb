use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Query execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QueryStatus {
    /// Execution succeeded
    Success,
    /// Execution failed
    Error(String),
}

/// Query history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Executed query string
    pub query: String,

    /// Execution timestamp
    pub executed_at: DateTime<Utc>,

    /// Execution time in milliseconds
    #[serde(default)]
    pub execution_time_ms: Option<u64>,

    /// Number of rows returned (only on success)
    #[serde(default)]
    pub row_count: Option<usize>,

    /// Connection name
    pub connection_name: String,

    /// Database name
    pub database: String,

    /// Execution status
    pub status: QueryStatus,
}

impl HistoryEntry {
    /// Create a history entry from a successful execution
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

    /// Create a history entry from a failed execution
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

    /// Check if the query succeeded
    pub fn is_success(&self) -> bool {
        matches!(self.status, QueryStatus::Success)
    }
}

/// Query history manager
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryHistory {
    /// List of history entries (newest first)
    #[serde(default)]
    pub entries: Vec<HistoryEntry>,

    /// Maximum number of entries
    #[serde(default = "default_max_entries")]
    pub max_entries: usize,
}

fn default_max_entries() -> usize {
    100
}

impl QueryHistory {
    /// Create a new empty history
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            max_entries: default_max_entries(),
        }
    }

    /// Create with a specified maximum number of entries
    #[allow(dead_code)]
    pub fn with_max_entries(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_entries,
        }
    }

    /// Add a history entry
    pub fn add(&mut self, entry: HistoryEntry) {
        // Avoid duplicate of the most recent entry (when the same query is executed consecutively)
        if let Some(last) = self.entries.first() {
            if last.query == entry.query
                && last.connection_name == entry.connection_name
                && last.database == entry.database
            {
                // Update with the latest execution for the same query
                self.entries[0] = entry;
                return;
            }
        }

        // Insert at the beginning
        self.entries.insert(0, entry);

        // Remove old entries if exceeding max
        if self.entries.len() > self.max_entries {
            self.entries.truncate(self.max_entries);
        }
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get the number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Get entry at the specified index
    pub fn get(&self, index: usize) -> Option<&HistoryEntry> {
        self.entries.get(index)
    }

    /// Get only successful queries
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
        // Only the latest 3 entries remain
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

        // Same query should not be duplicated
        assert_eq!(history.len(), 1);
        // Should be updated with the latest execution info
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
