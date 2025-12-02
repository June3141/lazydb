use super::table::Table;

#[derive(Debug, Clone)]
pub struct Connection {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub expanded: bool,
    pub tables: Vec<Table>,
}
