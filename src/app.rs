use crate::message::Message;
use crate::model::{
    Column, Connection, Constraint, ConstraintType, ForeignKey, ForeignKeyAction, Index,
    IndexColumn, IndexType, Project, QueryResult, Table,
};

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
    pub schema_sub_tab: SchemaSubTab,
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
                        Self::create_users_table(15420, 2_540_000),
                        Self::create_orders_table(89234, 15_200_000),
                        Self::create_order_items_table(245_000, 8_500_000),
                        Self::create_products_table(1250, 520_000),
                    ],
                },
                Connection {
                    name: "Analytics DB".to_string(),
                    host: "prod-analytics.example.com".to_string(),
                    port: 5432,
                    database: "analytics".to_string(),
                    expanded: false,
                    tables: vec![Self::create_events_table()],
                },
            ]),
            Project::new("Development").with_connections(vec![Connection {
                name: "Local DB".to_string(),
                host: "localhost".to_string(),
                port: 5432,
                database: "app_development".to_string(),
                expanded: false,
                tables: vec![
                    Self::create_users_table(150, 24_000),
                    Self::create_orders_table(500, 85_000),
                ],
            }]),
            Project::new("Staging").with_connections(vec![Connection {
                name: "Stage DB".to_string(),
                host: "staging.example.com".to_string(),
                port: 5432,
                database: "app_staging".to_string(),
                expanded: false,
                tables: vec![Self::create_users_table(1000, 160_000)],
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

    // Sample data helper functions

    fn create_users_table(row_count: usize, size_bytes: u64) -> Table {
        Table::new("users")
            .with_schema("public")
            .with_columns(vec![
                Column::new("id", "SERIAL")
                    .primary_key()
                    .auto_increment()
                    .position(1),
                Column::new("email", "VARCHAR(255)")
                    .not_null()
                    .unique()
                    .position(2),
                Column::new("name", "VARCHAR(100)").not_null().position(3),
                Column::new("password_hash", "VARCHAR(255)")
                    .not_null()
                    .position(4),
                Column::new("created_at", "TIMESTAMP")
                    .not_null()
                    .default("NOW()")
                    .position(5),
                Column::new("updated_at", "TIMESTAMP").position(6),
                Column::new("is_active", "BOOLEAN")
                    .not_null()
                    .default("true")
                    .position(7),
            ])
            .with_indexes(vec![
                Index::new("users_pkey", IndexType::Primary)
                    .with_columns(vec![IndexColumn::new("id")]),
                Index::new("users_email_key", IndexType::Unique)
                    .with_columns(vec![IndexColumn::new("email")]),
                Index::new("idx_users_created_at", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("created_at").desc()]),
            ])
            .with_constraints(vec![
                Constraint::new("users_pkey", ConstraintType::PrimaryKey)
                    .with_columns(vec!["id".to_string()]),
                Constraint::new("users_email_key", ConstraintType::Unique)
                    .with_columns(vec!["email".to_string()]),
                Constraint::new("users_email_check", ConstraintType::Check)
                    .with_definition("email ~* '^[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\\.[A-Z]{2,}$'"),
            ])
            .with_stats(row_count, size_bytes)
    }

    fn create_orders_table(row_count: usize, size_bytes: u64) -> Table {
        Table::new("orders")
            .with_schema("public")
            .with_columns(vec![
                Column::new("id", "SERIAL")
                    .primary_key()
                    .auto_increment()
                    .position(1),
                Column::new("user_id", "INTEGER").not_null().position(2),
                Column::new("status", "VARCHAR(20)")
                    .not_null()
                    .default("'pending'")
                    .position(3),
                Column::new("total_amount", "DECIMAL(10,2)")
                    .not_null()
                    .position(4),
                Column::new("created_at", "TIMESTAMP")
                    .not_null()
                    .default("NOW()")
                    .position(5),
                Column::new("shipped_at", "TIMESTAMP").position(6),
            ])
            .with_indexes(vec![
                Index::new("orders_pkey", IndexType::Primary)
                    .with_columns(vec![IndexColumn::new("id")]),
                Index::new("idx_orders_user_id", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("user_id")]),
                Index::new("idx_orders_status", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("status")]),
                Index::new("idx_orders_created_at", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("created_at").desc()]),
            ])
            .with_foreign_keys(vec![ForeignKey::new(
                "fk_orders_user",
                vec!["user_id".to_string()],
                "users",
                vec!["id".to_string()],
            )
            .on_delete(ForeignKeyAction::Cascade)
            .on_update(ForeignKeyAction::Cascade)])
            .with_constraints(vec![
                Constraint::new("orders_pkey", ConstraintType::PrimaryKey)
                    .with_columns(vec!["id".to_string()]),
                Constraint::new("orders_status_check", ConstraintType::Check).with_definition(
                    "status IN ('pending', 'processing', 'shipped', 'delivered', 'cancelled')",
                ),
            ])
            .with_stats(row_count, size_bytes)
    }

    fn create_order_items_table(row_count: usize, size_bytes: u64) -> Table {
        Table::new("order_items")
            .with_schema("public")
            .with_columns(vec![
                Column::new("id", "SERIAL")
                    .primary_key()
                    .auto_increment()
                    .position(1),
                Column::new("order_id", "INTEGER").not_null().position(2),
                Column::new("product_id", "INTEGER").not_null().position(3),
                Column::new("quantity", "INTEGER")
                    .not_null()
                    .default("1")
                    .position(4),
                Column::new("unit_price", "DECIMAL(10,2)")
                    .not_null()
                    .position(5),
            ])
            .with_indexes(vec![
                Index::new("order_items_pkey", IndexType::Primary)
                    .with_columns(vec![IndexColumn::new("id")]),
                Index::new("idx_order_items_order_id", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("order_id")]),
                Index::new("idx_order_items_product_id", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("product_id")]),
            ])
            .with_foreign_keys(vec![
                ForeignKey::new(
                    "fk_order_items_order",
                    vec!["order_id".to_string()],
                    "orders",
                    vec!["id".to_string()],
                )
                .on_delete(ForeignKeyAction::Cascade),
                ForeignKey::new(
                    "fk_order_items_product",
                    vec!["product_id".to_string()],
                    "products",
                    vec!["id".to_string()],
                )
                .on_delete(ForeignKeyAction::Restrict),
            ])
            .with_constraints(vec![
                Constraint::new("order_items_pkey", ConstraintType::PrimaryKey)
                    .with_columns(vec!["id".to_string()]),
                Constraint::new("order_items_quantity_check", ConstraintType::Check)
                    .with_definition("quantity > 0"),
            ])
            .with_stats(row_count, size_bytes)
    }

    fn create_products_table(row_count: usize, size_bytes: u64) -> Table {
        Table::new("products")
            .with_schema("public")
            .with_columns(vec![
                Column::new("id", "SERIAL")
                    .primary_key()
                    .auto_increment()
                    .position(1),
                Column::new("name", "VARCHAR(200)").not_null().position(2),
                Column::new("description", "TEXT").position(3),
                Column::new("price", "DECIMAL(10,2)").not_null().position(4),
                Column::new("stock_quantity", "INTEGER")
                    .not_null()
                    .default("0")
                    .position(5),
                Column::new("category", "VARCHAR(50)").position(6),
                Column::new("created_at", "TIMESTAMP")
                    .not_null()
                    .default("NOW()")
                    .position(7),
            ])
            .with_indexes(vec![
                Index::new("products_pkey", IndexType::Primary)
                    .with_columns(vec![IndexColumn::new("id")]),
                Index::new("idx_products_category", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("category")]),
                Index::new("idx_products_name", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("name")]),
            ])
            .with_constraints(vec![
                Constraint::new("products_pkey", ConstraintType::PrimaryKey)
                    .with_columns(vec!["id".to_string()]),
                Constraint::new("products_price_check", ConstraintType::Check)
                    .with_definition("price >= 0"),
                Constraint::new("products_stock_check", ConstraintType::Check)
                    .with_definition("stock_quantity >= 0"),
            ])
            .with_stats(row_count, size_bytes)
    }

    fn create_events_table() -> Table {
        Table::new("events")
            .with_schema("analytics")
            .with_columns(vec![
                Column::new("id", "BIGSERIAL")
                    .primary_key()
                    .auto_increment()
                    .position(1),
                Column::new("event_type", "VARCHAR(50)")
                    .not_null()
                    .position(2),
                Column::new("user_id", "INTEGER").position(3),
                Column::new("payload", "JSONB").position(4),
                Column::new("created_at", "TIMESTAMP")
                    .not_null()
                    .default("NOW()")
                    .position(5),
            ])
            .with_indexes(vec![
                Index::new("events_pkey", IndexType::Primary)
                    .with_columns(vec![IndexColumn::new("id")]),
                Index::new("idx_events_type", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("event_type")]),
                Index::new("idx_events_user_id", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("user_id")]),
                Index::new("idx_events_created_at", IndexType::Index)
                    .with_columns(vec![IndexColumn::new("created_at").desc()]),
            ])
            .with_stats(1_000_000, 50_000_000)
    }
}
