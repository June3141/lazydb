//! Async bridge types for communication between UI and DB worker threads
//!
//! This module defines the command and response types used for
//! non-blocking database operations.

use crate::model::{Connection, QueryResult, Table};

/// Parameters needed to establish a database connection.
/// This is a thread-safe, owned version of connection details.
#[derive(Debug, Clone)]
pub struct ConnectionParams {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
}

impl ConnectionParams {
    /// Create connection params from a Connection reference
    pub fn from_connection(conn: &Connection) -> Self {
        Self {
            host: conn.host.clone(),
            port: conn.port,
            database: conn.database.clone(),
            username: conn.username.clone(),
            password: conn.password.clone(),
        }
    }
}

/// Commands sent from the UI thread to the DB worker thread
#[derive(Debug)]
pub enum DbCommand {
    /// Fetch the list of tables for a connection
    FetchTables {
        request_id: u64,
        connection: ConnectionParams,
        schema: Option<String>,
        /// Project and connection index to update when complete
        target: (usize, usize),
    },

    /// Fetch detailed information about a specific table
    FetchTableDetails {
        request_id: u64,
        connection: ConnectionParams,
        table_name: String,
        schema: Option<String>,
        /// Project, connection, and table index to update when complete
        target: (usize, usize, usize),
    },

    /// Execute a query and return results
    ExecuteQuery {
        request_id: u64,
        connection: ConnectionParams,
        query: String,
        /// Project index for result storage
        project_idx: usize,
    },

    /// Shutdown the worker thread
    Shutdown,
}

/// Responses sent from the DB worker thread back to the UI thread
#[derive(Debug)]
pub enum DbResponse {
    /// Tables list was loaded
    TablesLoaded {
        request_id: u64,
        result: Result<Vec<Table>, String>,
        /// Project and connection index to update
        target: (usize, usize),
    },

    /// Table details were loaded
    TableDetailsLoaded {
        request_id: u64,
        result: Result<Table, String>,
        /// Project, connection, and table index to update
        target: (usize, usize, usize),
    },

    /// Query was executed
    QueryExecuted {
        request_id: u64,
        result: Result<QueryResult, String>,
        /// Project index for result storage
        project_idx: usize,
    },
}

impl DbCommand {
    /// Get the request ID for this command
    pub fn request_id(&self) -> Option<u64> {
        match self {
            DbCommand::FetchTables { request_id, .. } => Some(*request_id),
            DbCommand::FetchTableDetails { request_id, .. } => Some(*request_id),
            DbCommand::ExecuteQuery { request_id, .. } => Some(*request_id),
            DbCommand::Shutdown => None,
        }
    }
}

impl DbResponse {
    /// Get the request ID for this response
    pub fn request_id(&self) -> u64 {
        match self {
            DbResponse::TablesLoaded { request_id, .. } => *request_id,
            DbResponse::TableDetailsLoaded { request_id, .. } => *request_id,
            DbResponse::QueryExecuted { request_id, .. } => *request_id,
        }
    }

    /// Check if the response indicates success
    pub fn is_success(&self) -> bool {
        match self {
            DbResponse::TablesLoaded { result, .. } => result.is_ok(),
            DbResponse::TableDetailsLoaded { result, .. } => result.is_ok(),
            DbResponse::QueryExecuted { result, .. } => result.is_ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_connection() -> Connection {
        Connection {
            name: "test".to_string(),
            host: "localhost".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            expanded: false,
            tables: Vec::new(),
        }
    }

    #[test]
    fn test_connection_params_from_connection() {
        let conn = create_test_connection();
        let params = ConnectionParams::from_connection(&conn);

        assert_eq!(params.host, "localhost");
        assert_eq!(params.port, 5432);
        assert_eq!(params.database, "testdb");
        assert_eq!(params.username, "testuser");
        assert_eq!(params.password, "testpass");
    }

    #[test]
    fn test_db_command_request_id() {
        let params = ConnectionParams::from_connection(&create_test_connection());

        let cmd = DbCommand::FetchTables {
            request_id: 42,
            connection: params.clone(),
            schema: None,
            target: (0, 0),
        };
        assert_eq!(cmd.request_id(), Some(42));

        let cmd = DbCommand::FetchTableDetails {
            request_id: 100,
            connection: params.clone(),
            table_name: "users".to_string(),
            schema: None,
            target: (0, 0, 0),
        };
        assert_eq!(cmd.request_id(), Some(100));

        let cmd = DbCommand::ExecuteQuery {
            request_id: 999,
            connection: params,
            query: "SELECT 1".to_string(),
            project_idx: 0,
        };
        assert_eq!(cmd.request_id(), Some(999));

        let cmd = DbCommand::Shutdown;
        assert_eq!(cmd.request_id(), None);
    }

    #[test]
    fn test_db_response_request_id() {
        let resp = DbResponse::TablesLoaded {
            request_id: 42,
            result: Ok(vec![]),
            target: (0, 0),
        };
        assert_eq!(resp.request_id(), 42);

        let resp = DbResponse::TableDetailsLoaded {
            request_id: 100,
            result: Ok(Table::new("test")),
            target: (0, 0, 0),
        };
        assert_eq!(resp.request_id(), 100);

        let resp = DbResponse::QueryExecuted {
            request_id: 999,
            result: Err("error".to_string()),
            project_idx: 0,
        };
        assert_eq!(resp.request_id(), 999);
    }

    #[test]
    fn test_db_response_is_success() {
        let resp = DbResponse::TablesLoaded {
            request_id: 1,
            result: Ok(vec![]),
            target: (0, 0),
        };
        assert!(resp.is_success());

        let resp = DbResponse::TablesLoaded {
            request_id: 1,
            result: Err("error".to_string()),
            target: (0, 0),
        };
        assert!(!resp.is_success());

        let resp = DbResponse::TableDetailsLoaded {
            request_id: 1,
            result: Ok(Table::new("test")),
            target: (0, 0, 0),
        };
        assert!(resp.is_success());

        let resp = DbResponse::TableDetailsLoaded {
            request_id: 1,
            result: Err("error".to_string()),
            target: (0, 0, 0),
        };
        assert!(!resp.is_success());

        let resp = DbResponse::QueryExecuted {
            request_id: 1,
            result: Ok(QueryResult {
                columns: vec![],
                rows: vec![],
                execution_time_ms: 0,
                total_rows: 0,
            }),
            project_idx: 0,
        };
        assert!(resp.is_success());

        let resp = DbResponse::QueryExecuted {
            request_id: 1,
            result: Err("error".to_string()),
            project_idx: 0,
        };
        assert!(!resp.is_success());
    }

    #[test]
    fn test_db_command_fetch_tables_with_schema() {
        let params = ConnectionParams::from_connection(&create_test_connection());
        let cmd = DbCommand::FetchTables {
            request_id: 1,
            connection: params,
            schema: Some("public".to_string()),
            target: (0, 1),
        };

        if let DbCommand::FetchTables { schema, target, .. } = cmd {
            assert_eq!(schema, Some("public".to_string()));
            assert_eq!(target, (0, 1));
        } else {
            panic!("Expected FetchTables command");
        }
    }

    #[test]
    fn test_db_command_fetch_table_details() {
        let params = ConnectionParams::from_connection(&create_test_connection());
        let cmd = DbCommand::FetchTableDetails {
            request_id: 1,
            connection: params,
            table_name: "users".to_string(),
            schema: Some("public".to_string()),
            target: (0, 1, 2),
        };

        if let DbCommand::FetchTableDetails {
            table_name,
            schema,
            target,
            ..
        } = cmd
        {
            assert_eq!(table_name, "users");
            assert_eq!(schema, Some("public".to_string()));
            assert_eq!(target, (0, 1, 2));
        } else {
            panic!("Expected FetchTableDetails command");
        }
    }
}
