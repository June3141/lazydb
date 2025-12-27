use super::connection::Connection;
use crate::config::ProjectFile;

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
}

impl From<ProjectFile> for Project {
    fn from(file: ProjectFile) -> Self {
        Self {
            name: file.project.name,
            connections: file.connections.into_iter().map(Connection::from).collect(),
        }
    }
}
