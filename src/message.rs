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
}
