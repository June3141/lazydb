use crate::message::Message;
use crate::model::{Column, Connection, Project, QueryResult, Table};

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
}

/// Sidebar display mode - switches between Projects list and Connections list
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarMode {
    Projects,
    Connections(usize), // project index
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModalField {
    Name,
    Host,
    Port,
    User,
    Password,
    Database,
    ButtonOk,
    ButtonCancel,
}

impl ModalField {
    pub fn next(self) -> Self {
        match self {
            ModalField::Name => ModalField::Host,
            ModalField::Host => ModalField::Port,
            ModalField::Port => ModalField::User,
            ModalField::User => ModalField::Password,
            ModalField::Password => ModalField::Database,
            ModalField::Database => ModalField::ButtonOk,
            ModalField::ButtonOk => ModalField::ButtonCancel,
            ModalField::ButtonCancel => ModalField::Name,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            ModalField::Name => ModalField::ButtonCancel,
            ModalField::Host => ModalField::Name,
            ModalField::Port => ModalField::Host,
            ModalField::User => ModalField::Port,
            ModalField::Password => ModalField::User,
            ModalField::Database => ModalField::Password,
            ModalField::ButtonOk => ModalField::Database,
            ModalField::ButtonCancel => ModalField::ButtonOk,
        }
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
    pub focused_field: ModalField,
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
            focused_field: ModalField::Name,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ModalState {
    None,
    AddConnection(AddConnectionModal),
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
    pub status_message: String,
    pub modal_state: ModalState,
}

impl App {
    pub fn new_with_sample_data() -> Self {
        let projects = vec![
            Project::new("Production").with_connections(vec![
                Connection {
                    name: "Main DB".to_string(),
                    host: "prod.example.com".to_string(),
                    port: 5432,
                    database: "app_production".to_string(),
                    expanded: true,
                    tables: vec![
                        Table {
                            name: "users".to_string(),
                            row_count: 15420,
                            size_bytes: 2_540_000,
                            columns: vec![
                                Column {
                                    name: "id".to_string(),
                                    data_type: "INTEGER".to_string(),
                                    is_primary_key: true,
                                    comment: Some("Primary key".to_string()),
                                },
                                Column {
                                    name: "name".to_string(),
                                    data_type: "VARCHAR(100)".to_string(),
                                    is_primary_key: false,
                                    comment: Some("User's full name".to_string()),
                                },
                                Column {
                                    name: "email".to_string(),
                                    data_type: "VARCHAR(255)".to_string(),
                                    is_primary_key: false,
                                    comment: Some("Email address (unique)".to_string()),
                                },
                                Column {
                                    name: "created_at".to_string(),
                                    data_type: "TIMESTAMP".to_string(),
                                    is_primary_key: false,
                                    comment: None,
                                },
                            ],
                        },
                        Table {
                            name: "orders".to_string(),
                            row_count: 89234,
                            size_bytes: 15_200_000,
                            columns: vec![
                                Column {
                                    name: "id".to_string(),
                                    data_type: "INTEGER".to_string(),
                                    is_primary_key: true,
                                    comment: Some("Order ID".to_string()),
                                },
                                Column {
                                    name: "user_id".to_string(),
                                    data_type: "INTEGER".to_string(),
                                    is_primary_key: false,
                                    comment: Some("FK to users.id".to_string()),
                                },
                                Column {
                                    name: "total".to_string(),
                                    data_type: "DECIMAL(10,2)".to_string(),
                                    is_primary_key: false,
                                    comment: Some("Order total amount".to_string()),
                                },
                                Column {
                                    name: "status".to_string(),
                                    data_type: "VARCHAR(20)".to_string(),
                                    is_primary_key: false,
                                    comment: Some("pending/completed/cancelled".to_string()),
                                },
                            ],
                        },
                        Table {
                            name: "customers".to_string(),
                            row_count: 8921,
                            size_bytes: 1_240_000,
                            columns: vec![
                                Column {
                                    name: "id".to_string(),
                                    data_type: "INTEGER".to_string(),
                                    is_primary_key: true,
                                    comment: Some("Customer ID".to_string()),
                                },
                                Column {
                                    name: "name".to_string(),
                                    data_type: "VARCHAR(100)".to_string(),
                                    is_primary_key: false,
                                    comment: Some("Customer name".to_string()),
                                },
                                Column {
                                    name: "email".to_string(),
                                    data_type: "VARCHAR(255)".to_string(),
                                    is_primary_key: false,
                                    comment: None,
                                },
                                Column {
                                    name: "active".to_string(),
                                    data_type: "BOOLEAN".to_string(),
                                    is_primary_key: false,
                                    comment: Some("Is customer active".to_string()),
                                },
                            ],
                        },
                    ],
                },
                Connection {
                    name: "Analytics DB".to_string(),
                    host: "prod-analytics.example.com".to_string(),
                    port: 5432,
                    database: "analytics".to_string(),
                    expanded: false,
                    tables: vec![Table {
                        name: "events".to_string(),
                        row_count: 1_000_000,
                        size_bytes: 50_000_000,
                        columns: vec![
                            Column {
                                name: "id".to_string(),
                                data_type: "BIGINT".to_string(),
                                is_primary_key: true,
                                comment: Some("Event ID".to_string()),
                            },
                            Column {
                                name: "event_type".to_string(),
                                data_type: "VARCHAR(50)".to_string(),
                                is_primary_key: false,
                                comment: Some("Type of event".to_string()),
                            },
                        ],
                    }],
                },
            ]),
            Project::new("Development").with_connections(vec![Connection {
                name: "Local DB".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                database: "app_development".to_string(),
                expanded: false,
                tables: vec![
                    Table {
                        name: "users".to_string(),
                        row_count: 150,
                        size_bytes: 24_000,
                        columns: vec![
                            Column {
                                name: "id".to_string(),
                                data_type: "INTEGER".to_string(),
                                is_primary_key: true,
                                comment: None,
                            },
                            Column {
                                name: "name".to_string(),
                                data_type: "VARCHAR(100)".to_string(),
                                is_primary_key: false,
                                comment: None,
                            },
                        ],
                    },
                    Table {
                        name: "orders".to_string(),
                        row_count: 500,
                        size_bytes: 85_000,
                        columns: vec![
                            Column {
                                name: "id".to_string(),
                                data_type: "INTEGER".to_string(),
                                is_primary_key: true,
                                comment: None,
                            },
                            Column {
                                name: "user_id".to_string(),
                                data_type: "INTEGER".to_string(),
                                is_primary_key: false,
                                comment: None,
                            },
                        ],
                    },
                ],
            }]),
            Project::new("Staging").with_connections(vec![Connection {
                name: "Stage DB".to_string(),
                host: "staging.example.com".to_string(),
                port: 5432,
                database: "app_staging".to_string(),
                expanded: false,
                tables: vec![Table {
                    name: "users".to_string(),
                    row_count: 1000,
                    size_bytes: 160_000,
                    columns: vec![
                        Column {
                            name: "id".to_string(),
                            data_type: "INTEGER".to_string(),
                            is_primary_key: true,
                            comment: None,
                        },
                        Column {
                            name: "name".to_string(),
                            data_type: "VARCHAR(100)".to_string(),
                            is_primary_key: false,
                            comment: None,
                        },
                    ],
                }],
            }]),
        ];

        let query = "SELECT * FROM customers LIMIT 50;".to_string();

        let result = Some(QueryResult {
            columns: vec![
                "id".to_string(),
                "name".to_string(),
                "email".to_string(),
                "created_at".to_string(),
            ],
            rows: vec![
                vec![
                    "1".to_string(),
                    "Alice Johnson".to_string(),
                    "alice@example.com".to_string(),
                    "2024-01-15".to_string(),
                ],
                vec![
                    "2".to_string(),
                    "Bob Smith".to_string(),
                    "bob@example.com".to_string(),
                    "2024-01-16".to_string(),
                ],
                vec![
                    "3".to_string(),
                    "Carol Williams".to_string(),
                    "carol@example.com".to_string(),
                    "2024-01-17".to_string(),
                ],
                vec![
                    "4".to_string(),
                    "David Brown".to_string(),
                    "david@example.com".to_string(),
                    "2024-01-18".to_string(),
                ],
            ],
            execution_time_ms: 23,
        });

        App {
            projects,
            sidebar_mode: SidebarMode::Projects,
            selected_project_idx: 0,
            selected_connection_idx: 0,
            selected_table_idx: None,
            query,
            result,
            focus: Focus::Sidebar,
            main_panel_tab: MainPanelTab::Data,
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

            Message::OpenAddConnectionModal => {
                self.modal_state = ModalState::AddConnection(AddConnectionModal::default());
            }

            Message::CloseModal => {
                self.modal_state = ModalState::None;
            }

            Message::ModalConfirm => {
                if let ModalState::AddConnection(ref modal) = self.modal_state {
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
            }

            Message::ModalInputChar(c) => {
                if let ModalState::AddConnection(ref mut modal) = self.modal_state {
                    match modal.focused_field {
                        ModalField::Name => modal.name.push(c),
                        ModalField::Host => modal.host.push(c),
                        ModalField::Port => {
                            if c.is_ascii_digit() && modal.port.len() < 5 {
                                modal.port.push(c);
                            }
                        }
                        ModalField::User => modal.user.push(c),
                        ModalField::Password => modal.password.push(c),
                        ModalField::Database => modal.database.push(c),
                        ModalField::ButtonOk | ModalField::ButtonCancel => {}
                    }
                }
            }

            Message::ModalInputBackspace => {
                if let ModalState::AddConnection(ref mut modal) = self.modal_state {
                    match modal.focused_field {
                        ModalField::Name => {
                            modal.name.pop();
                        }
                        ModalField::Host => {
                            modal.host.pop();
                        }
                        ModalField::Port => {
                            modal.port.pop();
                        }
                        ModalField::User => {
                            modal.user.pop();
                        }
                        ModalField::Password => {
                            modal.password.pop();
                        }
                        ModalField::Database => {
                            modal.database.pop();
                        }
                        ModalField::ButtonOk | ModalField::ButtonCancel => {}
                    }
                }
            }

            Message::ModalNextField => {
                if let ModalState::AddConnection(ref mut modal) = self.modal_state {
                    modal.focused_field = modal.focused_field.next();
                }
            }

            Message::ModalPrevField => {
                if let ModalState::AddConnection(ref mut modal) = self.modal_state {
                    modal.focused_field = modal.focused_field.prev();
                }
            }
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
                }
            }
        }
    }

    fn activate_table(&mut self, proj_idx: usize) {
        if let Some(project) = self.projects.get(proj_idx) {
            if let Some(conn) = project.connections.get(self.selected_connection_idx) {
                if let Some(table_idx) = self.selected_table_idx {
                    if let Some(table) = conn.tables.get(table_idx) {
                        self.query = format!("SELECT * FROM {} LIMIT 50;", table.name);
                        self.status_message = format!(
                            "Selected: {}.{} ({} rows)",
                            conn.database, table.name, table.row_count
                        );
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
}
