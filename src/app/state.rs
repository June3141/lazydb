//! Core App state and update logic

use ratatui::widgets::TableState;

use crate::db::DbWorkerHandle;
use crate::message::Message;
use crate::model::{Connection, Pagination, Project, QueryHistory, QueryResult, Table};

use super::enums::{Focus, MainPanelTab, SchemaSubTab, SidebarMode};
use super::loading::LoadingState;
use super::modal_fields::ConfirmModalField;
use super::modals::{
    AddConnectionModal, ColumnVisibilityModal, DeleteProjectModal, HistoryModal, ModalState,
    ProjectModal, SearchConnectionModal, SearchProjectModal, SearchTableModal, UnifiedSearchModal,
};
use super::visibility::ColumnVisibilitySettings;

/// Main application state
pub struct App {
    pub projects: Vec<Project>,
    pub sidebar_mode: SidebarMode,
    pub selected_project_idx: usize,
    pub selected_connection_idx: usize,
    pub selected_table_idx: Option<usize>,
    pub query: String,
    pub result: Option<QueryResult>,
    pub pagination: Pagination,
    pub focus: Focus,
    pub panel_tab: MainPanelTab,
    pub schema_sub_tab: SchemaSubTab,
    pub status_message: String,
    pub modal_state: ModalState,
    /// Query execution history
    pub query_history: QueryHistory,
    /// Flag indicating that history has been modified and should be saved
    pub history_dirty: bool,
    /// Data table scroll state for navigation
    pub data_table_state: TableState,
    /// Column visibility settings for schema sub-tabs
    pub column_visibility: ColumnVisibilitySettings,
    /// Handle to the background DB worker thread
    pub(crate) db_worker: Option<DbWorkerHandle>,
    /// Current loading state for async operations
    pub loading: LoadingState,
    /// Counter for generating unique request IDs
    pub(crate) next_request_id: u64,
    /// Pending query info for history (conn_name, database, query)
    pub(crate) pending_query_info: Option<(String, String, String)>,
}

impl App {
    /// Create a new App with the given projects
    #[allow(dead_code)]
    pub fn new(projects: Vec<Project>) -> Self {
        App {
            projects,
            sidebar_mode: SidebarMode::Projects,
            selected_project_idx: 0,
            selected_connection_idx: 0,
            selected_table_idx: None,
            query: String::new(),
            result: None,
            pagination: Pagination::default(),
            focus: Focus::Sidebar,
            panel_tab: MainPanelTab::Schema,
            schema_sub_tab: SchemaSubTab::default(),
            status_message: "Ready".to_string(),
            modal_state: ModalState::None,
            query_history: QueryHistory::new(),
            history_dirty: false,
            data_table_state: TableState::default(),
            column_visibility: ColumnVisibilitySettings::default(),
            db_worker: None,
            loading: LoadingState::default(),
            next_request_id: 0,
            pending_query_info: None,
        }
    }

    /// Create a new App with the given projects and history
    pub fn with_history(projects: Vec<Project>, history: QueryHistory) -> Self {
        App {
            projects,
            sidebar_mode: SidebarMode::Projects,
            selected_project_idx: 0,
            selected_connection_idx: 0,
            selected_table_idx: None,
            query: String::new(),
            result: None,
            pagination: Pagination::default(),
            focus: Focus::Sidebar,
            panel_tab: MainPanelTab::Schema,
            schema_sub_tab: SchemaSubTab::default(),
            status_message: "Ready".to_string(),
            modal_state: ModalState::None,
            query_history: history,
            history_dirty: false,
            data_table_state: TableState::default(),
            column_visibility: ColumnVisibilitySettings::default(),
            db_worker: None,
            loading: LoadingState::default(),
            next_request_id: 0,
            pending_query_info: None,
        }
    }

    /// Check if a modal is currently open
    pub fn is_modal_open(&self) -> bool {
        !matches!(self.modal_state, ModalState::None)
    }

    /// Check if the currently selected connection is expanded
    pub fn is_connection_expanded(&self) -> bool {
        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
            if let Some(project) = self.projects.get(proj_idx) {
                if let Some(conn) = project.connections.get(self.selected_connection_idx) {
                    return conn.expanded;
                }
            }
        }
        false
    }

    /// Get currently selected table (if any)
    pub fn selected_table_info(&self) -> Option<&Table> {
        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
            let project = self.projects.get(proj_idx)?;
            let conn = project.connections.get(self.selected_connection_idx)?;
            let table_idx = self.selected_table_idx?;
            conn.tables.get(table_idx)
        } else {
            None
        }
    }

    /// Get currently selected connection (if any)
    pub fn selected_connection_info(&self) -> Option<&Connection> {
        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
            let project = self.projects.get(proj_idx)?;
            project.connections.get(self.selected_connection_idx)
        } else {
            None
        }
    }

    /// Get currently selected project
    pub fn selected_project_info(&self) -> Option<&Project> {
        match self.sidebar_mode {
            SidebarMode::Projects => self.projects.get(self.selected_project_idx),
            SidebarMode::Connections(proj_idx) => self.projects.get(proj_idx),
        }
    }

    /// Get all tables in current connection (for ER diagram)
    pub fn current_connection_tables(&self) -> Option<&[Table]> {
        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
            let project = self.projects.get(proj_idx)?;
            let conn = project.connections.get(self.selected_connection_idx)?;
            Some(&conn.tables)
        } else {
            None
        }
    }

    /// Update app state based on message. Returns true if app should quit.
    pub fn update(&mut self, message: Message) -> bool {
        match message {
            Message::Quit => return true,

            // Navigation messages (handled by handlers/navigation.rs)
            Message::NavigateUp => {
                if self.focus == Focus::Sidebar {
                    self.navigate_up();
                }
            }
            Message::NavigateDown => {
                if self.focus == Focus::Sidebar {
                    self.navigate_down();
                }
            }

            // Focus messages
            Message::NextFocus => {
                self.focus = match self.focus {
                    Focus::Sidebar => Focus::QueryEditor,
                    Focus::QueryEditor => Focus::MainPanel,
                    Focus::MainPanel => Focus::Sidebar,
                };
            }
            Message::PrevFocus => {
                self.focus = match self.focus {
                    Focus::Sidebar => Focus::MainPanel,
                    Focus::QueryEditor => Focus::Sidebar,
                    Focus::MainPanel => Focus::QueryEditor,
                };
            }
            Message::FocusLeft => {
                if self.focus == Focus::QueryEditor || self.focus == Focus::MainPanel {
                    self.focus = Focus::Sidebar;
                }
            }
            Message::FocusRight => {
                if self.focus == Focus::Sidebar {
                    self.focus = Focus::QueryEditor;
                }
            }
            Message::FocusUp => {
                if self.focus == Focus::MainPanel {
                    self.focus = Focus::QueryEditor;
                }
            }
            Message::FocusDown => {
                if self.focus == Focus::QueryEditor {
                    self.focus = Focus::MainPanel;
                }
            }

            // Sidebar actions (handled by handlers/sidebar.rs)
            Message::Activate => {
                if self.focus == Focus::Sidebar {
                    self.activate();
                }
            }
            Message::GoBack => {
                if self.focus == Focus::Sidebar {
                    self.go_back();
                }
            }

            // Tab switching
            Message::SwitchToSchema => {
                self.panel_tab = MainPanelTab::Schema;
            }
            Message::SwitchToData => {
                self.panel_tab = MainPanelTab::Data;
            }
            Message::SwitchToRelations => {
                self.panel_tab = MainPanelTab::Relations;
            }
            Message::SwitchToColumns => {
                self.panel_tab = MainPanelTab::Schema;
                self.schema_sub_tab = SchemaSubTab::Columns;
            }
            Message::SwitchToIndexes => {
                self.panel_tab = MainPanelTab::Schema;
                self.schema_sub_tab = SchemaSubTab::Indexes;
            }
            Message::SwitchToForeignKeys => {
                self.panel_tab = MainPanelTab::Schema;
                self.schema_sub_tab = SchemaSubTab::ForeignKeys;
            }
            Message::SwitchToConstraints => {
                self.panel_tab = MainPanelTab::Schema;
                self.schema_sub_tab = SchemaSubTab::Constraints;
            }
            Message::SwitchToTriggers => {
                self.panel_tab = MainPanelTab::Schema;
                self.schema_sub_tab = SchemaSubTab::Triggers;
            }
            Message::SwitchToDefinition => {
                if let Some(table) = self.selected_table_info() {
                    if table.table_type.is_view() {
                        self.panel_tab = MainPanelTab::Schema;
                        self.schema_sub_tab = SchemaSubTab::Definition;
                    }
                }
            }

            // Modal open messages
            Message::OpenAddConnectionModal => {
                self.modal_state = ModalState::AddConnection(AddConnectionModal::default());
            }
            Message::OpenAddProjectModal => {
                self.modal_state = ModalState::AddProject(ProjectModal::default());
            }
            Message::OpenEditProjectModal => {
                if let SidebarMode::Projects = self.sidebar_mode {
                    if let Some(project) = self.projects.get(self.selected_project_idx) {
                        self.modal_state = ModalState::EditProject(
                            self.selected_project_idx,
                            ProjectModal::with_name(&project.name),
                        );
                    }
                }
            }
            Message::DeleteProject => {
                if let SidebarMode::Projects = self.sidebar_mode {
                    if let Some(project) = self.projects.get(self.selected_project_idx) {
                        self.modal_state = ModalState::DeleteProject(DeleteProjectModal {
                            project_idx: self.selected_project_idx,
                            project_name: project.name.clone(),
                            focused_field: ConfirmModalField::ButtonCancel,
                        });
                    }
                }
            }
            Message::OpenSearchProjectModal => {
                if let SidebarMode::Projects = self.sidebar_mode {
                    self.modal_state = ModalState::SearchProject(
                        SearchProjectModal::with_all_projects(self.projects.len()),
                    );
                }
            }
            Message::OpenSearchConnectionModal => {
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        self.modal_state = ModalState::SearchConnection(
                            SearchConnectionModal::with_all_connections(project.connections.len()),
                        );
                    }
                }
            }
            Message::OpenSearchTableModal => {
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        if let Some(conn) = project.connections.get(self.selected_connection_idx) {
                            if conn.expanded && !conn.tables.is_empty() {
                                self.modal_state = ModalState::SearchTable(
                                    SearchTableModal::with_all_tables(conn.tables.len()),
                                );
                            }
                        }
                    }
                }
            }
            Message::OpenUnifiedSearchModal => {
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        let connection_count = project.connections.len();
                        let tables_first = self.is_connection_expanded();
                        let table_count = if let Some(conn) =
                            project.connections.get(self.selected_connection_idx)
                        {
                            if conn.expanded {
                                conn.tables.len()
                            } else {
                                0
                            }
                        } else {
                            0
                        };
                        self.modal_state = ModalState::UnifiedSearch(UnifiedSearchModal::new(
                            connection_count,
                            table_count,
                            tables_first,
                        ));
                    }
                }
            }
            Message::OpenColumnVisibilityModal => {
                if self.panel_tab == MainPanelTab::Schema {
                    self.modal_state = ModalState::ColumnVisibility(ColumnVisibilityModal::new(
                        self.schema_sub_tab,
                    ));
                }
            }
            Message::OpenHistoryModal => {
                if !self.query_history.is_empty() {
                    self.modal_state = ModalState::History(HistoryModal::default());
                } else {
                    self.status_message = "No query history".to_string();
                }
            }

            // Search confirm messages
            Message::SearchConfirm => {
                self.handle_search_confirm();
            }
            Message::SearchConnectionConfirm => {
                self.handle_search_connection_confirm();
            }
            Message::TableSearchConfirm => {
                self.handle_table_search_confirm();
            }
            Message::UnifiedSearchConfirm => {
                self.handle_unified_search_confirm();
            }
            Message::UnifiedSearchSwitchSection => {
                if let ModalState::UnifiedSearch(modal) = &mut self.modal_state {
                    modal.switch_section();
                }
            }

            // Column visibility
            Message::ToggleColumnVisibility => {
                self.toggle_column_visibility();
            }

            // Modal control
            Message::CloseModal => {
                self.modal_state = ModalState::None;
            }
            Message::ModalConfirm => {
                self.handle_modal_confirm();
            }
            Message::ModalInputChar(c) => {
                self.handle_modal_input_char(c);
            }
            Message::ModalInputBackspace => {
                self.handle_modal_backspace();
            }
            Message::ModalNextField => {
                self.handle_modal_next_field();
            }
            Message::ModalPrevField => {
                self.handle_modal_prev_field();
            }

            // History messages
            Message::HistoryNavigateUp => {
                self.handle_history_navigate_up();
            }
            Message::HistoryNavigateDown => {
                self.handle_history_navigate_down();
            }
            Message::HistorySelectEntry => {
                self.handle_history_select_entry();
            }
            Message::ClearHistory => {
                self.handle_clear_history();
            }

            // Query input modal messages
            Message::OpenQueryInputModal => {
                self.open_query_input_modal();
            }
            Message::QueryInputChar(c) => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.insert_char(c);
                }
            }
            Message::QueryInputBackspace => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.delete_char_before();
                }
            }
            Message::QueryInputDelete => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.delete_char_at();
                }
            }
            Message::QueryInputNewline => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.insert_newline();
                }
            }
            Message::QueryInputCursorLeft => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.move_cursor_left();
                }
            }
            Message::QueryInputCursorRight => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.move_cursor_right();
                }
            }
            Message::QueryInputCursorUp => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.move_cursor_up();
                }
            }
            Message::QueryInputCursorDown => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.move_cursor_down();
                }
            }
            Message::QueryInputCursorHome => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.move_cursor_home();
                }
            }
            Message::QueryInputCursorEnd => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.move_cursor_end();
                }
            }
            Message::QueryInputClear => {
                if let ModalState::QueryInput(modal) = &mut self.modal_state {
                    modal.clear();
                }
            }
            Message::QueryInputExecute => {
                self.execute_query_from_modal();
            }

            // Pagination messages
            Message::PageNext => {
                self.pagination.next_page();
            }
            Message::PagePrev => {
                self.pagination.prev_page();
            }
            Message::PageFirst => {
                self.pagination.first_page();
            }
            Message::PageLast => {
                self.pagination.last_page();
            }
            Message::PageSizeCycle => {
                self.pagination.cycle_page_size();
            }

            // Data table navigation (handled by handlers/navigation.rs)
            Message::DataTableUp => {
                self.navigate_data_table(-1);
            }
            Message::DataTableDown => {
                self.navigate_data_table(1);
            }
            Message::DataTablePageUp => {
                self.navigate_data_table(-10);
            }
            Message::DataTablePageDown => {
                self.navigate_data_table(10);
            }
            Message::DataTableFirst => {
                if self.result.is_some() {
                    self.data_table_state.select(Some(0));
                }
            }
            Message::DataTableLast => {
                if let Some(result) = &self.result {
                    if !result.rows.is_empty() {
                        self.data_table_state.select(Some(result.rows.len() - 1));
                    }
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::modals::UnifiedSearchSection;
    use crate::model::schema::TableType;

    fn create_test_app_with_result(row_count: usize) -> App {
        let mut app = App::new(vec![]);
        if row_count > 0 {
            app.result = Some(QueryResult {
                columns: vec!["id".to_string(), "name".to_string()],
                rows: (0..row_count)
                    .map(|i| vec![i.to_string(), format!("row_{}", i)])
                    .collect(),
                total_rows: row_count,
                execution_time_ms: 0,
            });
            app.pagination = Pagination::new(row_count);
        }
        app
    }

    fn create_test_tables() -> Vec<Table> {
        vec![
            Table {
                name: "users".to_string(),
                schema: Some("public".to_string()),
                table_type: TableType::BaseTable,
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![],
                constraints: vec![],
                triggers: vec![],
                row_count: 0,
                size_bytes: 0,
                comment: None,
                details_loaded: false,
                view_definition: None,
            },
            Table {
                name: "orders".to_string(),
                schema: Some("public".to_string()),
                table_type: TableType::BaseTable,
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![],
                constraints: vec![],
                triggers: vec![],
                row_count: 0,
                size_bytes: 0,
                comment: None,
                details_loaded: false,
                view_definition: None,
            },
            Table {
                name: "user_sessions".to_string(),
                schema: Some("public".to_string()),
                table_type: TableType::BaseTable,
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![],
                constraints: vec![],
                triggers: vec![],
                row_count: 0,
                size_bytes: 0,
                comment: None,
                details_loaded: false,
                view_definition: None,
            },
            Table {
                name: "products".to_string(),
                schema: Some("public".to_string()),
                table_type: TableType::BaseTable,
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![],
                constraints: vec![],
                triggers: vec![],
                row_count: 0,
                size_bytes: 0,
                comment: None,
                details_loaded: false,
                view_definition: None,
            },
        ]
    }

    #[test]
    fn test_navigate_data_table_down() {
        let mut app = create_test_app_with_result(10);
        app.data_table_state.select(Some(0));

        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(1));

        app.navigate_data_table(3);
        assert_eq!(app.data_table_state.selected(), Some(4));
    }

    #[test]
    fn test_navigate_data_table_up() {
        let mut app = create_test_app_with_result(10);
        app.data_table_state.select(Some(5));

        app.navigate_data_table(-1);
        assert_eq!(app.data_table_state.selected(), Some(4));

        app.navigate_data_table(-2);
        assert_eq!(app.data_table_state.selected(), Some(2));
    }

    #[test]
    fn test_navigate_data_table_boundary_first_row() {
        let mut app = create_test_app_with_result(10);
        app.data_table_state.select(Some(0));

        app.navigate_data_table(-1);
        assert_eq!(app.data_table_state.selected(), Some(0));

        app.navigate_data_table(-10);
        assert_eq!(app.data_table_state.selected(), Some(0));
    }

    #[test]
    fn test_navigate_data_table_boundary_last_row() {
        let mut app = create_test_app_with_result(10);
        app.data_table_state.select(Some(9));

        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(9));

        app.navigate_data_table(100);
        assert_eq!(app.data_table_state.selected(), Some(9));
    }

    #[test]
    fn test_navigate_data_table_empty_result() {
        let mut app = create_test_app_with_result(0);
        app.data_table_state.select(Some(0));

        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(0));
    }

    #[test]
    fn test_navigate_data_table_no_result() {
        let mut app = App::new(vec![]);
        app.data_table_state.select(Some(0));

        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(0));
    }

    #[test]
    fn test_navigate_data_table_no_selection() {
        let mut app = create_test_app_with_result(10);

        app.navigate_data_table(3);
        assert_eq!(app.data_table_state.selected(), Some(3));
    }

    #[test]
    fn test_navigate_data_table_page_navigation() {
        let mut app = create_test_app_with_result(100);
        app.pagination.next_page();
        app.data_table_state.select(Some(50));

        app.navigate_data_table(10);
        assert_eq!(app.data_table_state.selected(), Some(60));

        app.navigate_data_table(-10);
        assert_eq!(app.data_table_state.selected(), Some(50));
    }

    #[test]
    fn test_search_table_modal_with_all_tables_initializes_correctly() {
        let modal = SearchTableModal::with_all_tables(5);

        assert_eq!(modal.query, "");
        assert_eq!(modal.filtered_indices, vec![0, 1, 2, 3, 4]);
        assert_eq!(modal.selected_idx, 0);
    }

    #[test]
    fn test_search_table_modal_with_all_tables_empty() {
        let modal = SearchTableModal::with_all_tables(0);

        assert_eq!(modal.query, "");
        assert!(modal.filtered_indices.is_empty());
        assert_eq!(modal.selected_idx, 0);
    }

    #[test]
    fn test_search_table_modal_update_filter_matches_partial_name() {
        let tables = create_test_tables();
        let mut modal = SearchTableModal::with_all_tables(tables.len());

        modal.query = "user".to_string();
        modal.update_filter(&tables);

        assert_eq!(modal.filtered_indices, vec![0, 2]);
    }

    #[test]
    fn test_search_table_modal_update_filter_case_insensitive() {
        let tables = create_test_tables();
        let mut modal = SearchTableModal::with_all_tables(tables.len());

        modal.query = "USER".to_string();
        modal.update_filter(&tables);

        assert_eq!(modal.filtered_indices, vec![0, 2]);
    }

    #[test]
    fn test_search_table_modal_update_filter_empty_query_shows_all() {
        let tables = create_test_tables();
        let mut modal = SearchTableModal::with_all_tables(tables.len());

        modal.query = "".to_string();
        modal.update_filter(&tables);

        assert_eq!(modal.filtered_indices, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_search_table_modal_update_filter_no_matches() {
        let tables = create_test_tables();
        let mut modal = SearchTableModal::with_all_tables(tables.len());

        modal.query = "nonexistent".to_string();
        modal.update_filter(&tables);

        assert!(modal.filtered_indices.is_empty());
    }

    #[test]
    fn test_search_table_modal_update_filter_adjusts_selected_idx() {
        let tables = create_test_tables();
        let mut modal = SearchTableModal::with_all_tables(tables.len());

        modal.selected_idx = 3;
        modal.query = "user".to_string();
        modal.update_filter(&tables);

        assert_eq!(modal.selected_idx, 1);
    }

    #[test]
    fn test_search_table_modal_selected_table_idx_returns_correct_index() {
        let tables = create_test_tables();
        let mut modal = SearchTableModal::with_all_tables(tables.len());

        modal.query = "user".to_string();
        modal.update_filter(&tables);
        modal.selected_idx = 1;

        assert_eq!(modal.selected_table_idx(), Some(2));
    }

    #[test]
    fn test_search_table_modal_selected_table_idx_returns_none_when_empty() {
        let tables = create_test_tables();
        let mut modal = SearchTableModal::with_all_tables(tables.len());

        modal.query = "nonexistent".to_string();
        modal.update_filter(&tables);

        assert_eq!(modal.selected_table_idx(), None);
    }

    #[test]
    fn test_search_table_modal_navigate_down_wraps_around() {
        let mut modal = SearchTableModal::with_all_tables(3);

        modal.selected_idx = 2;
        modal.navigate_down();

        assert_eq!(modal.selected_idx, 0);
    }

    #[test]
    fn test_search_table_modal_navigate_up_wraps_around() {
        let mut modal = SearchTableModal::with_all_tables(3);

        modal.selected_idx = 0;
        modal.navigate_up();

        assert_eq!(modal.selected_idx, 2);
    }

    #[test]
    fn test_search_table_modal_navigate_on_empty_list() {
        let mut modal = SearchTableModal::with_all_tables(0);

        modal.navigate_down();
        assert_eq!(modal.selected_idx, 0);

        modal.navigate_up();
        assert_eq!(modal.selected_idx, 0);
    }

    #[test]
    fn test_search_table_modal_navigate_down_increments() {
        let mut modal = SearchTableModal::with_all_tables(5);

        modal.navigate_down();
        assert_eq!(modal.selected_idx, 1);

        modal.navigate_down();
        assert_eq!(modal.selected_idx, 2);
    }

    #[test]
    fn test_search_table_modal_navigate_up_decrements() {
        let mut modal = SearchTableModal::with_all_tables(5);
        modal.selected_idx = 3;

        modal.navigate_up();
        assert_eq!(modal.selected_idx, 2);

        modal.navigate_up();
        assert_eq!(modal.selected_idx, 1);
    }

    // UnifiedSearchModal tests

    fn create_test_connections() -> Vec<Connection> {
        vec![
            Connection {
                name: "postgres_local".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                username: "user".to_string(),
                password: "".to_string(),
                database: "db".to_string(),
                tables: vec![],
                expanded: false,
            },
            Connection {
                name: "postgres_prod".to_string(),
                host: "prod.example.com".to_string(),
                port: 5432,
                username: "user".to_string(),
                password: "".to_string(),
                database: "db".to_string(),
                tables: vec![],
                expanded: false,
            },
            Connection {
                name: "mysql_dev".to_string(),
                host: "localhost".to_string(),
                port: 3306,
                username: "user".to_string(),
                password: "".to_string(),
                database: "db".to_string(),
                tables: vec![],
                expanded: false,
            },
        ]
    }

    #[test]
    fn test_unified_search_modal_new_with_tables_priority() {
        let modal = UnifiedSearchModal::new(3, 4, true);

        assert_eq!(modal.query, "");
        assert_eq!(modal.filtered_connection_indices, vec![0, 1, 2]);
        assert_eq!(modal.filtered_table_indices, vec![0, 1, 2, 3]);
        assert_eq!(modal.selected_connection_idx, 0);
        assert_eq!(modal.selected_table_idx, 0);
        assert_eq!(modal.active_section, UnifiedSearchSection::Tables);
        assert!(modal.tables_first);
    }

    #[test]
    fn test_unified_search_modal_new_with_connections_priority() {
        let modal = UnifiedSearchModal::new(3, 4, false);

        assert_eq!(modal.active_section, UnifiedSearchSection::Connections);
        assert!(!modal.tables_first);
    }

    #[test]
    fn test_unified_search_modal_update_filter_connections() {
        let connections = create_test_connections();
        let tables = create_test_tables();
        let mut modal = UnifiedSearchModal::new(connections.len(), tables.len(), false);

        modal.query = "postgres".to_string();
        modal.update_filter(&connections, &tables);

        assert_eq!(modal.filtered_connection_indices, vec![0, 1]);
    }

    #[test]
    fn test_unified_search_modal_update_filter_tables() {
        let connections = create_test_connections();
        let tables = create_test_tables();
        let mut modal = UnifiedSearchModal::new(connections.len(), tables.len(), false);

        modal.query = "user".to_string();
        modal.update_filter(&connections, &tables);

        assert_eq!(modal.filtered_table_indices, vec![0, 2]);
    }

    #[test]
    fn test_unified_search_modal_update_filter_both() {
        let connections = create_test_connections();
        let tables = create_test_tables();
        let mut modal = UnifiedSearchModal::new(connections.len(), tables.len(), false);

        modal.query = "prod".to_string();
        modal.update_filter(&connections, &tables);

        assert_eq!(modal.filtered_connection_indices, vec![1]);
        assert_eq!(modal.filtered_table_indices, vec![3]);
    }

    #[test]
    fn test_unified_search_modal_switch_section() {
        let mut modal = UnifiedSearchModal::new(3, 4, false);
        assert_eq!(modal.active_section, UnifiedSearchSection::Connections);

        modal.switch_section();
        assert_eq!(modal.active_section, UnifiedSearchSection::Tables);

        modal.switch_section();
        assert_eq!(modal.active_section, UnifiedSearchSection::Connections);
    }

    #[test]
    fn test_unified_search_modal_navigate_within_connections() {
        let mut modal = UnifiedSearchModal::new(3, 4, false);
        modal.active_section = UnifiedSearchSection::Connections;

        modal.navigate_down();
        assert_eq!(modal.selected_connection_idx, 1);

        modal.navigate_down();
        assert_eq!(modal.selected_connection_idx, 2);

        modal.navigate_down();
        assert_eq!(modal.selected_connection_idx, 0);
    }

    #[test]
    fn test_unified_search_modal_navigate_within_tables() {
        let mut modal = UnifiedSearchModal::new(3, 4, true);
        modal.active_section = UnifiedSearchSection::Tables;

        modal.navigate_down();
        assert_eq!(modal.selected_table_idx, 1);

        modal.navigate_up();
        assert_eq!(modal.selected_table_idx, 0);

        modal.navigate_up();
        assert_eq!(modal.selected_table_idx, 3);
    }

    #[test]
    fn test_unified_search_modal_selected_connection() {
        let connections = create_test_connections();
        let tables = create_test_tables();
        let mut modal = UnifiedSearchModal::new(connections.len(), tables.len(), false);

        modal.query = "postgres".to_string();
        modal.update_filter(&connections, &tables);
        modal.active_section = UnifiedSearchSection::Connections;
        modal.selected_connection_idx = 1;

        assert_eq!(modal.selected_connection(), Some(1));
    }

    #[test]
    fn test_unified_search_modal_selected_table() {
        let connections = create_test_connections();
        let tables = create_test_tables();
        let mut modal = UnifiedSearchModal::new(connections.len(), tables.len(), false);

        modal.query = "user".to_string();
        modal.update_filter(&connections, &tables);
        modal.active_section = UnifiedSearchSection::Tables;
        modal.selected_table_idx = 1;

        assert_eq!(modal.selected_table(), Some(2));
    }

    #[test]
    fn test_unified_search_modal_connection_count_and_table_count() {
        let connections = create_test_connections();
        let tables = create_test_tables();
        let mut modal = UnifiedSearchModal::new(connections.len(), tables.len(), false);

        modal.query = "postgres".to_string();
        modal.update_filter(&connections, &tables);

        assert_eq!(modal.connection_count(), 2);
        assert_eq!(modal.table_count(), 0);
    }

    #[test]
    fn test_unified_search_modal_empty_filter() {
        let connections = create_test_connections();
        let tables = create_test_tables();
        let mut modal = UnifiedSearchModal::new(connections.len(), tables.len(), false);

        modal.query = "".to_string();
        modal.update_filter(&connections, &tables);

        assert_eq!(modal.filtered_connection_indices.len(), 3);
        assert_eq!(modal.filtered_table_indices.len(), 4);
    }

    #[test]
    fn test_navigate_data_table_respects_page_boundary_down() {
        let mut app = create_test_app_with_result(100);
        app.pagination = Pagination::new(100);
        app.pagination.page_size = 50;
        app.data_table_state.select(Some(49));

        app.navigate_data_table(1);

        assert_eq!(app.data_table_state.selected(), Some(49));
    }

    #[test]
    fn test_navigate_data_table_respects_page_boundary_up() {
        let mut app = create_test_app_with_result(100);
        app.pagination = Pagination::new(100);
        app.pagination.page_size = 50;
        app.pagination.next_page();
        app.data_table_state.select(Some(50));

        app.navigate_data_table(-1);

        assert_eq!(app.data_table_state.selected(), Some(50));
    }

    #[test]
    fn test_navigate_data_table_within_page_works_normally() {
        let mut app = create_test_app_with_result(100);
        app.pagination = Pagination::new(100);
        app.pagination.page_size = 50;
        app.data_table_state.select(Some(25));

        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(26));

        app.navigate_data_table(-1);
        assert_eq!(app.data_table_state.selected(), Some(25));

        app.navigate_data_table(10);
        assert_eq!(app.data_table_state.selected(), Some(35));
    }

    #[test]
    fn test_navigate_data_table_last_page_partial_rows() {
        let mut app = create_test_app_with_result(75);
        app.pagination = Pagination::new(75);
        app.pagination.page_size = 50;
        app.pagination.next_page();
        app.data_table_state.select(Some(74));

        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(74));
    }
}
