use crate::model::schema::{Column, Constraint, ForeignKey, Index, Table};
use crate::model::QueryResult;

/// Supported database types
#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
    MariaDB,
    // Future: add more database types
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::PostgreSQL => write!(f, "PostgreSQL"),
            DatabaseType::MySQL => write!(f, "MySQL"),
            DatabaseType::SQLite => write!(f, "SQLite"),
            DatabaseType::MariaDB => write!(f, "MariaDB"),
        }
    }
}

/// Database provider trait - extensible architecture for multiple DB support
///
/// This trait defines the interface for database metadata retrieval.
/// Implement this trait to add support for new database types.
///
/// # Responsibility Design
///
/// ## Required Methods (implementors must provide)
/// - `get_table_details`: The **canonical source** for complete table metadata.
///   Returns a fully populated `Table` including columns, indexes, foreign keys,
///   and constraints. Implementors should optimize this for single-query retrieval.
///
/// ## Convenience Methods (have default implementations)
/// - `get_columns`, `get_indexes`, `get_foreign_keys`, `get_constraints`:
///   These delegate to `get_table_details` by default. Override only if you need
///   optimized partial queries for specific use cases.
///
/// # Example
/// ```ignore
/// struct PostgresProvider { /* connection details */ }
///
/// impl DatabaseProvider for PostgresProvider {
///     fn database_type(&self) -> DatabaseType { DatabaseType::PostgreSQL }
///
///     fn get_table_details(&self, table_name: &str, schema: Option<&str>)
///         -> Result<Table, ProviderError> {
///         // Single optimized query to fetch all table metadata
///     }
///
///     // get_columns, get_indexes, etc. automatically work via defaults
/// }
/// ```
pub trait DatabaseProvider: Send + Sync {
    /// Returns the type of database this provider handles
    fn database_type(&self) -> DatabaseType;

    /// Get list of schemas/databases
    fn get_schemas(&self) -> Result<Vec<String>, ProviderError>;

    /// Get list of tables in a schema
    fn get_tables(&self, schema: Option<&str>) -> Result<Vec<Table>, ProviderError>;

    /// Get detailed table information including columns, indexes, foreign keys, and constraints.
    ///
    /// This is the **canonical source** for table metadata. Implementors should provide
    /// a complete `Table` struct with all fields populated. The individual accessor methods
    /// (`get_columns`, `get_indexes`, etc.) delegate to this method by default.
    fn get_table_details(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Table, ProviderError>;

    /// Get columns for a table.
    ///
    /// Default implementation delegates to `get_table_details`.
    /// Override only if you need an optimized partial query.
    fn get_columns(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Column>, ProviderError> {
        Ok(self.get_table_details(table_name, schema)?.columns)
    }

    /// Get indexes for a table.
    ///
    /// Default implementation delegates to `get_table_details`.
    /// Override only if you need an optimized partial query.
    fn get_indexes(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Index>, ProviderError> {
        Ok(self.get_table_details(table_name, schema)?.indexes)
    }

    /// Get foreign keys for a table.
    ///
    /// Default implementation delegates to `get_table_details`.
    /// Override only if you need an optimized partial query.
    fn get_foreign_keys(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Vec<ForeignKey>, ProviderError> {
        Ok(self.get_table_details(table_name, schema)?.foreign_keys)
    }

    /// Get constraints for a table.
    ///
    /// Default implementation delegates to `get_table_details`.
    /// Override only if you need an optimized partial query.
    fn get_constraints(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Constraint>, ProviderError> {
        Ok(self.get_table_details(table_name, schema)?.constraints)
    }

    /// Execute a query and return results
    fn execute_query(&self, query: &str) -> Result<QueryResult, ProviderError>;

    /// Get table row count
    fn get_row_count(&self, table_name: &str, schema: Option<&str>)
        -> Result<usize, ProviderError>;

    /// Get table size in bytes
    fn get_table_size(&self, table_name: &str, schema: Option<&str>) -> Result<u64, ProviderError>;

    /// Test the connection
    fn test_connection(&self) -> Result<(), ProviderError>;

    /// Get database version information
    fn get_version(&self) -> Result<String, ProviderError>;
}

/// Provider error types
#[derive(Debug, Clone)]
pub enum ProviderError {
    ConnectionFailed(String),
    QueryFailed(String),
    NotFound(String),
    PermissionDenied(String),
    Timeout(String),
    InvalidConfiguration(String),
    NotImplemented(String),
    /// Internal errors such as mutex poisoning or other synchronization issues
    InternalError(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            ProviderError::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
            ProviderError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ProviderError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            ProviderError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            ProviderError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            ProviderError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            ProviderError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for ProviderError {}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================
    // 接続エラーのテスト
    // ===========================================

    #[test]
    fn test_connection_failed_displays_user_friendly_message() {
        // 接続拒否エラー
        let error = ProviderError::ConnectionFailed("connection refused".to_string());
        let display = error.to_string();

        // ユーザーフレンドリーなメッセージを含むべき
        assert!(
            display.contains("接続が拒否されました")
                || display.contains("Connection refused"),
            "Expected user-friendly connection refused message, got: {}",
            display
        );
        // 対処法のヒントを含むべき
        assert!(
            display.contains("ホスト") || display.contains("ポート") || display.contains("host") || display.contains("port"),
            "Expected hint about host/port, got: {}",
            display
        );
    }

    #[test]
    fn test_connection_failed_displays_auth_error_message() {
        // 認証エラー
        let error = ProviderError::ConnectionFailed(
            "password authentication failed for user \"postgres\"".to_string(),
        );
        let display = error.to_string();

        // 認証エラーとわかるメッセージを含むべき
        assert!(
            display.contains("認証") || display.contains("パスワード")
                || display.contains("authentication") || display.contains("password"),
            "Expected authentication error message, got: {}",
            display
        );
    }

    #[test]
    fn test_connection_failed_displays_hostname_error_message() {
        // ホスト名解決エラー
        let error = ProviderError::ConnectionFailed(
            "could not translate host name \"invalid-host\" to address".to_string(),
        );
        let display = error.to_string();

        // ホスト名エラーとわかるメッセージを含むべき
        assert!(
            display.contains("ホスト名") || display.contains("解決")
                || display.contains("host") || display.contains("resolve"),
            "Expected hostname resolution error message, got: {}",
            display
        );
    }

    #[test]
    fn test_connection_failed_displays_timeout_message() {
        // タイムアウトエラー
        let error = ProviderError::ConnectionFailed("connection timed out".to_string());
        let display = error.to_string();

        // タイムアウトエラーとわかるメッセージを含むべき
        assert!(
            display.contains("タイムアウト") || display.contains("timeout") || display.contains("Timeout"),
            "Expected timeout error message, got: {}",
            display
        );
    }

    // ===========================================
    // クエリエラーのテスト
    // ===========================================

    #[test]
    fn test_query_failed_displays_syntax_error_message() {
        // SQL構文エラー
        let error = ProviderError::QueryFailed(
            "syntax error at or near \"SELEC\"".to_string(),
        );
        let display = error.to_string();

        // 構文エラーとわかるメッセージを含むべき
        assert!(
            display.contains("構文") || display.contains("syntax") || display.contains("Syntax"),
            "Expected syntax error message, got: {}",
            display
        );
    }

    #[test]
    fn test_query_failed_displays_table_not_found_message() {
        // テーブルが見つからないエラー
        let error = ProviderError::QueryFailed(
            "relation \"nonexistent_table\" does not exist".to_string(),
        );
        let display = error.to_string();

        // テーブルが見つからないとわかるメッセージを含むべき
        assert!(
            display.contains("テーブル") || display.contains("存在しません")
                || display.contains("table") || display.contains("not exist") || display.contains("not found"),
            "Expected table not found message, got: {}",
            display
        );
    }

    #[test]
    fn test_query_failed_displays_column_not_found_message() {
        // カラムが見つからないエラー
        let error = ProviderError::QueryFailed(
            "column \"nonexistent_column\" does not exist".to_string(),
        );
        let display = error.to_string();

        // カラムが見つからないとわかるメッセージを含むべき
        assert!(
            display.contains("カラム") || display.contains("列")
                || display.contains("column"),
            "Expected column not found message, got: {}",
            display
        );
    }

    #[test]
    fn test_query_failed_displays_permission_error_message() {
        // 権限エラー
        let error = ProviderError::QueryFailed(
            "permission denied for table users".to_string(),
        );
        let display = error.to_string();

        // 権限エラーとわかるメッセージを含むべき
        assert!(
            display.contains("権限") || display.contains("permission") || display.contains("Permission"),
            "Expected permission error message, got: {}",
            display
        );
    }

    // ===========================================
    // その他のエラーのテスト
    // ===========================================

    #[test]
    fn test_timeout_error_displays_user_friendly_message() {
        let error = ProviderError::Timeout("query took too long".to_string());
        let display = error.to_string();

        assert!(
            display.contains("タイムアウト") || display.contains("timeout") || display.contains("Timeout"),
            "Expected timeout message, got: {}",
            display
        );
    }

    #[test]
    fn test_permission_denied_displays_user_friendly_message() {
        let error = ProviderError::PermissionDenied("access denied to schema".to_string());
        let display = error.to_string();

        assert!(
            display.contains("権限") || display.contains("アクセス")
                || display.contains("permission") || display.contains("Permission") || display.contains("access"),
            "Expected permission denied message, got: {}",
            display
        );
    }

    #[test]
    fn test_not_found_displays_user_friendly_message() {
        let error = ProviderError::NotFound("resource not available".to_string());
        let display = error.to_string();

        assert!(
            display.contains("見つかりません") || display.contains("not found") || display.contains("Not found"),
            "Expected not found message, got: {}",
            display
        );
    }

    #[test]
    fn test_invalid_configuration_displays_user_friendly_message() {
        let error = ProviderError::InvalidConfiguration("invalid port number".to_string());
        let display = error.to_string();

        assert!(
            display.contains("設定") || display.contains("configuration") || display.contains("Configuration") || display.contains("config"),
            "Expected configuration error message, got: {}",
            display
        );
    }

    #[test]
    fn test_internal_error_displays_user_friendly_message() {
        let error = ProviderError::InternalError("mutex poisoned".to_string());
        let display = error.to_string();

        // 内部エラーでもユーザーに理解できるメッセージを含むべき
        assert!(
            display.contains("内部") || display.contains("internal") || display.contains("Internal"),
            "Expected internal error message, got: {}",
            display
        );
    }

    // ===========================================
    // エラーメッセージにデバッグ情報を含むテスト
    // ===========================================

    #[test]
    fn test_error_preserves_original_details() {
        let original_error = "connection refused (os error 111)";
        let error = ProviderError::ConnectionFailed(original_error.to_string());
        let display = error.to_string();

        // デバッグのため、元のエラー情報も含むべき
        assert!(
            display.contains("111") || display.contains("refused"),
            "Expected original error details to be preserved, got: {}",
            display
        );
    }
}
