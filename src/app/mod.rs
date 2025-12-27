//! Application state and logic
//!
//! This module contains the core application state, modal dialogs,
//! and related types organized into submodules.

mod enums;
mod modal_fields;
mod modals;
mod state;
mod visibility;

// Re-export all public types for external use
pub use enums::{Focus, MainPanelTab, SchemaSubTab, SidebarMode};
pub use modal_fields::{ConfirmModalField, ConnectionModalField, ProjectModalField};
pub use modals::{
    AddConnectionModal, ColumnVisibilityModal, DeleteProjectModal, HistoryModal, ModalState,
    ProjectModal, SearchConnectionModal, SearchProjectModal, SearchTableModal,
    UnifiedSearchModal, UnifiedSearchSection,
};
pub use state::App;
pub use visibility::{
    ColumnVisibilitySettings, ColumnsVisibility, ConstraintsVisibility, ForeignKeysVisibility,
    IndexesVisibility,
};
