use super::connection::Connection;

#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub connections: Vec<Connection>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            connections: Vec::new(),
        }
    }

    pub fn with_connections(mut self, connections: Vec<Connection>) -> Self {
        self.connections = connections;
        self
    }
}
