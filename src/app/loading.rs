//! Loading state management for async DB operations

#![allow(dead_code)] // Methods will be used for UI loading indicators

/// Tracks loading states for various async database operations
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct LoadingState {
    /// Connection index currently fetching tables list
    pub fetching_tables: Option<usize>,
    /// (project_idx, connection_idx, table_idx) currently fetching table details
    pub fetching_details: Option<(usize, usize, usize)>,
    /// Whether a query is currently executing
    pub executing_query: bool,
    /// Status message to display
    pub message: Option<String>,
}

impl LoadingState {
    /// Returns true if any loading operation is in progress
    pub fn is_loading(&self) -> bool {
        self.fetching_tables.is_some() || self.fetching_details.is_some() || self.executing_query
    }

    /// Clear all loading states
    pub fn clear(&mut self) {
        self.fetching_tables = None;
        self.fetching_details = None;
        self.executing_query = false;
        self.message = None;
    }

    /// Set tables fetching state for a connection
    pub fn start_fetching_tables(&mut self, conn_idx: usize) {
        self.fetching_tables = Some(conn_idx);
        self.message = Some("Loading tables...".to_string());
    }

    /// Set table details fetching state
    pub fn start_fetching_details(&mut self, proj_idx: usize, conn_idx: usize, table_idx: usize) {
        self.fetching_details = Some((proj_idx, conn_idx, table_idx));
        self.message = Some("Loading table details...".to_string());
    }

    /// Set query executing state
    pub fn start_executing_query(&mut self) {
        self.executing_query = true;
        self.message = Some("Executing query...".to_string());
    }

    /// Check if any table details fetch is in progress
    pub fn is_fetching_details(&self) -> bool {
        self.fetching_details.is_some()
    }

    /// Check if a specific connection is fetching tables
    pub fn is_fetching_tables_for(&self, conn_idx: usize) -> bool {
        self.fetching_tables == Some(conn_idx)
    }

    /// Check if specific table details are being fetched
    pub fn is_fetching_details_for(
        &self,
        proj_idx: usize,
        conn_idx: usize,
        table_idx: usize,
    ) -> bool {
        self.fetching_details == Some((proj_idx, conn_idx, table_idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_not_loading() {
        let state = LoadingState::default();
        assert!(!state.is_loading());
        assert!(state.fetching_tables.is_none());
        assert!(state.fetching_details.is_none());
        assert!(!state.executing_query);
        assert!(state.message.is_none());
    }

    #[test]
    fn test_is_loading_when_fetching_tables() {
        let mut state = LoadingState::default();
        state.start_fetching_tables(0);

        assert!(state.is_loading());
        assert_eq!(state.fetching_tables, Some(0));
        assert!(state.message.is_some());
    }

    #[test]
    fn test_is_loading_when_fetching_details() {
        let mut state = LoadingState::default();
        state.start_fetching_details(0, 1, 2);

        assert!(state.is_loading());
        assert_eq!(state.fetching_details, Some((0, 1, 2)));
        assert!(state.message.is_some());
    }

    #[test]
    fn test_is_loading_when_executing_query() {
        let mut state = LoadingState::default();
        state.start_executing_query();

        assert!(state.is_loading());
        assert!(state.executing_query);
        assert!(state.message.is_some());
    }

    #[test]
    fn test_clear_resets_all_states() {
        let mut state = LoadingState {
            fetching_tables: Some(0),
            fetching_details: Some((0, 1, 2)),
            executing_query: true,
            message: Some("test".to_string()),
        };

        state.clear();

        assert!(!state.is_loading());
        assert!(state.fetching_tables.is_none());
        assert!(state.fetching_details.is_none());
        assert!(!state.executing_query);
        assert!(state.message.is_none());
    }

    #[test]
    fn test_is_fetching_tables_for() {
        let mut state = LoadingState::default();
        state.start_fetching_tables(5);

        assert!(state.is_fetching_tables_for(5));
        assert!(!state.is_fetching_tables_for(0));
        assert!(!state.is_fetching_tables_for(10));
    }

    #[test]
    fn test_is_fetching_details_for() {
        let mut state = LoadingState::default();
        state.start_fetching_details(1, 2, 3);

        assert!(state.is_fetching_details_for(1, 2, 3));
        assert!(!state.is_fetching_details_for(0, 2, 3));
        assert!(!state.is_fetching_details_for(1, 0, 3));
        assert!(!state.is_fetching_details_for(1, 2, 0));
    }

    #[test]
    fn test_multiple_loading_states() {
        // Start with multiple operations in progress
        let mut state = LoadingState {
            fetching_tables: Some(0),
            executing_query: true,
            ..Default::default()
        };

        assert!(state.is_loading());

        // Clear only one
        state.fetching_tables = None;
        assert!(state.is_loading()); // Still loading due to executing_query

        state.executing_query = false;
        assert!(!state.is_loading());
    }
}
