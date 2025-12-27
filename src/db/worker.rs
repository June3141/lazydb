//! Background database worker thread
//!
//! This module provides a worker thread that handles database operations
//! in the background, preventing UI blocking during network I/O.

use std::sync::mpsc::{Receiver, Sender};
use std::thread::{self, JoinHandle};

use super::async_bridge::{ConnectionParams, DbCommand, DbResponse};
use super::{DatabaseProvider, PostgresProvider};

/// Background worker that processes database commands
pub struct DbWorker {
    command_rx: Receiver<DbCommand>,
    response_tx: Sender<DbResponse>,
}

impl DbWorker {
    /// Create a new worker with the given channels
    pub fn new(command_rx: Receiver<DbCommand>, response_tx: Sender<DbResponse>) -> Self {
        Self {
            command_rx,
            response_tx,
        }
    }

    /// Run the worker's main loop
    ///
    /// This will block until a Shutdown command is received or the command
    /// channel is closed.
    pub fn run(self) {
        loop {
            match self.command_rx.recv() {
                Ok(DbCommand::Shutdown) => break,
                Ok(cmd) => self.handle_command(cmd),
                Err(_) => break, // Channel closed
            }
        }
    }

    /// Handle a single database command
    fn handle_command(&self, cmd: DbCommand) {
        match cmd {
            DbCommand::FetchTables {
                request_id,
                connection,
                schema,
                target,
            } => {
                let result = self.fetch_tables(&connection, schema.as_deref());
                let _ = self.response_tx.send(DbResponse::TablesLoaded {
                    request_id,
                    result,
                    target,
                });
            }

            DbCommand::FetchTableDetails {
                request_id,
                connection,
                table_name,
                schema,
                target,
            } => {
                let result = self.fetch_table_details(&connection, &table_name, schema.as_deref());
                let _ = self.response_tx.send(DbResponse::TableDetailsLoaded {
                    request_id,
                    result,
                    target,
                });
            }

            DbCommand::ExecuteQuery {
                request_id,
                connection,
                query,
                project_idx,
            } => {
                let result = self.execute_query(&connection, &query);
                let _ = self.response_tx.send(DbResponse::QueryExecuted {
                    request_id,
                    result,
                    project_idx,
                });
            }

            DbCommand::Shutdown => {
                // Already handled in run()
            }
        }
    }

    /// Create a provider connection and fetch tables
    fn fetch_tables(
        &self,
        conn: &ConnectionParams,
        schema: Option<&str>,
    ) -> Result<Vec<crate::model::Table>, String> {
        let provider = self.create_provider(conn)?;
        provider.get_tables(schema).map_err(|e| e.to_string())
    }

    /// Create a provider connection and fetch table details
    fn fetch_table_details(
        &self,
        conn: &ConnectionParams,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<crate::model::Table, String> {
        let provider = self.create_provider(conn)?;
        provider
            .get_table_details(table_name, schema)
            .map_err(|e| e.to_string())
    }

    /// Create a provider connection and execute a query
    fn execute_query(
        &self,
        conn: &ConnectionParams,
        query: &str,
    ) -> Result<crate::model::QueryResult, String> {
        let provider = self.create_provider(conn)?;
        provider.execute_query(query).map_err(|e| e.to_string())
    }

    /// Create a new database provider from connection parameters
    fn create_provider(&self, conn: &ConnectionParams) -> Result<PostgresProvider, String> {
        PostgresProvider::connect(
            &conn.host,
            conn.port,
            &conn.database,
            &conn.username,
            &conn.password,
        )
        .map_err(|e| e.to_string())
    }
}

/// Handle to the spawned worker thread
pub struct DbWorkerHandle {
    /// Channel to send commands to the worker
    pub command_tx: Sender<DbCommand>,
    /// Channel to receive responses from the worker
    pub response_rx: Receiver<DbResponse>,
    /// Handle to the worker thread (for joining on shutdown)
    thread_handle: Option<JoinHandle<()>>,
}

impl DbWorkerHandle {
    /// Send a command to the worker
    #[allow(clippy::result_large_err)]
    pub fn send(&self, cmd: DbCommand) -> Result<(), std::sync::mpsc::SendError<DbCommand>> {
        self.command_tx.send(cmd)
    }

    /// Try to receive a response without blocking
    pub fn try_recv(&self) -> Result<DbResponse, std::sync::mpsc::TryRecvError> {
        self.response_rx.try_recv()
    }

    /// Shutdown the worker and wait for it to finish
    pub fn shutdown(mut self) {
        let _ = self.command_tx.send(DbCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for DbWorkerHandle {
    fn drop(&mut self) {
        // Send shutdown signal if we still have the handle
        let _ = self.command_tx.send(DbCommand::Shutdown);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Spawn a new database worker thread
///
/// Returns a handle that can be used to send commands and receive responses.
pub fn spawn_db_worker() -> DbWorkerHandle {
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
    let (resp_tx, resp_rx) = std::sync::mpsc::channel();

    let handle = thread::Builder::new()
        .name("db-worker".to_string())
        .spawn(move || {
            let worker = DbWorker::new(cmd_rx, resp_tx);
            worker.run();
        })
        .expect("Failed to spawn db-worker thread");

    DbWorkerHandle {
        command_tx: cmd_tx,
        response_rx: resp_rx,
        thread_handle: Some(handle),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn test_worker_shutdown() {
        let (cmd_tx, cmd_rx) = mpsc::channel();
        let (resp_tx, _resp_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let worker = DbWorker::new(cmd_rx, resp_tx);
            worker.run();
        });

        // Send shutdown command
        cmd_tx.send(DbCommand::Shutdown).unwrap();

        // Worker should exit gracefully
        handle.join().expect("Worker thread panicked");
    }

    #[test]
    fn test_worker_channel_close() {
        let (cmd_tx, cmd_rx) = mpsc::channel::<DbCommand>();
        let (resp_tx, _resp_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let worker = DbWorker::new(cmd_rx, resp_tx);
            worker.run();
        });

        // Drop the sender to close the channel
        drop(cmd_tx);

        // Worker should exit gracefully when channel closes
        handle.join().expect("Worker thread panicked");
    }

    #[test]
    fn test_spawn_db_worker() {
        let handle = spawn_db_worker();

        // Shutdown cleanly
        handle.shutdown();
    }

    #[test]
    fn test_worker_handle_drop() {
        let handle = spawn_db_worker();

        // Just drop it - should shutdown cleanly via Drop impl
        drop(handle);
    }

    #[test]
    fn test_fetch_tables_connection_error() {
        let handle = spawn_db_worker();

        // Send a command with invalid connection params
        let invalid_conn = ConnectionParams {
            host: "invalid-host-that-does-not-exist.local".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        handle
            .send(DbCommand::FetchTables {
                request_id: 1,
                connection: invalid_conn,
                schema: Some("public".to_string()),
                target: (0, 0),
            })
            .unwrap();

        // Wait for response with timeout
        let response = loop {
            match handle.try_recv() {
                Ok(resp) => break resp,
                Err(mpsc::TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("Worker disconnected unexpectedly");
                }
            }
        };

        // Should get an error response
        match response {
            DbResponse::TablesLoaded {
                request_id, result, ..
            } => {
                assert_eq!(request_id, 1);
                assert!(result.is_err());
            }
            _ => panic!("Expected TablesLoaded response"),
        }

        handle.shutdown();
    }

    #[test]
    fn test_fetch_table_details_connection_error() {
        let handle = spawn_db_worker();

        let invalid_conn = ConnectionParams {
            host: "invalid-host-that-does-not-exist.local".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        handle
            .send(DbCommand::FetchTableDetails {
                request_id: 2,
                connection: invalid_conn,
                table_name: "users".to_string(),
                schema: Some("public".to_string()),
                target: (0, 0, 0),
            })
            .unwrap();

        // Wait for response with timeout
        let response = loop {
            match handle.try_recv() {
                Ok(resp) => break resp,
                Err(mpsc::TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("Worker disconnected unexpectedly");
                }
            }
        };

        match response {
            DbResponse::TableDetailsLoaded {
                request_id, result, ..
            } => {
                assert_eq!(request_id, 2);
                assert!(result.is_err());
            }
            _ => panic!("Expected TableDetailsLoaded response"),
        }

        handle.shutdown();
    }

    #[test]
    fn test_execute_query_connection_error() {
        let handle = spawn_db_worker();

        let invalid_conn = ConnectionParams {
            host: "invalid-host-that-does-not-exist.local".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        handle
            .send(DbCommand::ExecuteQuery {
                request_id: 3,
                connection: invalid_conn,
                query: "SELECT 1".to_string(),
                project_idx: 0,
            })
            .unwrap();

        // Wait for response with timeout
        let response = loop {
            match handle.try_recv() {
                Ok(resp) => break resp,
                Err(mpsc::TryRecvError::Empty) => {
                    thread::sleep(Duration::from_millis(100));
                    continue;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("Worker disconnected unexpectedly");
                }
            }
        };

        match response {
            DbResponse::QueryExecuted {
                request_id, result, ..
            } => {
                assert_eq!(request_id, 3);
                assert!(result.is_err());
            }
            _ => panic!("Expected QueryExecuted response"),
        }

        handle.shutdown();
    }

    #[test]
    fn test_multiple_commands() {
        let handle = spawn_db_worker();

        let invalid_conn = ConnectionParams {
            host: "invalid-host".to_string(),
            port: 5432,
            database: "testdb".to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
        };

        // Send multiple commands
        for i in 0..3 {
            handle
                .send(DbCommand::FetchTables {
                    request_id: i,
                    connection: invalid_conn.clone(),
                    schema: None,
                    target: (0, i as usize),
                })
                .unwrap();
        }

        // Collect all responses
        let mut responses = Vec::new();
        for _ in 0..3 {
            let response = loop {
                match handle.try_recv() {
                    Ok(resp) => break resp,
                    Err(mpsc::TryRecvError::Empty) => {
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => {
                        panic!("Worker disconnected unexpectedly");
                    }
                }
            };
            responses.push(response);
        }

        // Should have received all 3 responses
        assert_eq!(responses.len(), 3);

        // All should be TablesLoaded with errors
        for resp in responses {
            match resp {
                DbResponse::TablesLoaded { result, .. } => {
                    assert!(result.is_err());
                }
                _ => panic!("Expected TablesLoaded response"),
            }
        }

        handle.shutdown();
    }
}
