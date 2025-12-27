use crate::db::{DatabaseProvider, PostgresProvider};
use crate::message::Message;
use crate::model::{
    Connection, HistoryEntry, Pagination, Project, QueryHistory, QueryResult, Table,
};
use ratatui::widgets::TableState;

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Sidebar,
    QueryEditor,
    MainPanel,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainPanelTab {
    Schema,
    Data,
    Relations,
}

/// Sub-tabs for the Schema tab
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SchemaSubTab {
    #[default]
    Columns,
    Indexes,
    ForeignKeys,
    Constraints,
}

/// Sidebar display mode - switches between Projects list and Connections list
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarMode {
    Projects,
    Connections(usize), // project index
}

/// Field identifiers for Connection modal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionModalField {
    Name,
    Host,
    Port,
    User,
    Password,
    Database,
    ButtonOk,
    ButtonCancel,
}

impl ConnectionModalField {
    pub fn next(self) -> Self {
        match self {
            ConnectionModalField::Name => ConnectionModalField::Host,
            ConnectionModalField::Host => ConnectionModalField::Port,
            ConnectionModalField::Port => ConnectionModalField::User,
            ConnectionModalField::User => ConnectionModalField::Password,
            ConnectionModalField::Password => ConnectionModalField::Database,
            ConnectionModalField::Database => ConnectionModalField::ButtonOk,
            ConnectionModalField::ButtonOk => ConnectionModalField::ButtonCancel,
            ConnectionModalField::ButtonCancel => ConnectionModalField::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ConnectionModalField::Name => ConnectionModalField::ButtonCancel,
            ConnectionModalField::Host => ConnectionModalField::Name,
            ConnectionModalField::Port => ConnectionModalField::Host,
            ConnectionModalField::User => ConnectionModalField::Port,
            ConnectionModalField::Password => ConnectionModalField::User,
            ConnectionModalField::Database => ConnectionModalField::Password,
            ConnectionModalField::ButtonOk => ConnectionModalField::Database,
            ConnectionModalField::ButtonCancel => ConnectionModalField::ButtonOk,
        }
    }
}

/// Field identifiers for Project modal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectModalField {
    Name,
    ButtonOk,
    ButtonCancel,
}

impl ProjectModalField {
    pub fn next(self) -> Self {
        match self {
            ProjectModalField::Name => ProjectModalField::ButtonOk,
            ProjectModalField::ButtonOk => ProjectModalField::ButtonCancel,
            ProjectModalField::ButtonCancel => ProjectModalField::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ProjectModalField::Name => ProjectModalField::ButtonCancel,
            ProjectModalField::ButtonOk => ProjectModalField::Name,
            ProjectModalField::ButtonCancel => ProjectModalField::ButtonOk,
        }
    }
}

/// Field identifiers for delete confirmation modal
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfirmModalField {
    ButtonOk,
    ButtonCancel,
}

impl ConfirmModalField {
    pub fn next(self) -> Self {
        match self {
            ConfirmModalField::ButtonOk => ConfirmModalField::ButtonCancel,
            ConfirmModalField::ButtonCancel => ConfirmModalField::ButtonOk,
        }
    }

    pub fn prev(self) -> Self {
        self.next()
    }
}

#[derive(Debug, Clone)]
pub struct AddConnectionModal {
    pub name: String,
    pub host: String,
    pub port: String,
    pub user: String,
    pub password: String,
    pub database: String,
    pub focused_field: ConnectionModalField,
}

impl Default for AddConnectionModal {
    fn default() -> Self {
        Self {
            name: String::new(),
            host: "localhost".to_string(),
            port: "5432".to_string(),
            user: String::new(),
            password: String::new(),
            database: String::new(),
            focused_field: ConnectionModalField::Name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProjectModal {
    pub name: String,
    pub focused_field: ProjectModalField,
}

impl Default for ProjectModal {
    fn default() -> Self {
        Self {
            name: String::new(),
            focused_field: ProjectModalField::Name,
        }
    }
}

impl ProjectModal {
    pub fn with_name(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            focused_field: ProjectModalField::Name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteProjectModal {
    pub project_idx: usize,
    pub project_name: String,
    pub focused_field: ConfirmModalField,
}

/// Search modal for filtering projects
#[derive(Debug, Clone, Default)]
pub struct SearchProjectModal {
    pub query: String,
    pub filtered_indices: Vec<usize>,
    pub selected_idx: usize,
}

impl SearchProjectModal {
    pub fn with_all_projects(project_count: usize) -> Self {
        Self {
            query: String::new(),
            filtered_indices: (0..project_count).collect(),
            selected_idx: 0,
        }
    }

    pub fn update_filter(&mut self, projects: &[Project]) {
        let query_lower = self.query.to_lowercase();
        self.filtered_indices = projects
            .iter()
            .enumerate()
            .filter(|(_, p)| self.query.is_empty() || p.name.to_lowercase().contains(&query_lower))
            .map(|(idx, _)| idx)
            .collect();

        // Adjust selected index if needed
        if self.selected_idx >= self.filtered_indices.len() {
            self.selected_idx = self.filtered_indices.len().saturating_sub(1);
        }
    }

    pub fn selected_project_idx(&self) -> Option<usize> {
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

/// Query history modal state
#[derive(Debug, Clone, Default)]
pub struct HistoryModal {
    /// Currently selected index in the history list
    pub selected_idx: usize,
}

/// Search modal for filtering connections within a project
#[derive(Debug, Clone, Default)]
pub struct SearchConnectionModal {
    pub query: String,
    pub filtered_indices: Vec<usize>,
    pub selected_idx: usize,
}

/// Search modal for filtering tables within a connection
#[derive(Debug, Clone, Default)]
pub struct SearchTableModal {
    pub query: String,
    pub filtered_indices: Vec<usize>,
    pub selected_idx: usize,
}

/// Column visibility settings for Columns tab
#[derive(Debug, Clone)]
pub struct ColumnsVisibility {
    pub show_icon: bool,
    pub show_name: bool,
    pub show_type: bool,
    pub show_nullable: bool,
    pub show_default: bool,
    pub show_key: bool,
}

impl Default for ColumnsVisibility {
    fn default() -> Self {
        Self {
            show_icon: true,
            show_name: true,
            show_type: true,
            show_nullable: true,
            show_default: true,
            show_key: true,
        }
    }
}

impl ColumnsVisibility {
    pub fn all_columns() -> &'static [&'static str] {
        &["Icon", "Name", "Type", "Null", "Default", "Key"]
    }

    pub fn is_visible(&self, index: usize) -> bool {
        match index {
            0 => self.show_icon,
            1 => self.show_name,
            2 => self.show_type,
            3 => self.show_nullable,
            4 => self.show_default,
            5 => self.show_key,
            _ => false,
        }
    }

    pub fn toggle(&mut self, index: usize) {
        match index {
            0 => self.show_icon = !self.show_icon,
            1 => self.show_name = !self.show_name,
            2 => self.show_type = !self.show_type,
            3 => self.show_nullable = !self.show_nullable,
            4 => self.show_default = !self.show_default,
            5 => self.show_key = !self.show_key,
            _ => {}
        }
    }
}

/// Column visibility settings for Indexes tab
#[derive(Debug, Clone)]
pub struct IndexesVisibility {
    pub show_name: bool,
    pub show_type: bool,
    pub show_method: bool,
    pub show_columns: bool,
}

impl Default for IndexesVisibility {
    fn default() -> Self {
        Self {
            show_name: true,
            show_type: true,
            show_method: true,
            show_columns: true,
        }
    }
}

impl IndexesVisibility {
    pub fn all_columns() -> &'static [&'static str] {
        &["Name", "Type", "Method", "Columns"]
    }

    pub fn is_visible(&self, index: usize) -> bool {
        match index {
            0 => self.show_name,
            1 => self.show_type,
            2 => self.show_method,
            3 => self.show_columns,
            _ => false,
        }
    }

    pub fn toggle(&mut self, index: usize) {
        match index {
            0 => self.show_name = !self.show_name,
            1 => self.show_type = !self.show_type,
            2 => self.show_method = !self.show_method,
            3 => self.show_columns = !self.show_columns,
            _ => {}
        }
    }
}

/// Column visibility settings for Foreign Keys tab
#[derive(Debug, Clone)]
pub struct ForeignKeysVisibility {
    pub show_name: bool,
    pub show_column: bool,
    pub show_references: bool,
    pub show_on_delete: bool,
    pub show_on_update: bool,
}

impl Default for ForeignKeysVisibility {
    fn default() -> Self {
        Self {
            show_name: true,
            show_column: true,
            show_references: true,
            show_on_delete: true,
            show_on_update: true,
        }
    }
}

impl ForeignKeysVisibility {
    pub fn all_columns() -> &'static [&'static str] {
        &["Name", "Column", "References", "ON DELETE", "ON UPDATE"]
    }

    pub fn is_visible(&self, index: usize) -> bool {
        match index {
            0 => self.show_name,
            1 => self.show_column,
            2 => self.show_references,
            3 => self.show_on_delete,
            4 => self.show_on_update,
            _ => false,
        }
    }

    pub fn toggle(&mut self, index: usize) {
        match index {
            0 => self.show_name = !self.show_name,
            1 => self.show_column = !self.show_column,
            2 => self.show_references = !self.show_references,
            3 => self.show_on_delete = !self.show_on_delete,
            4 => self.show_on_update = !self.show_on_update,
            _ => {}
        }
    }
}

/// Column visibility settings for Constraints tab
#[derive(Debug, Clone)]
pub struct ConstraintsVisibility {
    pub show_name: bool,
    pub show_type: bool,
    pub show_columns: bool,
    pub show_definition: bool,
}

impl Default for ConstraintsVisibility {
    fn default() -> Self {
        Self {
            show_name: true,
            show_type: true,
            show_columns: true,
            show_definition: true,
        }
    }
}

impl ConstraintsVisibility {
    pub fn all_columns() -> &'static [&'static str] {
        &["Name", "Type", "Columns", "Definition"]
    }

    pub fn is_visible(&self, index: usize) -> bool {
        match index {
            0 => self.show_name,
            1 => self.show_type,
            2 => self.show_columns,
            3 => self.show_definition,
            _ => false,
        }
    }

    pub fn toggle(&mut self, index: usize) {
        match index {
            0 => self.show_name = !self.show_name,
            1 => self.show_type = !self.show_type,
            2 => self.show_columns = !self.show_columns,
            3 => self.show_definition = !self.show_definition,
            _ => {}
        }
    }
}

/// Column visibility settings for all schema sub-tabs
#[derive(Debug, Clone, Default)]
pub struct ColumnVisibilitySettings {
    pub columns: ColumnsVisibility,
    pub indexes: IndexesVisibility,
    pub foreign_keys: ForeignKeysVisibility,
    pub constraints: ConstraintsVisibility,
}

/// Modal for configuring column visibility
#[derive(Debug, Clone)]
pub struct ColumnVisibilityModal {
    pub tab: SchemaSubTab,
    pub selected_idx: usize,
}

impl ColumnVisibilityModal {
    pub fn new(tab: SchemaSubTab) -> Self {
        Self {
            tab,
            selected_idx: 0,
        }
    }

    pub fn column_count(&self) -> usize {
        match self.tab {
            SchemaSubTab::Columns => ColumnsVisibility::all_columns().len(),
            SchemaSubTab::Indexes => IndexesVisibility::all_columns().len(),
            SchemaSubTab::ForeignKeys => ForeignKeysVisibility::all_columns().len(),
            SchemaSubTab::Constraints => ConstraintsVisibility::all_columns().len(),
        }
    }

    pub fn navigate_up(&mut self) {
        let count = self.column_count();
        if count > 0 {
            if self.selected_idx > 0 {
                self.selected_idx -= 1;
            } else {
                self.selected_idx = count - 1;
            }
        }
    }

    pub fn navigate_down(&mut self) {
        let count = self.column_count();
        if count > 0 {
            if self.selected_idx + 1 < count {
                self.selected_idx += 1;
            } else {
                self.selected_idx = 0;
            }
        }
    }
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

#[derive(Debug, Clone)]
pub enum ModalState {
    None,
    AddConnection(AddConnectionModal),
    AddProject(ProjectModal),
    EditProject(usize, ProjectModal), // (project index, modal)
    DeleteProject(DeleteProjectModal),
    SearchProject(SearchProjectModal),
    SearchConnection(SearchConnectionModal),
    SearchTable(SearchTableModal),
    History(HistoryModal),
    ColumnVisibility(ColumnVisibilityModal),
}

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
    fn navigate_data_table(&mut self, delta: i32) {
        if let Some(result) = &self.result {
            if result.rows.is_empty() {
                return;
            }
            let row_count = result.rows.len();
            let current = self.data_table_state.selected().unwrap_or(0);
            let new_idx = if delta < 0 {
                current.saturating_sub((-delta) as usize)
            } else {
                (current + delta as usize).min(row_count - 1)
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

        // Skip if details are already loaded
        if table.details_loaded {
            return;
        }

        let table_name = table.name.clone();
        let schema = table.schema.clone();

        // Fetch table details using helper
        match Self::create_provider(conn) {
            Ok(provider) => {
                match provider.get_table_details(&table_name, schema.as_deref()) {
                    Ok(detailed_table) => {
                        // Update the table with full details using captured indices
                        if let Some(project) = self.projects.get_mut(proj_idx) {
                            if let Some(conn) = project.connections.get_mut(conn_idx) {
                                if let Some(table) = conn.tables.get_mut(table_idx) {
                                    table.columns = detailed_table.columns;
                                    table.indexes = detailed_table.indexes;
                                    table.foreign_keys = detailed_table.foreign_keys;
                                    table.constraints = detailed_table.constraints;
                                    table.details_loaded = true;
                                }
                            }
                        }
                        self.status_message = format!("Loaded schema for {}", table_name);
                    }
                    Err(e) => {
                        self.status_message = format!("Failed to get table details: {}", e);
                    }
                }
            }
            Err(e) => {
                self.status_message = format!("Connection failed: {}", e);
            }
        }
    }

    /// Create a PostgresProvider from a Connection
    fn create_provider(conn: &Connection) -> Result<PostgresProvider, crate::db::ProviderError> {
        PostgresProvider::connect(
            &conn.host,
            conn.port,
            &conn.database,
            &conn.username,
            &conn.password,
        )
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
        if let Some(project) = self.projects.get_mut(proj_idx) {
            if let Some(conn) = project.connections.get_mut(self.selected_connection_idx) {
                conn.expanded = !conn.expanded;
                if !conn.expanded {
                    self.selected_table_idx = None;
                } else if conn.tables.is_empty() {
                    // Fetch tables when expanding for the first time
                    match Self::create_provider(conn) {
                        Ok(provider) => match provider.get_tables(Some("public")) {
                            Ok(tables) => {
                                conn.tables = tables;
                                self.status_message =
                                    format!("Loaded {} tables", conn.tables.len());
                            }
                            Err(e) => {
                                self.status_message = format!("Failed to get tables: {}", e);
                            }
                        },
                        Err(e) => {
                            self.status_message = format!("Connection failed: {}", e);
                            conn.expanded = false;
                        }
                    }
                }
            }
        }
    }

    fn activate_table(&mut self, proj_idx: usize) {
        if let Some(project) = self.projects.get(proj_idx) {
            if let Some(conn) = project.connections.get(self.selected_connection_idx) {
                if let Some(table_idx) = self.selected_table_idx {
                    if let Some(table) = conn.tables.get(table_idx) {
                        let query = format!("SELECT * FROM {}", table.name);
                        self.query = format!("{};", query);

                        let conn_name = conn.name.clone();
                        let database = conn.database.clone();

                        // Execute query to fetch data
                        match Self::create_provider(conn) {
                            Ok(provider) => match provider.execute_query(&query) {
                                Ok(result) => {
                                    let row_count = result.rows.len();
                                    let execution_time_ms = result.execution_time_ms;

                                    // Add to history
                                    self.query_history.add(HistoryEntry::success(
                                        &query,
                                        &conn_name,
                                        &database,
                                        execution_time_ms,
                                        row_count,
                                    ));
                                    self.history_dirty = true;

                                    // Reset pagination with new total rows
                                    self.pagination = Pagination::new(row_count);
                                    self.result = Some(result);
                                    self.status_message = format!(
                                        "Fetched {} rows from {}.{}",
                                        row_count, database, table.name
                                    );
                                }
                                Err(e) => {
                                    // Add error to history
                                    self.query_history.add(HistoryEntry::error(
                                        &query,
                                        &conn_name,
                                        &database,
                                        e.to_string(),
                                    ));
                                    self.history_dirty = true;

                                    self.result = None;
                                    self.pagination = Pagination::default();
                                    self.status_message = format!("Query failed: {}", e);
                                }
                            },
                            Err(e) => {
                                self.result = None;
                                self.pagination = Pagination::default();
                                self.status_message = format!("Connection failed: {}", e);
                            }
                        }
                    }
                }
            }
        }
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
        let mut app = create_test_app_with_result(100);
        app.data_table_state.select(Some(50));

        // Page down (10 rows)
        app.navigate_data_table(10);
        assert_eq!(app.data_table_state.selected(), Some(60));

        // Page up (10 rows)
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
}
