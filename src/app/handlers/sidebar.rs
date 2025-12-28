//! Sidebar action handlers (activate, go back, toggle expand)

use crate::app::enums::SidebarMode;
use crate::app::App;

impl App {
    /// Activate current selection (Enter key)
    pub(crate) fn activate(&mut self) {
        match self.sidebar_mode {
            SidebarMode::Projects => {
                // Enter project -> show its connections
                self.sidebar_mode = SidebarMode::Connections(self.selected_project_idx);
                self.selected_connection_idx = 0;
                self.selected_table_idx = None;
                if let Some(project) = self.projects.get(self.selected_project_idx) {
                    self.status_message = format!("Project: {}", project.name);
                }
            }
            SidebarMode::Connections(proj_idx) => {
                if self.selected_table_idx.is_some() {
                    // Table selected: generate query
                    self.activate_table(proj_idx);
                } else {
                    // Connection selected: toggle expand
                    self.toggle_connection_expand(proj_idx);
                }
            }
        }
    }

    /// Go back to Projects view (Backspace key)
    pub(crate) fn go_back(&mut self) {
        if let SidebarMode::Connections(_) = self.sidebar_mode {
            self.sidebar_mode = SidebarMode::Projects;
            self.status_message = "Projects".to_string();
        }
    }

    pub(crate) fn toggle_connection_expand(&mut self, proj_idx: usize) {
        let conn_idx = self.selected_connection_idx;

        // Skip if tables are already being fetched for this connection
        if self.loading.is_fetching_tables_for(conn_idx) {
            return;
        }

        // First, check if we need to fetch tables (connection is being expanded and has no tables)
        let should_fetch = if let Some(project) = self.projects.get(proj_idx) {
            if let Some(conn) = project.connections.get(conn_idx) {
                !conn.expanded && conn.tables.is_empty()
            } else {
                false
            }
        } else {
            false
        };

        // Toggle the expanded state
        if let Some(project) = self.projects.get_mut(proj_idx) {
            if let Some(conn) = project.connections.get_mut(conn_idx) {
                conn.expanded = !conn.expanded;
                if !conn.expanded {
                    self.selected_table_idx = None;
                }
            }
        }

        // If we need to fetch tables, send async command
        if should_fetch {
            // Clone connection data for the async operation
            let conn_clone = self
                .projects
                .get(proj_idx)
                .and_then(|p| p.connections.get(conn_idx))
                .cloned();

            if let Some(conn) = conn_clone {
                self.send_fetch_tables(&conn, proj_idx, conn_idx);
            }
        }
    }

    pub(crate) fn activate_table(&mut self, proj_idx: usize) {
        // Skip if a query is already executing
        if self.loading.executing_query {
            return;
        }

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

        // Safely quote the table name as a SQL identifier, escaping any embedded double quotes
        let escaped_table_name = table.name.replace('"', "\"\"");
        let query = format!("SELECT * FROM \"{}\"", escaped_table_name);
        self.query = format!("{};", query);

        // Clone connection for async operation
        let conn_clone = conn.clone();

        // Send async command to execute query
        self.send_execute_query(&conn_clone, &query, proj_idx);
    }
}
