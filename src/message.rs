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
    SwitchToSchema,
    SwitchToData,
    // Connection modal messages
    OpenAddConnectionModal,
    // Project modal messages
    OpenAddProjectModal,
    OpenEditProjectModal,
    DeleteProject,
    // Common modal messages
    CloseModal,
    ModalConfirm,
    ModalInputChar(char),
    ModalInputBackspace,
    ModalNextField,
    ModalPrevField,
}
