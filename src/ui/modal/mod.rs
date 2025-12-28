//! Modal dialog rendering
//!
//! This module contains all modal dialog rendering functions, organized by type.

mod connection_modal;
mod helpers;
mod history_modal;
mod project_modal;
mod search_modal;
mod visibility_modal;

use crate::app::{ColumnVisibilitySettings, ModalState};
use crate::model::{Connection, Project, QueryHistory, Table};
use ratatui::Frame;

// Re-export for potential external use
#[allow(unused_imports)]
pub use helpers::{centered_rect, draw_input_field, highlight_match};

pub fn draw_modal(
    frame: &mut Frame,
    modal_state: &ModalState,
    projects: &[Project],
    connections: &[Connection],
    tables: Option<&[Table]>,
    history: &QueryHistory,
    column_visibility: &ColumnVisibilitySettings,
) {
    match modal_state {
        ModalState::None => {}
        ModalState::AddConnection(modal) => {
            connection_modal::draw_add_connection_modal(frame, modal);
        }
        ModalState::AddProject(modal) => {
            project_modal::draw_project_modal(frame, modal, " Add Project ");
        }
        ModalState::EditProject(_, modal) => {
            project_modal::draw_project_modal(frame, modal, " Edit Project ");
        }
        ModalState::DeleteProject(modal) => {
            project_modal::draw_delete_project_modal(frame, modal);
        }
        ModalState::SearchProject(modal) => {
            search_modal::draw_search_project_modal(frame, modal, projects);
        }
        ModalState::SearchConnection(modal) => {
            search_modal::draw_search_connection_modal(frame, modal, connections);
        }
        ModalState::SearchTable(modal) => {
            if let Some(tables) = tables {
                search_modal::draw_search_table_modal(frame, modal, tables);
            }
        }
        ModalState::UnifiedSearch(modal) => {
            search_modal::draw_unified_search_modal(
                frame,
                modal,
                connections,
                tables.unwrap_or(&[]),
            );
        }
        ModalState::History(modal) => {
            history_modal::draw_history_modal(frame, modal, history);
        }
        ModalState::ColumnVisibility(modal) => {
            visibility_modal::draw_column_visibility_modal(frame, modal, column_visibility);
        }
    }
}
