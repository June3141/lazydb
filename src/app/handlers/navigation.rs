//! Navigation handlers for sidebar and data table

use crate::app::enums::SidebarMode;
use crate::app::App;

impl App {
    /// Navigate up based on current sidebar mode
    pub(crate) fn navigate_up(&mut self) {
        match self.sidebar_mode {
            SidebarMode::Projects => {
                if self.selected_project_idx > 0 {
                    self.selected_project_idx -= 1;
                } else if !self.projects.is_empty() {
                    self.selected_project_idx = self.projects.len() - 1;
                }
            }
            SidebarMode::Connections(proj_idx) => {
                self.navigate_connections_up(proj_idx);
            }
        }
    }

    /// Navigate down based on current sidebar mode
    pub(crate) fn navigate_down(&mut self) {
        match self.sidebar_mode {
            SidebarMode::Projects => {
                if self.selected_project_idx + 1 < self.projects.len() {
                    self.selected_project_idx += 1;
                } else if !self.projects.is_empty() {
                    self.selected_project_idx = 0;
                }
            }
            SidebarMode::Connections(proj_idx) => {
                self.navigate_connections_down(proj_idx);
            }
        }
    }

    fn navigate_connections_up(&mut self, proj_idx: usize) {
        let Some(project) = self.projects.get(proj_idx) else {
            return;
        };

        // Build flat list of (conn_idx, Option<table_idx>)
        let items: Vec<(usize, Option<usize>)> = project
            .connections
            .iter()
            .enumerate()
            .flat_map(|(conn_idx, conn)| {
                let mut v = vec![(conn_idx, None)];
                if conn.expanded {
                    for table_idx in 0..conn.tables.len() {
                        v.push((conn_idx, Some(table_idx)));
                    }
                }
                v
            })
            .collect();

        if items.is_empty() {
            return;
        }

        let current = items
            .iter()
            .position(|(c, t)| *c == self.selected_connection_idx && *t == self.selected_table_idx)
            .unwrap_or(0);

        let new_idx = if current == 0 {
            items.len() - 1
        } else {
            current - 1
        };
        let (conn_idx, table_idx) = items[new_idx];
        self.selected_connection_idx = conn_idx;
        self.selected_table_idx = table_idx;

        // Fetch table details if a table is selected
        if table_idx.is_some() {
            self.fetch_table_details_if_needed(proj_idx);
        }
    }

    fn navigate_connections_down(&mut self, proj_idx: usize) {
        let Some(project) = self.projects.get(proj_idx) else {
            return;
        };

        let items: Vec<(usize, Option<usize>)> = project
            .connections
            .iter()
            .enumerate()
            .flat_map(|(conn_idx, conn)| {
                let mut v = vec![(conn_idx, None)];
                if conn.expanded {
                    for table_idx in 0..conn.tables.len() {
                        v.push((conn_idx, Some(table_idx)));
                    }
                }
                v
            })
            .collect();

        if items.is_empty() {
            return;
        }

        let current = items
            .iter()
            .position(|(c, t)| *c == self.selected_connection_idx && *t == self.selected_table_idx)
            .unwrap_or(0);

        let new_idx = if current + 1 >= items.len() {
            0
        } else {
            current + 1
        };
        let (conn_idx, table_idx) = items[new_idx];
        self.selected_connection_idx = conn_idx;
        self.selected_table_idx = table_idx;

        // Fetch table details if a table is selected
        if table_idx.is_some() {
            self.fetch_table_details_if_needed(proj_idx);
        }
    }

    /// Navigate data table by the given delta (positive = down, negative = up)
    ///
    /// Navigation is constrained within the current page boundaries to prevent
    /// the row index from going beyond what is displayed on the current page.
    pub(crate) fn navigate_data_table(&mut self, delta: i32) {
        if let Some(result) = &self.result {
            if result.rows.is_empty() {
                return;
            }
            let row_count = result.rows.len();
            let current = self.data_table_state.selected().unwrap_or(0);

            // Get page boundaries from pagination state
            let page_start = self.pagination.start_index();
            let page_end = self.pagination.end_index().min(row_count);

            let new_idx = if delta < 0 {
                // Moving up: don't go below page start
                current.saturating_sub((-delta) as usize).max(page_start)
            } else {
                // Moving down: don't go beyond page end - 1 (or row_count - 1)
                let max_idx = page_end.saturating_sub(1);
                (current + delta as usize).min(max_idx)
            };
            self.data_table_state.select(Some(new_idx));
        }
    }
}
