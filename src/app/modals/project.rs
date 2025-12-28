//! Project modal states

use crate::model::Project;

use super::super::modal_fields::{ConfirmModalField, ProjectModalField};

/// Modal for adding/editing a project
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

/// Modal for confirming project deletion
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
