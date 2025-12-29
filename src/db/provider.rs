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
            ProviderError::ConnectionFailed(msg) => {
                write!(f, "{}", format_connection_error(msg))
            }
            ProviderError::QueryFailed(msg) => {
                write!(f, "{}", format_query_error(msg))
            }
            ProviderError::NotFound(msg) => write!(f, "Not found: {}", msg),
            ProviderError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            ProviderError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            ProviderError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            ProviderError::NotImplemented(msg) => write!(f, "Not implemented: {}", msg),
            ProviderError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

/// Format connection error with user-friendly message and hints
fn format_connection_error(msg: &str) -> String {
    let msg_lower = msg.to_lowercase();

    if msg_lower.contains("connection refused") || msg_lower.contains("could not connect") {
        format!(
            "Connection refused: Cannot connect to the database server. \
             Check that the host and port are correct and the server is running. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("authentication failed")
        || msg_lower.contains("invalid password")
        || ((msg_lower.contains("password") || msg_lower.contains("auth"))
            && (msg_lower.contains("fail") || msg_lower.contains("denied")))
    {
        format!(
            "Authentication failed: Invalid username or password. \
             Check your credentials and try again. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("could not translate host")
        || msg_lower.contains("name resolution")
        || msg_lower.contains("unknown host")
        || msg_lower.contains("no such host")
    {
        format!(
            "Host not found: Cannot resolve the hostname. \
             Check that the host address is correct. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("timed out") || msg_lower.contains("timeout") {
        format!(
            "Connection timeout: The server did not respond in time. \
             Check network connectivity and firewall settings. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("ssl") || msg_lower.contains("tls") {
        format!(
            "SSL/TLS error: Secure connection failed. \
             Check SSL configuration and certificates. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("database") && msg_lower.contains("does not exist") {
        format!(
            "Database not found: The specified database does not exist. \
             Check the database name and ensure it has been created. \
             (Details: {})",
            msg
        )
    } else {
        format!("Connection failed: {} (Check connection settings)", msg)
    }
}

/// Format query error with user-friendly message
fn format_query_error(msg: &str) -> String {
    let msg_lower = msg.to_lowercase();

    if msg_lower.contains("syntax error") {
        format!(
            "SQL syntax error: Check your SQL query for typos or missing keywords. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("relation") && msg_lower.contains("does not exist") {
        format!(
            "Table not found: The specified table does not exist. \
             Check the table name and schema. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("column") && msg_lower.contains("does not exist") {
        format!(
            "Column not found: The specified column does not exist. \
             Check the column name in your query. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("permission denied") {
        format!(
            "Permission denied: You don't have access to perform this operation. \
             Contact your database administrator. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("unique constraint") || msg_lower.contains("duplicate key") {
        format!(
            "Duplicate key error: A record with the same unique key already exists. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("foreign key")
        || (msg_lower.contains("violates") && msg_lower.contains("constraint"))
    {
        format!(
            "Foreign key constraint violation: The operation conflicts with a foreign key. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("deadlock") {
        format!(
            "Deadlock detected: The query was aborted due to a deadlock. \
             Please retry the operation. \
             (Details: {})",
            msg
        )
    } else if msg_lower.contains("out of memory")
        || msg_lower.contains("memory limit")
        || msg_lower.contains("insufficient memory")
    {
        format!(
            "Memory error: The query requires more memory than available. \
             Try simplifying the query or contact your administrator. \
             (Details: {})",
            msg
        )
    } else {
        format!("Query failed: {}", msg)
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
            display.contains("Connection refused"),
            "Expected user-friendly connection refused message, got: {}",
            display
        );
        // 対処法のヒントを含むべき
        assert!(
            display.contains("host") || display.contains("port"),
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
            display.contains("Authentication") || display.contains("password"),
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
            display.contains("Host not found") || display.contains("resolve"),
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
            display.contains("timeout") || display.contains("Timeout"),
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
        let error = ProviderError::QueryFailed("syntax error at or near \"SELEC\"".to_string());
        let display = error.to_string();

        // 構文エラーとわかるメッセージを含むべき
        assert!(
            display.contains("SQL syntax error") || display.contains("syntax"),
            "Expected syntax error message, got: {}",
            display
        );
    }

    #[test]
    fn test_query_failed_displays_table_not_found_message() {
        // テーブルが見つからないエラー
        let error =
            ProviderError::QueryFailed("relation \"nonexistent_table\" does not exist".to_string());
        let display = error.to_string();

        // テーブルが見つからないとわかるメッセージを含むべき
        assert!(
            display.contains("Table not found") || display.contains("does not exist"),
            "Expected table not found message, got: {}",
            display
        );
    }

    #[test]
    fn test_query_failed_displays_column_not_found_message() {
        // カラムが見つからないエラー
        let error =
            ProviderError::QueryFailed("column \"nonexistent_column\" does not exist".to_string());
        let display = error.to_string();

        // カラムが見つからないとわかるメッセージを含むべき
        assert!(
            display.contains("Column not found") || display.contains("column"),
            "Expected column not found message, got: {}",
            display
        );
    }

    #[test]
    fn test_query_failed_displays_permission_error_message() {
        // 権限エラー
        let error = ProviderError::QueryFailed("permission denied for table users".to_string());
        let display = error.to_string();

        // 権限エラーとわかるメッセージを含むべき
        assert!(
            display.contains("Permission denied") || display.contains("permission"),
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
            display.contains("Timeout"),
            "Expected timeout message, got: {}",
            display
        );
    }

    #[test]
    fn test_permission_denied_displays_user_friendly_message() {
        let error = ProviderError::PermissionDenied("access denied to schema".to_string());
        let display = error.to_string();

        assert!(
            display.contains("Permission denied"),
            "Expected permission denied message, got: {}",
            display
        );
    }

    #[test]
    fn test_not_found_displays_user_friendly_message() {
        let error = ProviderError::NotFound("resource not available".to_string());
        let display = error.to_string();

        assert!(
            display.contains("Not found"),
            "Expected not found message, got: {}",
            display
        );
    }

    #[test]
    fn test_invalid_configuration_displays_user_friendly_message() {
        let error = ProviderError::InvalidConfiguration("invalid port number".to_string());
        let display = error.to_string();

        assert!(
            display.contains("Invalid configuration"),
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
            display.contains("Internal error"),
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
