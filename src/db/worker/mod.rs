//! Background database worker thread
//!
//! This module provides a worker thread that handles database operations
//! in the background, preventing UI blocking during network I/O.

mod handle;

#[cfg(test)]
mod tests;

use std::sync::mpsc::{Receiver, Sender};

use super::async_bridge::{ConnectionParams, DbCommand, DbResponse};
use super::{DatabaseProvider, PostgresProvider};

pub use handle::{spawn_db_worker, DbWorkerHandle};

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

            DbCommand::FetchRoutines {
                request_id,
                connection,
                schema,
                target,
            } => {
                let result = self.fetch_routines(&connection, schema.as_deref());
                let _ = self.response_tx.send(DbResponse::RoutinesLoaded {
                    request_id,
                    result,
                    target,
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

    /// Create a provider connection and fetch routines
    fn fetch_routines(
        &self,
        conn: &ConnectionParams,
        schema: Option<&str>,
    ) -> Result<Vec<crate::model::schema::Routine>, String> {
        let provider = self.create_provider(conn)?;
        provider.get_routines(schema).map_err(|e| e.to_string())
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
