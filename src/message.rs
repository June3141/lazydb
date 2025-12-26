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
    // Schema sub-tabs (1-4 keys)
    SwitchToColumns,
    SwitchToIndexes,
    SwitchToForeignKeys,
    SwitchToConstraints,
    // Connection modal messages
    OpenAddConnectionModal,
    // Project modal messages
    OpenAddProjectModal,
    OpenEditProjectModal,
    DeleteProject,
    // Search modal messages
    OpenSearchProjectModal,
    OpenSearchConnectionModal,
    SearchConfirm,
    SearchConnectionConfirm,
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
