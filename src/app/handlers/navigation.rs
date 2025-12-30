//! Navigation handlers for sidebar and data table

use crate::app::enums::SidebarMode;
use crate::app::App;

/// Sidebar navigation item: (connection_idx, table_idx, routine_idx)
type NavItem = (usize, Option<usize>, Option<usize>);

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

    /// Build a flat list of navigation items for the connections view
    fn build_nav_items(&self, proj_idx: usize) -> Vec<NavItem> {
        let Some(project) = self.projects.get(proj_idx) else {
            return Vec::new();
        };

        project
            .connections
            .iter()
            .enumerate()
            .flat_map(|(conn_idx, conn)| {
                let mut v: Vec<NavItem> = vec![(conn_idx, None, None)];
                if conn.expanded {
                    // Add tables
                    for table_idx in 0..conn.tables.len() {
                        v.push((conn_idx, Some(table_idx), None));
                    }
                    // Add routines
                    for routine_idx in 0..conn.routines.len() {
                        v.push((conn_idx, None, Some(routine_idx)));
                    }
                }
                v
            })
            .collect()
    }

    /// Find current position in navigation items
    fn find_current_nav_position(&self, items: &[NavItem]) -> usize {
        items
            .iter()
            .position(|(c, t, r)| {
                *c == self.selected_connection_idx
                    && *t == self.selected_table_idx
                    && *r == self.selected_routine_idx
            })
            .unwrap_or(0)
    }

    /// Apply navigation selection
    fn apply_nav_selection(&mut self, item: NavItem, proj_idx: usize) {
        let (conn_idx, table_idx, routine_idx) = item;
        self.selected_connection_idx = conn_idx;
        self.selected_table_idx = table_idx;
        self.selected_routine_idx = routine_idx;

        // Fetch table details if a table is selected
        if table_idx.is_some() {
            self.fetch_table_details_if_needed(proj_idx);
        }
    }

    fn navigate_connections_up(&mut self, proj_idx: usize) {
        let items = self.build_nav_items(proj_idx);
        if items.is_empty() {
            return;
        }

        let current = self.find_current_nav_position(&items);
        let new_idx = if current == 0 {
            items.len() - 1
        } else {
            current - 1
        };

        self.apply_nav_selection(items[new_idx], proj_idx);
    }

    fn navigate_connections_down(&mut self, proj_idx: usize) {
        let items = self.build_nav_items(proj_idx);
        if items.is_empty() {
            return;
        }

        let current = self.find_current_nav_position(&items);
        let new_idx = if current + 1 >= items.len() {
            0
        } else {
            current + 1
        };

        self.apply_nav_selection(items[new_idx], proj_idx);
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
