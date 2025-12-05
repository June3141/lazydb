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
    // Modal messages
    OpenAddConnectionModal,
    CloseModal,
    ModalConfirm,
    ModalInputChar(char),
    ModalInputBackspace,
    ModalNextField,
    ModalPrevField,
}
