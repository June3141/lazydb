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
/// # Example
/// ```ignore
/// struct PostgresProvider { /* connection details */ }
///
/// impl DatabaseProvider for PostgresProvider {
///     fn database_type(&self) -> DatabaseType { DatabaseType::PostgreSQL }
///     // ... implement other methods
/// }
/// ```
pub trait DatabaseProvider: Send + Sync {
    /// Returns the type of database this provider handles
    fn database_type(&self) -> DatabaseType;

    /// Get list of schemas/databases
    fn get_schemas(&self) -> Result<Vec<String>, ProviderError>;

    /// Get list of tables in a schema
    fn get_tables(&self, schema: Option<&str>) -> Result<Vec<Table>, ProviderError>;

    /// Get detailed table information including columns, indexes, etc.
    fn get_table_details(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Table, ProviderError>;

    /// Get columns for a table
    fn get_columns(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Column>, ProviderError>;

    /// Get indexes for a table
    fn get_indexes(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Index>, ProviderError>;

    /// Get foreign keys for a table
    fn get_foreign_keys(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Vec<ForeignKey>, ProviderError>;

    /// Get constraints for a table
    fn get_constraints(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Vec<Constraint>, ProviderError>;

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
        }
    }
}

impl std::error::Error for ProviderError {}
