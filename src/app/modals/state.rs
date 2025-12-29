//! Modal state enum

use super::connection::AddConnectionModal;
use super::history::HistoryModal;
use super::project::{DeleteProjectModal, ProjectModal, SearchProjectModal};
use super::query_input::QueryInputModal;
use super::search::{SearchConnectionModal, SearchTableModal, UnifiedSearchModal};
use super::visibility::ColumnVisibilityModal;

/// Current modal state
#[derive(Debug, Clone)]
pub enum ModalState {
    None,
    AddConnection(AddConnectionModal),
    AddProject(ProjectModal),
    EditProject(usize, ProjectModal), // (project index, modal)
    DeleteProject(DeleteProjectModal),
    SearchProject(SearchProjectModal),
    SearchConnection(SearchConnectionModal),
    SearchTable(SearchTableModal),
    UnifiedSearch(UnifiedSearchModal),
    History(HistoryModal),
    ColumnVisibility(ColumnVisibilityModal),
    /// Query input modal for executing arbitrary SQL queries
    QueryInput(QueryInputModal),
}
