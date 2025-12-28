//! Query history modal state

/// Query history modal state
#[derive(Debug, Clone, Default)]
pub struct HistoryModal {
    /// Currently selected index in the history list
    pub selected_idx: usize,
}
