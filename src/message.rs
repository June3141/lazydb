#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Quit,
    NavigateUp,
    NavigateDown,
    NextFocus,
    PrevFocus,
    Select,
    ToggleExpandCollapse,
    SwitchToSchema,
    SwitchToData,
}
