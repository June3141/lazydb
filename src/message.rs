//! Application messages for the TEA (The Elm Architecture) pattern
//!
//! This module defines all messages that can be sent to update the application state.
//! Messages are triggered by user input (keyboard events) and processed by the
//! `App::update()` method.
//!
//! # Message Categories
//!
//! - **Navigation** - Moving between UI elements
//! - **Tab switching** - Switching main panel and sub-tabs
//! - **Modal operations** - Opening/closing modal dialogs
//! - **Pagination** - Navigating through paginated data
//! - **Data table** - Scrolling within the data table

/// Application message type
///
/// Each variant represents a specific user action or state change request.
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Quit,
    NavigateUp,
    NavigateDown,
    NextFocus,
    PrevFocus,
    // Directional pane focus (Shift + h/j/k/l or arrow keys)
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    Activate,
    GoBack,
    // Main panel tabs
    SwitchToSchema,
    SwitchToData,
    SwitchToRelations,
    // Schema sub-tabs (1-6 keys)
    SwitchToColumns,
    SwitchToIndexes,
    SwitchToForeignKeys,
    SwitchToConstraints,
    SwitchToTriggers,
    SwitchToDefinition,
    // Connection modal messages
    OpenAddConnectionModal,
    // Project modal messages
    OpenAddProjectModal,
    OpenEditProjectModal,
    DeleteProject,
    // Search modal messages
    OpenSearchProjectModal,
    #[allow(dead_code)] // Legacy: kept for potential future use
    OpenSearchConnectionModal,
    #[allow(dead_code)] // Legacy: kept for potential future use
    OpenSearchTableModal,
    OpenUnifiedSearchModal,
    SearchConfirm,
    SearchConnectionConfirm,
    TableSearchConfirm,
    UnifiedSearchConfirm,
    UnifiedSearchSwitchSection,
    // Column visibility modal messages
    OpenColumnVisibilityModal,
    ToggleColumnVisibility,
    // Common modal messages
    CloseModal,
    ModalConfirm,
    ModalInputChar(char),
    ModalInputBackspace,
    ModalNextField,
    ModalPrevField,
    // Query history messages
    OpenHistoryModal,
    HistoryNavigateUp,
    HistoryNavigateDown,
    HistorySelectEntry,
    ClearHistory,
    // Query input modal messages
    OpenQueryInputModal,
    QueryInputChar(char),
    QueryInputBackspace,
    QueryInputDelete,
    QueryInputNewline,
    QueryInputCursorLeft,
    QueryInputCursorRight,
    QueryInputCursorUp,
    QueryInputCursorDown,
    QueryInputCursorHome,
    QueryInputCursorEnd,
    QueryInputClear,
    QueryInputExecute,
    // Pagination messages
    PageNext,
    PagePrev,
    PageFirst,
    PageLast,
    PageSizeCycle,
    // Data table navigation messages
    DataTableUp,
    DataTableDown,
    DataTablePageUp,
    DataTablePageDown,
    DataTableFirst,
    DataTableLast,
}
