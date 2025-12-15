use crate::db::{DatabaseProvider, PostgresProvider};
use crate::message::Message;
use crate::model::{Connection, Project, QueryResult, Table};

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

#[derive(Debug, Clone)]
pub enum ModalState {
    None,
    AddConnection(AddConnectionModal),
    AddProject(ProjectModal),
    EditProject(usize, ProjectModal), // (project index, modal)
    DeleteProject(DeleteProjectModal),
}

pub struct App {
    pub projects: Vec<Project>,
    pub sidebar_mode: SidebarMode,
    pub selected_project_idx: usize,
    pub selected_connection_idx: usize,
    pub selected_table_idx: Option<usize>,
    pub query: String,
    pub result: Option<QueryResult>,
    pub focus: Focus,
    pub main_panel_tab: MainPanelTab,
    pub schema_sub_tab: SchemaSubTab,
    pub status_message: String,
    pub modal_state: ModalState,
}

impl App {
    /// Create a new App with the given projects
    pub fn new(projects: Vec<Project>) -> Self {
        App {
            projects,
            sidebar_mode: SidebarMode::Projects,
            selected_project_idx: 0,
            selected_connection_idx: 0,
            selected_table_idx: None,
            query: String::new(),
            result: None,
            focus: Focus::Sidebar,
            main_panel_tab: MainPanelTab::Schema,
            schema_sub_tab: SchemaSubTab::default(),
            status_message: "Ready".to_string(),
            modal_state: ModalState::None,
        }
    }

    /// Check if a modal is currently open
    pub fn is_modal_open(&self) -> bool {
        !matches!(self.modal_state, ModalState::None)
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
                                "Invalid: fill name, host, database and valid port".to_string();
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
                    ModalState::None => {}
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
                ModalState::None => {}
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
                ModalState::None => {}
            },
        }

        false
    }

    fn create_connection_from_modal(&self, modal: &AddConnectionModal) -> Option<Connection> {
        let port: u16 = modal.port.parse().ok()?;
        // Validate port range (1-65535)
        if port == 0 {
            return None;
        }
        if modal.name.is_empty() || modal.host.is_empty() || modal.database.is_empty() {
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
                    match PostgresProvider::connect(
                        &conn.host,
                        conn.port,
                        &conn.database,
                        &conn.username,
                        &conn.password,
                    ) {
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
                        let query = format!("SELECT * FROM {} LIMIT 50", table.name);
                        self.query = format!("{};", query);

                        // Execute query to fetch data
                        match PostgresProvider::connect(
                            &conn.host,
                            conn.port,
                            &conn.database,
                            &conn.username,
                            &conn.password,
                        ) {
                            Ok(provider) => match provider.execute_query(&query) {
                                Ok(result) => {
                                    let row_count = result.rows.len();
                                    self.result = Some(result);
                                    self.status_message = format!(
                                        "Fetched {} rows from {}.{}",
                                        row_count, conn.database, table.name
                                    );
                                }
                                Err(e) => {
                                    self.result = None;
                                    self.status_message = format!("Query failed: {}", e);
                                }
                            },
                            Err(e) => {
                                self.result = None;
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
