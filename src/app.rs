use crate::message::Message;
use crate::model::{Column, Connection, QueryResult, Table};

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

pub struct App {
    pub connections: Vec<Connection>,
    pub selected_connection: usize,
    pub selected_table: Option<usize>,
    pub query: String,
    pub result: Option<QueryResult>,
    pub focus: Focus,
    pub main_panel_tab: MainPanelTab,
    pub status_message: String,
}

impl App {
    pub fn new_with_sample_data() -> Self {
        let connections = vec![
            Connection {
                name: "Production DB".to_string(),
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
                            Column { name: "id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: true },
                            Column { name: "name".to_string(), data_type: "VARCHAR(100)".to_string(), is_primary_key: false },
                            Column { name: "email".to_string(), data_type: "VARCHAR(255)".to_string(), is_primary_key: false },
                            Column { name: "created_at".to_string(), data_type: "TIMESTAMP".to_string(), is_primary_key: false },
                        ],
                    },
                    Table {
                        name: "orders".to_string(),
                        row_count: 89234,
                        size_bytes: 15_200_000,
                        columns: vec![
                            Column { name: "id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: true },
                            Column { name: "user_id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: false },
                            Column { name: "total".to_string(), data_type: "DECIMAL(10,2)".to_string(), is_primary_key: false },
                            Column { name: "status".to_string(), data_type: "VARCHAR(20)".to_string(), is_primary_key: false },
                            Column { name: "created_at".to_string(), data_type: "TIMESTAMP".to_string(), is_primary_key: false },
                        ],
                    },
                    Table {
                        name: "products".to_string(),
                        row_count: 1523,
                        size_bytes: 890_000,
                        columns: vec![
                            Column { name: "id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: true },
                            Column { name: "name".to_string(), data_type: "VARCHAR(200)".to_string(), is_primary_key: false },
                            Column { name: "price".to_string(), data_type: "DECIMAL(10,2)".to_string(), is_primary_key: false },
                        ],
                    },
                    Table {
                        name: "customers".to_string(),
                        row_count: 8921,
                        size_bytes: 1_240_000,
                        columns: vec![
                            Column { name: "id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: true },
                            Column { name: "name".to_string(), data_type: "VARCHAR(100)".to_string(), is_primary_key: false },
                            Column { name: "email".to_string(), data_type: "VARCHAR(255)".to_string(), is_primary_key: false },
                            Column { name: "created_at".to_string(), data_type: "DATE".to_string(), is_primary_key: false },
                            Column { name: "active".to_string(), data_type: "BOOLEAN".to_string(), is_primary_key: false },
                        ],
                    },
                    Table {
                        name: "invoices".to_string(),
                        row_count: 45123,
                        size_bytes: 8_900_000,
                        columns: vec![
                            Column { name: "id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: true },
                            Column { name: "order_id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: false },
                            Column { name: "amount".to_string(), data_type: "DECIMAL(10,2)".to_string(), is_primary_key: false },
                            Column { name: "issued_at".to_string(), data_type: "TIMESTAMP".to_string(), is_primary_key: false },
                        ],
                    },
                ],
            },
            Connection {
                name: "Development".to_string(),
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
                            Column { name: "id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: true },
                            Column { name: "name".to_string(), data_type: "VARCHAR(100)".to_string(), is_primary_key: false },
                        ],
                    },
                    Table {
                        name: "orders".to_string(),
                        row_count: 500,
                        size_bytes: 85_000,
                        columns: vec![
                            Column { name: "id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: true },
                            Column { name: "user_id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: false },
                        ],
                    },
                    Table {
                        name: "products".to_string(),
                        row_count: 100,
                        size_bytes: 15_000,
                        columns: vec![
                            Column { name: "id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: true },
                            Column { name: "name".to_string(), data_type: "VARCHAR(200)".to_string(), is_primary_key: false },
                        ],
                    },
                ],
            },
            Connection {
                name: "Staging".to_string(),
                host: "staging.example.com".to_string(),
                port: 5432,
                database: "app_staging".to_string(),
                expanded: false,
                tables: vec![
                    Table {
                        name: "users".to_string(),
                        row_count: 1000,
                        size_bytes: 160_000,
                        columns: vec![
                            Column { name: "id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: true },
                            Column { name: "name".to_string(), data_type: "VARCHAR(100)".to_string(), is_primary_key: false },
                        ],
                    },
                    Table {
                        name: "orders".to_string(),
                        row_count: 5000,
                        size_bytes: 850_000,
                        columns: vec![
                            Column { name: "id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: true },
                            Column { name: "user_id".to_string(), data_type: "INTEGER".to_string(), is_primary_key: false },
                        ],
                    },
                ],
            },
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
                vec!["1".to_string(), "Alice Johnson".to_string(), "alice@example.com".to_string(), "2024-01-15".to_string()],
                vec!["2".to_string(), "Bob Smith".to_string(), "bob@example.com".to_string(), "2024-01-16".to_string()],
                vec!["3".to_string(), "Carol Williams".to_string(), "carol@example.com".to_string(), "2024-01-17".to_string()],
                vec!["4".to_string(), "David Brown".to_string(), "david@example.com".to_string(), "2024-01-18".to_string()],
                vec!["5".to_string(), "Eve Davis".to_string(), "eve@example.com".to_string(), "2024-01-19".to_string()],
                vec!["6".to_string(), "Frank Miller".to_string(), "frank@example.com".to_string(), "2024-01-20".to_string()],
                vec!["7".to_string(), "Grace Wilson".to_string(), "grace@example.com".to_string(), "2024-01-21".to_string()],
                vec!["8".to_string(), "Henry Moore".to_string(), "henry@example.com".to_string(), "2024-01-22".to_string()],
            ],
            execution_time_ms: 23,
        });

        App {
            connections,
            selected_connection: 0,
            selected_table: Some(3), // customers
            query,
            result,
            focus: Focus::Sidebar,
            main_panel_tab: MainPanelTab::Data,
            status_message: "Connected to Production DB".to_string(),
        }
    }

    /// Update app state based on message. Returns true if app should quit.
    pub fn update(&mut self, message: Message) -> bool {
        match message {
            Message::Quit => return true,

            Message::NavigateUp => {
                match self.focus {
                    Focus::Sidebar => {
                        self.navigate_sidebar_up();
                    }
                    Focus::MainPanel => {
                        // TODO: scroll main panel up
                    }
                    Focus::QueryEditor => {
                        // TODO: move cursor up in query
                    }
                }
            }

            Message::NavigateDown => {
                match self.focus {
                    Focus::Sidebar => {
                        self.navigate_sidebar_down();
                    }
                    Focus::MainPanel => {
                        // TODO: scroll main panel down
                    }
                    Focus::QueryEditor => {
                        // TODO: move cursor down in query
                    }
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

            Message::Activate => {
                if self.focus == Focus::Sidebar {
                    self.activate_current_item();
                }
            }

            Message::ToggleExpandCollapse => {
                if self.focus == Focus::Sidebar {
                    if let Some(conn) = self.connections.get_mut(self.selected_connection) {
                        conn.expanded = !conn.expanded;
                    }
                }
            }

            Message::SwitchToSchema => {
                self.main_panel_tab = MainPanelTab::Schema;
            }

            Message::SwitchToData => {
                self.main_panel_tab = MainPanelTab::Data;
            }
        }

        false
    }

    fn navigate_sidebar_up(&mut self) {
        if let Some(conn) = self.connections.get(self.selected_connection) {
            if conn.expanded {
                // If a table is selected, move up within tables or to connection
                if let Some(table_idx) = self.selected_table {
                    if table_idx > 0 {
                        self.selected_table = Some(table_idx - 1);
                    } else {
                        self.selected_table = None;
                    }
                } else if self.selected_connection > 0 {
                    // Move to previous connection's last table (if expanded)
                    self.selected_connection -= 1;
                    let prev_conn = &self.connections[self.selected_connection];
                    if prev_conn.expanded && !prev_conn.tables.is_empty() {
                        self.selected_table = Some(prev_conn.tables.len() - 1);
                    } else {
                        self.selected_table = None;
                    }
                }
            } else if self.selected_connection > 0 {
                self.selected_connection -= 1;
                self.selected_table = None;
            }
        }
    }

    fn navigate_sidebar_down(&mut self) {
        if let Some(conn) = self.connections.get(self.selected_connection) {
            if conn.expanded {
                if let Some(table_idx) = self.selected_table {
                    if table_idx + 1 < conn.tables.len() {
                        self.selected_table = Some(table_idx + 1);
                    } else if self.selected_connection + 1 < self.connections.len() {
                        // Move to next connection
                        self.selected_connection += 1;
                        self.selected_table = None;
                    }
                } else {
                    // No table selected, select first table
                    if !conn.tables.is_empty() {
                        self.selected_table = Some(0);
                    } else if self.selected_connection + 1 < self.connections.len() {
                        self.selected_connection += 1;
                    }
                }
            } else if self.selected_connection + 1 < self.connections.len() {
                self.selected_connection += 1;
                self.selected_table = None;
            }
        }
    }

    fn activate_current_item(&mut self) {
        if let Some(table_idx) = self.selected_table {
            if let Some(conn) = self.connections.get(self.selected_connection) {
                if let Some(table) = conn.tables.get(table_idx) {
                    // Generate query for selected table
                    self.query = format!("SELECT * FROM {} LIMIT 50;", table.name);
                    self.status_message = format!(
                        "Selected: {}.{} ({} rows)",
                        conn.database, table.name, table.row_count
                    );
                }
            }
        } else {
            // Toggle expand/collapse on connection
            if let Some(conn) = self.connections.get_mut(self.selected_connection) {
                conn.expanded = !conn.expanded;
            }
        }
    }

    /// Get currently selected table (if any)
    pub fn selected_table_info(&self) -> Option<&Table> {
        let table_idx = self.selected_table?;
        let conn = self.connections.get(self.selected_connection)?;
        conn.tables.get(table_idx)
    }

    /// Get currently selected connection
    pub fn selected_connection_info(&self) -> Option<&Connection> {
        self.connections.get(self.selected_connection)
    }
}
