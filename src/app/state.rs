//! Core App state and update logic

use ratatui::widgets::TableState;

use crate::db::{ConnectionParams, DbCommand, DbResponse, DbWorkerHandle};
use crate::message::Message;
use crate::model::{
    Connection, HistoryEntry, Pagination, Project, QueryHistory, QueryResult, Table,
};

use super::enums::{Focus, MainPanelTab, SchemaSubTab, SidebarMode};
use super::loading::LoadingState;
use super::modal_fields::{ConfirmModalField, ConnectionModalField, ProjectModalField};
use super::modals::{
    AddConnectionModal, ColumnVisibilityModal, DeleteProjectModal, HistoryModal, ModalState,
    ProjectModal, SearchConnectionModal, SearchProjectModal, SearchTableModal, UnifiedSearchModal,
    UnifiedSearchSection,
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
    pub main_panel_tab: MainPanelTab,
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
    db_worker: Option<DbWorkerHandle>,
    /// Current loading state for async operations
    pub loading: LoadingState,
    /// Counter for generating unique request IDs
    next_request_id: u64,
    /// Pending query info for history (conn_name, database, query)
    pending_query_info: Option<(String, String, String)>,
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
            main_panel_tab: MainPanelTab::Schema,
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
            main_panel_tab: MainPanelTab::Schema,
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

    /// Update app state based on message. Returns true if app should quit.
    pub fn update(&mut self, message: Message) -> bool {
        match message {
            Message::Quit => return true,

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

            Message::SwitchToSchema => {
                self.main_panel_tab = MainPanelTab::Schema;
            }

            Message::SwitchToData => {
                self.main_panel_tab = MainPanelTab::Data;
            }

            Message::SwitchToRelations => {
                self.main_panel_tab = MainPanelTab::Relations;
            }

            Message::SwitchToColumns => {
                self.main_panel_tab = MainPanelTab::Schema;
                self.schema_sub_tab = SchemaSubTab::Columns;
            }

            Message::SwitchToIndexes => {
                self.main_panel_tab = MainPanelTab::Schema;
                self.schema_sub_tab = SchemaSubTab::Indexes;
            }

            Message::SwitchToForeignKeys => {
                self.main_panel_tab = MainPanelTab::Schema;
                self.schema_sub_tab = SchemaSubTab::ForeignKeys;
            }

            Message::SwitchToConstraints => {
                self.main_panel_tab = MainPanelTab::Schema;
                self.schema_sub_tab = SchemaSubTab::Constraints;
            }

            Message::SwitchToDefinition => {
                // Only switch to Definition tab if current table is a view
                if let Some(table) = self.selected_table_info() {
                    if table.table_type.is_view() {
                        self.main_panel_tab = MainPanelTab::Schema;
                        self.schema_sub_tab = SchemaSubTab::Definition;
                    }
                }
            }

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

            Message::SearchConfirm => {
                if let ModalState::SearchProject(modal) = &self.modal_state {
                    if let Some(proj_idx) = modal.selected_project_idx() {
                        self.selected_project_idx = proj_idx;
                        self.status_message = format!("Selected: {}", self.projects[proj_idx].name);
                    }
                }
                self.modal_state = ModalState::None;
            }

            Message::SearchConnectionConfirm => {
                if let ModalState::SearchConnection(modal) = &self.modal_state {
                    if let Some(conn_idx) = modal.selected_connection_idx() {
                        self.selected_connection_idx = conn_idx;
                        self.selected_table_idx = None;
                        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                            if let Some(project) = self.projects.get(proj_idx) {
                                if let Some(conn) = project.connections.get(conn_idx) {
                                    self.status_message = format!("Selected: {}", conn.name);
                                }
                            }
                        }
                    }
                }
                self.modal_state = ModalState::None;
            }

            Message::TableSearchConfirm => {
                if let ModalState::SearchTable(modal) = &self.modal_state {
                    if let Some(table_idx) = modal.selected_table_idx() {
                        self.selected_table_idx = Some(table_idx);
                        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                            if let Some(project) = self.projects.get(proj_idx) {
                                if let Some(conn) =
                                    project.connections.get(self.selected_connection_idx)
                                {
                                    if let Some(table) = conn.tables.get(table_idx) {
                                        self.status_message = format!("Selected: {}", table.name);
                                    }
                                }
                            }
                        }
                    }
                }
                self.modal_state = ModalState::None;
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

            Message::UnifiedSearchConfirm => {
                if let ModalState::UnifiedSearch(modal) = &self.modal_state {
                    match modal.active_section {
                        UnifiedSearchSection::Connections => {
                            if let Some(conn_idx) = modal.selected_connection() {
                                self.selected_connection_idx = conn_idx;
                                self.selected_table_idx = None;
                                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                                    if let Some(project) = self.projects.get(proj_idx) {
                                        if let Some(conn) = project.connections.get(conn_idx) {
                                            self.status_message =
                                                format!("Selected: {}", conn.name);
                                        }
                                    }
                                }
                            }
                        }
                        UnifiedSearchSection::Tables => {
                            if let Some(table_idx) = modal.selected_table() {
                                self.selected_table_idx = Some(table_idx);
                                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                                    if let Some(project) = self.projects.get(proj_idx) {
                                        if let Some(conn) =
                                            project.connections.get(self.selected_connection_idx)
                                        {
                                            if let Some(table) = conn.tables.get(table_idx) {
                                                self.status_message =
                                                    format!("Selected: {}", table.name);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                self.modal_state = ModalState::None;
            }

            Message::UnifiedSearchSwitchSection => {
                if let ModalState::UnifiedSearch(modal) = &mut self.modal_state {
                    modal.switch_section();
                }
            }

            Message::OpenColumnVisibilityModal => {
                // Only open in Schema tab (not Data or Relations)
                if self.main_panel_tab == MainPanelTab::Schema {
                    self.modal_state = ModalState::ColumnVisibility(ColumnVisibilityModal::new(
                        self.schema_sub_tab,
                    ));
                }
            }

            Message::ToggleColumnVisibility => {
                if let ModalState::ColumnVisibility(modal) = &self.modal_state {
                    let idx = modal.selected_idx;
                    let tab = modal.tab;
                    match tab {
                        SchemaSubTab::Columns => {
                            self.column_visibility.columns.toggle(idx);
                        }
                        SchemaSubTab::Indexes => {
                            self.column_visibility.indexes.toggle(idx);
                        }
                        SchemaSubTab::ForeignKeys => {
                            self.column_visibility.foreign_keys.toggle(idx);
                        }
                        SchemaSubTab::Constraints => {
                            self.column_visibility.constraints.toggle(idx);
                        }
                        SchemaSubTab::Definition => {
                            // No visibility settings for Definition tab
                        }
                    }
                }
            }

            Message::CloseModal => {
                self.modal_state = ModalState::None;
            }

            Message::ModalConfirm => {
                match &self.modal_state {
                    ModalState::AddConnection(modal) => {
                        if let Some(conn) = self.create_connection_from_modal(modal) {
                            // Add connection to current project if in Connections mode
                            if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                                if let Some(project) = self.projects.get_mut(proj_idx) {
                                    project.connections.push(conn);
                                    self.status_message = "Connection added".to_string();
                                }
                            }
                            self.modal_state = ModalState::None;
                        } else {
                            self.status_message =
                                "Invalid: fill name, host, user, database and valid port (1-65535)"
                                    .to_string();
                            // Keep modal open for user to correct input
                        }
                    }
                    ModalState::AddProject(modal) => {
                        if modal.name.trim().is_empty() {
                            self.status_message = "Project name cannot be empty".to_string();
                        } else {
                            let new_project = Project::new(modal.name.trim());
                            self.projects.push(new_project);
                            self.selected_project_idx = self.projects.len() - 1;
                            self.status_message = "Project added".to_string();
                            self.modal_state = ModalState::None;
                        }
                    }
                    ModalState::EditProject(proj_idx, modal) => {
                        if modal.name.trim().is_empty() {
                            self.status_message = "Project name cannot be empty".to_string();
                        } else {
                            let proj_idx = *proj_idx;
                            let new_name = modal.name.trim().to_string();
                            if let Some(project) = self.projects.get_mut(proj_idx) {
                                project.name = new_name;
                                self.status_message = "Project updated".to_string();
                            }
                            self.modal_state = ModalState::None;
                        }
                    }
                    ModalState::DeleteProject(modal) => {
                        let proj_idx = modal.project_idx;
                        if proj_idx < self.projects.len() {
                            self.projects.remove(proj_idx);
                            // Adjust selection if needed
                            if self.selected_project_idx >= self.projects.len()
                                && !self.projects.is_empty()
                            {
                                self.selected_project_idx = self.projects.len() - 1;
                            }
                            self.status_message = "Project deleted".to_string();
                        }
                        self.modal_state = ModalState::None;
                    }
                    ModalState::SearchProject(_) => {
                        // SearchProject uses SearchConfirm instead of ModalConfirm
                    }
                    ModalState::SearchConnection(_) => {
                        // SearchConnection uses SearchConnectionConfirm instead of ModalConfirm
                    }
                    ModalState::SearchTable(_) => {
                        // SearchTable uses TableSearchConfirm instead of ModalConfirm
                    }
                    ModalState::UnifiedSearch(_) => {
                        // UnifiedSearch uses UnifiedSearchConfirm instead of ModalConfirm
                    }
                    ModalState::ColumnVisibility(_) => {
                        // ColumnVisibility uses ToggleColumnVisibility, just close on confirm
                        self.modal_state = ModalState::None;
                    }
                    ModalState::None | ModalState::History(_) => {}
                }
            }

            Message::ModalInputChar(c) => match &mut self.modal_state {
                ModalState::AddConnection(modal) => match modal.focused_field {
                    ConnectionModalField::Name => modal.name.push(c),
                    ConnectionModalField::Host => modal.host.push(c),
                    ConnectionModalField::Port => {
                        if c.is_ascii_digit() && modal.port.len() < 5 {
                            modal.port.push(c);
                        }
                    }
                    ConnectionModalField::User => modal.user.push(c),
                    ConnectionModalField::Password => modal.password.push(c),
                    ConnectionModalField::Database => modal.database.push(c),
                    ConnectionModalField::ButtonOk | ConnectionModalField::ButtonCancel => {}
                },
                ModalState::AddProject(modal) | ModalState::EditProject(_, modal) => {
                    if modal.focused_field == ProjectModalField::Name {
                        modal.name.push(c);
                    }
                }
                ModalState::SearchProject(modal) => {
                    modal.query.push(c);
                    modal.update_filter(&self.projects);
                }
                ModalState::SearchConnection(modal) => {
                    modal.query.push(c);
                    if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                        if let Some(project) = self.projects.get(proj_idx) {
                            modal.update_filter(&project.connections);
                        }
                    }
                }
                ModalState::SearchTable(modal) => {
                    modal.query.push(c);
                    if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                        if let Some(project) = self.projects.get(proj_idx) {
                            if let Some(conn) =
                                project.connections.get(self.selected_connection_idx)
                            {
                                modal.update_filter(&conn.tables);
                            }
                        }
                    }
                }
                ModalState::UnifiedSearch(modal) => {
                    modal.query.push(c);
                    if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                        if let Some(project) = self.projects.get(proj_idx) {
                            let tables = if let Some(conn) =
                                project.connections.get(self.selected_connection_idx)
                            {
                                if conn.expanded {
                                    &conn.tables[..]
                                } else {
                                    &[]
                                }
                            } else {
                                &[]
                            };
                            modal.update_filter(&project.connections, tables);
                        }
                    }
                }
                _ => {}
            },

            Message::ModalInputBackspace => match &mut self.modal_state {
                ModalState::AddConnection(modal) => match modal.focused_field {
                    ConnectionModalField::Name => {
                        modal.name.pop();
                    }
                    ConnectionModalField::Host => {
                        modal.host.pop();
                    }
                    ConnectionModalField::Port => {
                        modal.port.pop();
                    }
                    ConnectionModalField::User => {
                        modal.user.pop();
                    }
                    ConnectionModalField::Password => {
                        modal.password.pop();
                    }
                    ConnectionModalField::Database => {
                        modal.database.pop();
                    }
                    ConnectionModalField::ButtonOk | ConnectionModalField::ButtonCancel => {}
                },
                ModalState::AddProject(modal) | ModalState::EditProject(_, modal) => {
                    if modal.focused_field == ProjectModalField::Name {
                        modal.name.pop();
                    }
                }
                ModalState::SearchProject(modal) => {
                    modal.query.pop();
                    modal.update_filter(&self.projects);
                }
                ModalState::SearchConnection(modal) => {
                    modal.query.pop();
                    if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                        if let Some(project) = self.projects.get(proj_idx) {
                            modal.update_filter(&project.connections);
                        }
                    }
                }
                ModalState::SearchTable(modal) => {
                    modal.query.pop();
                    if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                        if let Some(project) = self.projects.get(proj_idx) {
                            if let Some(conn) =
                                project.connections.get(self.selected_connection_idx)
                            {
                                modal.update_filter(&conn.tables);
                            }
                        }
                    }
                }
                ModalState::UnifiedSearch(modal) => {
                    modal.query.pop();
                    if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                        if let Some(project) = self.projects.get(proj_idx) {
                            let tables = if let Some(conn) =
                                project.connections.get(self.selected_connection_idx)
                            {
                                if conn.expanded {
                                    &conn.tables[..]
                                } else {
                                    &[]
                                }
                            } else {
                                &[]
                            };
                            modal.update_filter(&project.connections, tables);
                        }
                    }
                }
                _ => {}
            },

            Message::ModalNextField => match &mut self.modal_state {
                ModalState::AddConnection(modal) => {
                    modal.focused_field = modal.focused_field.next();
                }
                ModalState::AddProject(modal) | ModalState::EditProject(_, modal) => {
                    modal.focused_field = modal.focused_field.next();
                }
                ModalState::DeleteProject(modal) => {
                    modal.focused_field = modal.focused_field.next();
                }
                ModalState::SearchProject(modal) => {
                    modal.navigate_down();
                }
                ModalState::SearchConnection(modal) => {
                    modal.navigate_down();
                }
                ModalState::SearchTable(modal) => {
                    modal.navigate_down();
                }
                ModalState::UnifiedSearch(modal) => {
                    modal.navigate_down();
                }
                ModalState::ColumnVisibility(modal) => {
                    modal.navigate_down();
                }
                ModalState::None | ModalState::History(_) => {}
            },

            Message::ModalPrevField => match &mut self.modal_state {
                ModalState::AddConnection(modal) => {
                    modal.focused_field = modal.focused_field.prev();
                }
                ModalState::AddProject(modal) | ModalState::EditProject(_, modal) => {
                    modal.focused_field = modal.focused_field.prev();
                }
                ModalState::DeleteProject(modal) => {
                    modal.focused_field = modal.focused_field.prev();
                }
                ModalState::SearchProject(modal) => {
                    modal.navigate_up();
                }
                ModalState::SearchConnection(modal) => {
                    modal.navigate_up();
                }
                ModalState::SearchTable(modal) => {
                    modal.navigate_up();
                }
                ModalState::UnifiedSearch(modal) => {
                    modal.navigate_up();
                }
                ModalState::ColumnVisibility(modal) => {
                    modal.navigate_up();
                }
                ModalState::None | ModalState::History(_) => {}
            },

            // Query history messages
            Message::OpenHistoryModal => {
                if !self.query_history.is_empty() {
                    self.modal_state = ModalState::History(HistoryModal::default());
                } else {
                    self.status_message = "No query history".to_string();
                }
            }

            Message::HistoryNavigateUp => {
                if let ModalState::History(modal) = &mut self.modal_state {
                    if modal.selected_idx > 0 {
                        modal.selected_idx -= 1;
                    } else if !self.query_history.is_empty() {
                        modal.selected_idx = self.query_history.len() - 1;
                    }
                }
            }

            Message::HistoryNavigateDown => {
                if let ModalState::History(modal) = &mut self.modal_state {
                    if modal.selected_idx + 1 < self.query_history.len() {
                        modal.selected_idx += 1;
                    } else {
                        modal.selected_idx = 0;
                    }
                }
            }

            Message::HistorySelectEntry => {
                if let ModalState::History(modal) = &self.modal_state {
                    if let Some(entry) = self.query_history.get(modal.selected_idx) {
                        self.query = entry.query.clone();
                        self.status_message =
                            format!("Loaded query from history ({})", entry.connection_name);
                    }
                    self.modal_state = ModalState::None;
                }
            }

            Message::ClearHistory => {
                self.query_history.clear();
                self.history_dirty = true;
                // Reset selected_idx before closing modal
                if let ModalState::History(modal) = &mut self.modal_state {
                    modal.selected_idx = 0;
                }
                self.modal_state = ModalState::None;
                self.status_message = "Query history cleared".to_string();
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

            // Data table navigation
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

    fn create_connection_from_modal(&self, modal: &AddConnectionModal) -> Option<Connection> {
        // Parse port as u16, which constrains the upper bound to 65535
        let port: u16 = modal.port.parse().ok()?;
        // Validate port range: reject port 0 (u16 already ensures <= 65535)
        if port == 0 {
            return None;
        }
        // Validate required fields: name, host, database, and user
        if modal.name.is_empty()
            || modal.host.is_empty()
            || modal.database.is_empty()
            || modal.user.is_empty()
        {
            return None;
        }
        Some(Connection {
            name: modal.name.clone(),
            host: modal.host.clone(),
            port,
            database: modal.database.clone(),
            username: modal.user.clone(),
            password: modal.password.clone(),
            expanded: false,
            tables: vec![],
        })
    }

    /// Navigate up based on current sidebar mode
    fn navigate_up(&mut self) {
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
    fn navigate_down(&mut self) {
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

    /// Fetch table details (columns, indexes, foreign keys, constraints) if not already loaded
    fn fetch_table_details_if_needed(&mut self, proj_idx: usize) {
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

    /// Activate current selection (Enter key)
    fn activate(&mut self) {
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
    fn go_back(&mut self) {
        if let SidebarMode::Connections(_) = self.sidebar_mode {
            self.sidebar_mode = SidebarMode::Projects;
            self.status_message = "Projects".to_string();
        }
    }

    fn toggle_connection_expand(&mut self, proj_idx: usize) {
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

    fn activate_table(&mut self, proj_idx: usize) {
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

    // ========================================================================
    // Async DB Worker Methods
    // ========================================================================

    /// Set the DB worker handle for async operations
    pub fn set_db_worker(&mut self, worker: DbWorkerHandle) {
        self.db_worker = Some(worker);
    }

    /// Get the next unique request ID
    fn next_request_id(&mut self) -> u64 {
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
    fn send_fetch_tables(&mut self, conn: &Connection, proj_idx: usize, conn_idx: usize) {
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
    fn send_fetch_table_details(
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
    fn send_execute_query(&mut self, conn: &Connection, query: &str, proj_idx: usize) {
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
}

#[cfg(test)]
mod tests {
    use super::*;
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
            // Initialize pagination to match the result for navigation tests
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

        // Should not go below 0
        app.navigate_data_table(-1);
        assert_eq!(app.data_table_state.selected(), Some(0));

        app.navigate_data_table(-10);
        assert_eq!(app.data_table_state.selected(), Some(0));
    }

    #[test]
    fn test_navigate_data_table_boundary_last_row() {
        let mut app = create_test_app_with_result(10);
        app.data_table_state.select(Some(9));

        // Should not exceed row_count - 1
        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(9));

        app.navigate_data_table(100);
        assert_eq!(app.data_table_state.selected(), Some(9));
    }

    #[test]
    fn test_navigate_data_table_empty_result() {
        let mut app = create_test_app_with_result(0);
        app.data_table_state.select(Some(0));

        // Should do nothing with empty result
        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(0));
    }

    #[test]
    fn test_navigate_data_table_no_result() {
        let mut app = App::new(vec![]);
        app.data_table_state.select(Some(0));

        // Should do nothing when result is None
        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(0));
    }

    #[test]
    fn test_navigate_data_table_no_selection() {
        let mut app = create_test_app_with_result(10);
        // No selection initially

        app.navigate_data_table(3);
        // Should start from 0 and move to 3
        assert_eq!(app.data_table_state.selected(), Some(3));
    }

    #[test]
    fn test_navigate_data_table_page_navigation() {
        // Test navigation within a single page (page size 50, 100 rows total)
        let mut app = create_test_app_with_result(100);
        app.pagination.next_page(); // Move to page 2 (rows 50-99)
        app.data_table_state.select(Some(50));

        // Page down (10 rows) within page 2
        app.navigate_data_table(10);
        assert_eq!(app.data_table_state.selected(), Some(60));

        // Page up (10 rows) within page 2
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

        // Should match "users" (index 0) and "user_sessions" (index 2)
        assert_eq!(modal.filtered_indices, vec![0, 2]);
    }

    #[test]
    fn test_search_table_modal_update_filter_case_insensitive() {
        let tables = create_test_tables();
        let mut modal = SearchTableModal::with_all_tables(tables.len());

        modal.query = "USER".to_string();
        modal.update_filter(&tables);

        // Should match "users" and "user_sessions" (case insensitive)
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

        modal.selected_idx = 3; // Select last item
        modal.query = "user".to_string();
        modal.update_filter(&tables);

        // After filtering to 2 items, selected_idx should be adjusted to 1
        assert_eq!(modal.selected_idx, 1);
    }

    #[test]
    fn test_search_table_modal_selected_table_idx_returns_correct_index() {
        let tables = create_test_tables();
        let mut modal = SearchTableModal::with_all_tables(tables.len());

        modal.query = "user".to_string();
        modal.update_filter(&tables);
        modal.selected_idx = 1; // Select second filtered item

        // filtered_indices is [0, 2], so selected_idx 1 -> table index 2
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

        modal.selected_idx = 2; // Last item
        modal.navigate_down();

        assert_eq!(modal.selected_idx, 0); // Wrapped to first
    }

    #[test]
    fn test_search_table_modal_navigate_up_wraps_around() {
        let mut modal = SearchTableModal::with_all_tables(3);

        modal.selected_idx = 0; // First item
        modal.navigate_up();

        assert_eq!(modal.selected_idx, 2); // Wrapped to last
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

        // Should match "postgres_local" (0) and "postgres_prod" (1)
        assert_eq!(modal.filtered_connection_indices, vec![0, 1]);
    }

    #[test]
    fn test_unified_search_modal_update_filter_tables() {
        let connections = create_test_connections();
        let tables = create_test_tables();
        let mut modal = UnifiedSearchModal::new(connections.len(), tables.len(), false);

        modal.query = "user".to_string();
        modal.update_filter(&connections, &tables);

        // Should match "users" (0) and "user_sessions" (2)
        assert_eq!(modal.filtered_table_indices, vec![0, 2]);
    }

    #[test]
    fn test_unified_search_modal_update_filter_both() {
        let connections = create_test_connections();
        let tables = create_test_tables();
        let mut modal = UnifiedSearchModal::new(connections.len(), tables.len(), false);

        modal.query = "prod".to_string();
        modal.update_filter(&connections, &tables);

        // "postgres_prod" matches in connections, "products" matches in tables
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

        modal.navigate_down(); // wrap
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

        modal.navigate_up(); // wrap
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

        // filtered_connection_indices is [0, 1], selected_idx 1 -> connection index 1
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

        // filtered_table_indices is [0, 2], selected_idx 1 -> table index 2
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
        assert_eq!(modal.table_count(), 0); // "postgres" doesn't match any table
    }

    #[test]
    fn test_unified_search_modal_empty_filter() {
        let connections = create_test_connections();
        let tables = create_test_tables();
        let mut modal = UnifiedSearchModal::new(connections.len(), tables.len(), false);

        modal.query = "".to_string();
        modal.update_filter(&connections, &tables);

        // Empty query shows all
        assert_eq!(modal.filtered_connection_indices.len(), 3);
        assert_eq!(modal.filtered_table_indices.len(), 4);
    }

    #[test]
    fn test_navigate_data_table_respects_page_boundary_down() {
        // Create app with 100 rows and page size 50
        let mut app = create_test_app_with_result(100);
        app.pagination = Pagination::new(100);
        app.pagination.page_size = 50;
        app.data_table_state.select(Some(49)); // Last row of page 1

        // Pressing down at the last row of the page should NOT move past page boundary
        app.navigate_data_table(1);

        // Should stay at 49 (last row of current page), not go to 50
        assert_eq!(app.data_table_state.selected(), Some(49));
    }

    #[test]
    fn test_navigate_data_table_respects_page_boundary_up() {
        // Create app with 100 rows and page size 50
        let mut app = create_test_app_with_result(100);
        app.pagination = Pagination::new(100);
        app.pagination.page_size = 50;
        app.pagination.next_page(); // Go to page 2 (rows 50-99)
        app.data_table_state.select(Some(50)); // First row of page 2

        // Pressing up at the first row of the page should NOT move past page boundary
        app.navigate_data_table(-1);

        // Should stay at 50 (first row of current page), not go to 49
        assert_eq!(app.data_table_state.selected(), Some(50));
    }

    #[test]
    fn test_navigate_data_table_within_page_works_normally() {
        // Create app with 100 rows and page size 50
        let mut app = create_test_app_with_result(100);
        app.pagination = Pagination::new(100);
        app.pagination.page_size = 50;
        app.data_table_state.select(Some(25)); // Middle of page 1

        // Normal navigation within page should work
        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(26));

        app.navigate_data_table(-1);
        assert_eq!(app.data_table_state.selected(), Some(25));

        // Jump navigation within page
        app.navigate_data_table(10);
        assert_eq!(app.data_table_state.selected(), Some(35));
    }

    #[test]
    fn test_navigate_data_table_last_page_partial_rows() {
        // Create app with 75 rows and page size 50
        // Page 2 has only 25 rows (indices 50-74)
        let mut app = create_test_app_with_result(75);
        app.pagination = Pagination::new(75);
        app.pagination.page_size = 50;
        app.pagination.next_page(); // Go to page 2
        app.data_table_state.select(Some(74)); // Last row of data

        // Trying to go down should stay at 74
        app.navigate_data_table(1);
        assert_eq!(app.data_table_state.selected(), Some(74));
    }
}
