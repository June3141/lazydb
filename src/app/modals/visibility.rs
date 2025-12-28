//! Column visibility modal state

use super::super::enums::SchemaSubTab;
use super::super::visibility::{
    ColumnsVisibility, ConstraintsVisibility, ForeignKeysVisibility, IndexesVisibility,
    TriggersVisibility,
};

/// Modal for configuring column visibility
#[derive(Debug, Clone)]
pub struct ColumnVisibilityModal {
    pub tab: SchemaSubTab,
    pub selected_idx: usize,
}

impl ColumnVisibilityModal {
    pub fn new(tab: SchemaSubTab) -> Self {
        Self {
            tab,
            selected_idx: 0,
        }
    }

    pub fn column_count(&self) -> usize {
        match self.tab {
            SchemaSubTab::Columns => ColumnsVisibility::all_columns().len(),
            SchemaSubTab::Indexes => IndexesVisibility::all_columns().len(),
            SchemaSubTab::ForeignKeys => ForeignKeysVisibility::all_columns().len(),
            SchemaSubTab::Constraints => ConstraintsVisibility::all_columns().len(),
            SchemaSubTab::Triggers => TriggersVisibility::all_columns().len(),
            SchemaSubTab::Definition => 0, // No visibility settings for Definition tab
        }
    }

    pub fn navigate_up(&mut self) {
        let count = self.column_count();
        if count > 0 {
            if self.selected_idx > 0 {
                self.selected_idx -= 1;
            } else {
                self.selected_idx = count - 1;
            }
        }
    }

    pub fn navigate_down(&mut self) {
        let count = self.column_count();
        if count > 0 {
            if self.selected_idx + 1 < count {
                self.selected_idx += 1;
            } else {
                self.selected_idx = 0;
            }
        }
    }
}
