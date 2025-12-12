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
