//! Search modal states for connections, tables, and unified search

use crate::model::{Connection, Table};

/// Search modal for filtering connections within a project
#[derive(Debug, Clone, Default)]
pub struct SearchConnectionModal {
    pub query: String,
    pub filtered_indices: Vec<usize>,
    pub selected_idx: usize,
}

impl SearchConnectionModal {
    pub fn with_all_connections(connection_count: usize) -> Self {
        Self {
            query: String::new(),
            filtered_indices: (0..connection_count).collect(),
            selected_idx: 0,
        }
    }

    pub fn update_filter(&mut self, connections: &[Connection]) {
        let query_lower = self.query.to_lowercase();
        self.filtered_indices = connections
            .iter()
            .enumerate()
            .filter(|(_, c)| self.query.is_empty() || c.name.to_lowercase().contains(&query_lower))
            .map(|(idx, _)| idx)
            .collect();

        // Adjust selected index if needed
        if self.selected_idx >= self.filtered_indices.len() {
            self.selected_idx = self.filtered_indices.len().saturating_sub(1);
        }
    }

    pub fn selected_connection_idx(&self) -> Option<usize> {
        self.filtered_indices.get(self.selected_idx).copied()
    }

    pub fn navigate_up(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected_idx > 0 {
                self.selected_idx -= 1;
            } else {
                self.selected_idx = self.filtered_indices.len() - 1;
            }
        }
    }

    pub fn navigate_down(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected_idx + 1 < self.filtered_indices.len() {
                self.selected_idx += 1;
            } else {
                self.selected_idx = 0;
            }
        }
    }
}

/// Search modal for filtering tables within a connection
#[derive(Debug, Clone, Default)]
pub struct SearchTableModal {
    pub query: String,
    pub filtered_indices: Vec<usize>,
    pub selected_idx: usize,
}

impl SearchTableModal {
    pub fn with_all_tables(table_count: usize) -> Self {
        Self {
            query: String::new(),
            filtered_indices: (0..table_count).collect(),
            selected_idx: 0,
        }
    }

    pub fn update_filter(&mut self, tables: &[Table]) {
        let query_lower = self.query.to_lowercase();
        self.filtered_indices = tables
            .iter()
            .enumerate()
            .filter(|(_, t)| self.query.is_empty() || t.name.to_lowercase().contains(&query_lower))
            .map(|(idx, _)| idx)
            .collect();

        // Adjust selected index if needed
        if self.selected_idx >= self.filtered_indices.len() {
            self.selected_idx = self.filtered_indices.len().saturating_sub(1);
        }
    }

    pub fn selected_table_idx(&self) -> Option<usize> {
        self.filtered_indices.get(self.selected_idx).copied()
    }

    pub fn navigate_up(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected_idx > 0 {
                self.selected_idx -= 1;
            } else {
                self.selected_idx = self.filtered_indices.len() - 1;
            }
        }
    }

    pub fn navigate_down(&mut self) {
        if !self.filtered_indices.is_empty() {
            if self.selected_idx + 1 < self.filtered_indices.len() {
                self.selected_idx += 1;
            } else {
                self.selected_idx = 0;
            }
        }
    }
}

/// Which section is currently focused in the unified search modal
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum UnifiedSearchSection {
    #[default]
    Connections,
    Tables,
}

/// Unified search modal for searching both connections and tables
#[derive(Debug, Clone, Default)]
pub struct UnifiedSearchModal {
    pub query: String,
    pub filtered_connection_indices: Vec<usize>,
    pub filtered_table_indices: Vec<usize>,
    pub selected_connection_idx: usize,
    pub selected_table_idx: usize,
    pub active_section: UnifiedSearchSection,
    pub tables_first: bool, // true if tables section should be displayed first
}

impl UnifiedSearchModal {
    /// Create a new unified search modal
    /// - `connection_count`: number of connections to search
    /// - `table_count`: number of tables to search
    /// - `tables_first`: if true, tables section is displayed first and selected by default
    pub fn new(connection_count: usize, table_count: usize, tables_first: bool) -> Self {
        Self {
            query: String::new(),
            filtered_connection_indices: (0..connection_count).collect(),
            filtered_table_indices: (0..table_count).collect(),
            selected_connection_idx: 0,
            selected_table_idx: 0,
            active_section: if tables_first {
                UnifiedSearchSection::Tables
            } else {
                UnifiedSearchSection::Connections
            },
            tables_first,
        }
    }

    /// Update the filter based on the current query
    pub fn update_filter(&mut self, connections: &[Connection], tables: &[Table]) {
        let query_lower = self.query.to_lowercase();

        // Filter connections
        self.filtered_connection_indices = connections
            .iter()
            .enumerate()
            .filter(|(_, c)| self.query.is_empty() || c.name.to_lowercase().contains(&query_lower))
            .map(|(idx, _)| idx)
            .collect();

        // Filter tables
        self.filtered_table_indices = tables
            .iter()
            .enumerate()
            .filter(|(_, t)| self.query.is_empty() || t.name.to_lowercase().contains(&query_lower))
            .map(|(idx, _)| idx)
            .collect();

        // Adjust selected indices if needed
        if self.selected_connection_idx >= self.filtered_connection_indices.len() {
            self.selected_connection_idx = self.filtered_connection_indices.len().saturating_sub(1);
        }
        if self.selected_table_idx >= self.filtered_table_indices.len() {
            self.selected_table_idx = self.filtered_table_indices.len().saturating_sub(1);
        }
    }

    /// Switch between connections and tables sections
    pub fn switch_section(&mut self) {
        self.active_section = match self.active_section {
            UnifiedSearchSection::Connections => UnifiedSearchSection::Tables,
            UnifiedSearchSection::Tables => UnifiedSearchSection::Connections,
        };
    }

    /// Navigate up within the current section
    pub fn navigate_up(&mut self) {
        match self.active_section {
            UnifiedSearchSection::Connections => {
                if !self.filtered_connection_indices.is_empty() {
                    if self.selected_connection_idx > 0 {
                        self.selected_connection_idx -= 1;
                    } else {
                        self.selected_connection_idx = self.filtered_connection_indices.len() - 1;
                    }
                }
            }
            UnifiedSearchSection::Tables => {
                if !self.filtered_table_indices.is_empty() {
                    if self.selected_table_idx > 0 {
                        self.selected_table_idx -= 1;
                    } else {
                        self.selected_table_idx = self.filtered_table_indices.len() - 1;
                    }
                }
            }
        }
    }

    /// Navigate down within the current section
    pub fn navigate_down(&mut self) {
        match self.active_section {
            UnifiedSearchSection::Connections => {
                if !self.filtered_connection_indices.is_empty() {
                    if self.selected_connection_idx + 1 < self.filtered_connection_indices.len() {
                        self.selected_connection_idx += 1;
                    } else {
                        self.selected_connection_idx = 0;
                    }
                }
            }
            UnifiedSearchSection::Tables => {
                if !self.filtered_table_indices.is_empty() {
                    if self.selected_table_idx + 1 < self.filtered_table_indices.len() {
                        self.selected_table_idx += 1;
                    } else {
                        self.selected_table_idx = 0;
                    }
                }
            }
        }
    }

    /// Get the selected connection index (original index, not filtered)
    pub fn selected_connection(&self) -> Option<usize> {
        self.filtered_connection_indices
            .get(self.selected_connection_idx)
            .copied()
    }

    /// Get the selected table index (original index, not filtered)
    pub fn selected_table(&self) -> Option<usize> {
        self.filtered_table_indices
            .get(self.selected_table_idx)
            .copied()
    }

    /// Get the count of filtered connections
    pub fn connection_count(&self) -> usize {
        self.filtered_connection_indices.len()
    }

    /// Get the count of filtered tables
    pub fn table_count(&self) -> usize {
        self.filtered_table_indices.len()
    }
}
