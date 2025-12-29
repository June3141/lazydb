//! Database worker handlers (async operations)

use crate::app::App;
use crate::db::{ConnectionParams, DbCommand, DbResponse, DbWorkerHandle};
use crate::model::schema::Routine;
use crate::model::{Connection, HistoryEntry, Pagination, QueryResult, Table};

impl App {
    /// Set the DB worker handle for async operations
    pub fn set_db_worker(&mut self, worker: DbWorkerHandle) {
        self.db_worker = Some(worker);
    }

    /// Get the next unique request ID
    pub(crate) fn next_request_id(&mut self) -> u64 {
        let id = self.next_request_id;
        self.next_request_id += 1;
        id
    }

    /// Process any pending responses from the DB worker.
    /// This should be called regularly from the event loop.
    pub fn process_db_responses(&mut self) {
        // Collect responses first to avoid borrow issues
        let responses: Vec<DbResponse> = {
            let Some(ref worker) = self.db_worker else {
                return;
            };
            let mut responses = Vec::new();
            while let Ok(response) = worker.try_recv() {
                responses.push(response);
            }
            responses
        };

        // Process all collected responses
        for response in responses {
            self.handle_db_response(response);
        }
    }

    /// Handle a single DB response
    fn handle_db_response(&mut self, response: DbResponse) {
        match response {
            DbResponse::TablesLoaded { result, target, .. } => {
                self.handle_tables_loaded(result, target);
            }
            DbResponse::TableDetailsLoaded { result, target, .. } => {
                self.handle_table_details_loaded(result, target);
            }
            DbResponse::QueryExecuted {
                result,
                project_idx,
                ..
            } => {
                self.handle_query_executed(result, project_idx);
            }
            DbResponse::RoutinesLoaded { result, target, .. } => {
                self.handle_routines_loaded(result, target);
            }
        }
    }

    /// Handle tables loaded response
    fn handle_tables_loaded(&mut self, result: Result<Vec<Table>, String>, target: (usize, usize)) {
        let (proj_idx, conn_idx) = target;

        // Clear loading state
        self.loading.fetching_tables = None;

        match result {
            Ok(tables) => {
                if let Some(project) = self.projects.get_mut(proj_idx) {
                    if let Some(conn) = project.connections.get_mut(conn_idx) {
                        let table_count = tables.len();
                        conn.tables = tables;
                        self.status_message = format!("Loaded {} tables", table_count);
                        self.loading.message = None;
                    }
                }
            }
            Err(e) => {
                // Collapse the connection on error
                if let Some(project) = self.projects.get_mut(proj_idx) {
                    if let Some(conn) = project.connections.get_mut(conn_idx) {
                        conn.expanded = false;
                    }
                }
                self.status_message = format!("Failed to get tables: {}", e);
                self.loading.message = None;
            }
        }
    }

    /// Handle table details loaded response
    fn handle_table_details_loaded(
        &mut self,
        result: Result<Table, String>,
        target: (usize, usize, usize),
    ) {
        let (proj_idx, conn_idx, table_idx) = target;

        // Clear loading state
        self.loading.fetching_details = None;

        match result {
            Ok(detailed_table) => {
                if let Some(project) = self.projects.get_mut(proj_idx) {
                    if let Some(conn) = project.connections.get_mut(conn_idx) {
                        if let Some(table) = conn.tables.get_mut(table_idx) {
                            let table_name = table.name.clone();
                            table.columns = detailed_table.columns;
                            table.indexes = detailed_table.indexes;
                            table.foreign_keys = detailed_table.foreign_keys;
                            table.constraints = detailed_table.constraints;
                            table.triggers = detailed_table.triggers;
                            table.details_loaded = true;
                            self.status_message = format!("Loaded schema for {}", table_name);
                        }
                    }
                }
                self.loading.message = None;
            }
            Err(e) => {
                self.status_message = format!("Failed to get table details: {}", e);
                self.loading.message = None;
            }
        }
    }

    /// Handle query executed response
    fn handle_query_executed(&mut self, result: Result<QueryResult, String>, _project_idx: usize) {
        // Clear loading state
        self.loading.executing_query = false;

        // Get pending query info for history
        let query_info = self.pending_query_info.take();

        match result {
            Ok(query_result) => {
                let row_count = query_result.rows.len();
                let execution_time_ms = query_result.execution_time_ms;

                // Add to history if we have query info
                if let Some((conn_name, database, query)) = query_info {
                    self.query_history.add(HistoryEntry::success(
                        &query,
                        &conn_name,
                        &database,
                        execution_time_ms,
                        row_count,
                    ));
                    self.history_dirty = true;
                    self.status_message = format!("Fetched {} rows from {}", row_count, database);
                }

                // Update result
                self.pagination = Pagination::new(row_count);
                self.result = Some(query_result);
                self.loading.message = None;
            }
            Err(e) => {
                // Add error to history if we have query info
                if let Some((conn_name, database, query)) = query_info {
                    self.query_history.add(HistoryEntry::error(
                        &query,
                        &conn_name,
                        &database,
                        e.clone(),
                    ));
                    self.history_dirty = true;
                }

                self.result = None;
                self.pagination = Pagination::default();
                self.status_message = format!("Query failed: {}", e);
                self.loading.message = None;
            }
        }
    }

    /// Send a command to fetch tables asynchronously
    pub(crate) fn send_fetch_tables(
        &mut self,
        conn: &Connection,
        proj_idx: usize,
        conn_idx: usize,
    ) {
        let request_id = self.next_request_id();
        let connection = ConnectionParams::from_connection(conn);

        let cmd = DbCommand::FetchTables {
            request_id,
            connection,
            schema: Some("public".to_string()),
            target: (proj_idx, conn_idx),
        };

        if let Some(worker) = self.db_worker.as_ref() {
            if worker.send(cmd).is_ok() {
                self.loading.start_fetching_tables(conn_idx);
            } else {
                self.status_message = "Failed to send command to DB worker".to_string();
            }
        } else {
            self.status_message = "DB worker not initialized".to_string();
        }
    }

    /// Send a command to fetch table details asynchronously
    pub(crate) fn send_fetch_table_details(
        &mut self,
        conn: &Connection,
        table_name: &str,
        schema: Option<&str>,
        proj_idx: usize,
        conn_idx: usize,
        table_idx: usize,
    ) {
        let request_id = self.next_request_id();
        let connection = ConnectionParams::from_connection(conn);

        let cmd = DbCommand::FetchTableDetails {
            request_id,
            connection,
            table_name: table_name.to_string(),
            schema: schema.map(|s| s.to_string()),
            target: (proj_idx, conn_idx, table_idx),
        };

        if let Some(worker) = self.db_worker.as_ref() {
            if worker.send(cmd).is_ok() {
                self.loading
                    .start_fetching_details(proj_idx, conn_idx, table_idx);
            } else {
                self.status_message = "Failed to send command to DB worker".to_string();
            }
        } else {
            self.status_message = "DB worker not initialized".to_string();
        }
    }

    /// Send a command to execute a query asynchronously
    pub(crate) fn send_execute_query(&mut self, conn: &Connection, query: &str, proj_idx: usize) {
        let request_id = self.next_request_id();
        let connection = ConnectionParams::from_connection(conn);

        // Store query info for history
        self.pending_query_info =
            Some((conn.name.clone(), conn.database.clone(), query.to_string()));

        let cmd = DbCommand::ExecuteQuery {
            request_id,
            connection,
            query: query.to_string(),
            project_idx: proj_idx,
        };

        if let Some(worker) = self.db_worker.as_ref() {
            if worker.send(cmd).is_ok() {
                self.loading.start_executing_query();
            } else {
                self.status_message = "Failed to send command to DB worker".to_string();
                self.pending_query_info = None;
            }
        } else {
            self.status_message = "DB worker not initialized".to_string();
            self.pending_query_info = None;
        }
    }

    /// Fetch table details (columns, indexes, foreign keys, constraints) if not already loaded
    pub(crate) fn fetch_table_details_if_needed(&mut self, proj_idx: usize) {
        // Capture indices at the start to avoid race conditions
        let conn_idx = self.selected_connection_idx;
        let Some(table_idx) = self.selected_table_idx else {
            return;
        };

        let Some(project) = self.projects.get(proj_idx) else {
            return;
        };
        let Some(conn) = project.connections.get(conn_idx) else {
            return;
        };
        let Some(table) = conn.tables.get(table_idx) else {
            return;
        };

        // Skip if details are already loaded or currently loading
        if table.details_loaded || self.loading.is_fetching_details() {
            return;
        }

        let table_name = table.name.clone();
        let schema = table.schema.clone();
        let conn_clone = conn.clone();

        // Send async command to fetch table details
        self.send_fetch_table_details(
            &conn_clone,
            &table_name,
            schema.as_deref(),
            proj_idx,
            conn_idx,
            table_idx,
        );
    }

    /// Handle routines loaded response
    fn handle_routines_loaded(
        &mut self,
        result: Result<Vec<Routine>, String>,
        target: (usize, usize),
    ) {
        let (proj_idx, conn_idx) = target;

        // Clear loading state
        self.loading.fetching_routines = None;

        match result {
            Ok(routines) => {
                if let Some(project) = self.projects.get_mut(proj_idx) {
                    if let Some(conn) = project.connections.get_mut(conn_idx) {
                        let routine_count = routines.len();
                        conn.routines = routines;
                        conn.routines_loaded = true;
                        self.status_message = format!("Loaded {} routines", routine_count);
                        self.loading.message = None;
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Failed to get routines: {}", e);
                self.loading.message = None;
            }
        }
    }

    /// Send a command to fetch routines (stored procedures and functions) asynchronously
    pub(crate) fn send_fetch_routines(
        &mut self,
        conn: &Connection,
        proj_idx: usize,
        conn_idx: usize,
    ) {
        let request_id = self.next_request_id();
        let connection = ConnectionParams::from_connection(conn);

        let cmd = DbCommand::FetchRoutines {
            request_id,
            connection,
            schema: Some("public".to_string()),
            target: (proj_idx, conn_idx),
        };

        if let Some(worker) = self.db_worker.as_ref() {
            if worker.send(cmd).is_ok() {
                self.loading.start_fetching_routines(conn_idx);
            } else {
                self.status_message = "Failed to send command to DB worker".to_string();
            }
        } else {
            self.status_message = "DB worker not initialized".to_string();
        }
    }
}
