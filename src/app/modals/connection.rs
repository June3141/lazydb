//! Connection modal state

use super::super::modal_fields::ConnectionModalField;

/// Modal for adding a new connection
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

impl AddConnectionModal {
    /// Create a modal pre-filled with existing connection data (for editing)
    pub fn from_connection(conn: &crate::model::Connection) -> Self {
        Self {
            name: conn.name.clone(),
            host: conn.host.clone(),
            port: conn.port.to_string(),
            user: conn.username.clone(),
            password: conn.password.clone(),
            database: conn.database.clone(),
            focused_field: ConnectionModalField::Name,
        }
    }
}
