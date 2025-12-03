#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Quit,
    NavigateUp,
    NavigateDown,
    NextFocus,
    PrevFocus,
    Activate,
    ToggleExpandCollapse,
    SwitchToSchema,
    SwitchToData,
    // Modal messages
    OpenAddConnectionModal,
    CloseModal,
    ModalConfirm,
    ModalInputChar(char),
    ModalInputBackspace,
    ModalNextField,
    ModalPrevField,
}
