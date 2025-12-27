//! Column visibility settings for schema sub-tabs

/// Column visibility settings for Columns tab
#[derive(Debug, Clone)]
pub struct ColumnsVisibility {
    pub show_icon: bool,
    pub show_name: bool,
    pub show_type: bool,
    pub show_nullable: bool,
    pub show_default: bool,
    pub show_key: bool,
}

impl Default for ColumnsVisibility {
    fn default() -> Self {
        Self {
            show_icon: true,
            show_name: true,
            show_type: true,
            show_nullable: true,
            show_default: true,
            show_key: true,
        }
    }
}

impl ColumnsVisibility {
    pub fn all_columns() -> &'static [&'static str] {
        &["Icon", "Name", "Type", "Null", "Default", "Key"]
    }

    pub fn is_visible(&self, index: usize) -> bool {
        match index {
            0 => self.show_icon,
            1 => self.show_name,
            2 => self.show_type,
            3 => self.show_nullable,
            4 => self.show_default,
            5 => self.show_key,
            _ => false,
        }
    }

    pub fn toggle(&mut self, index: usize) {
        match index {
            0 => self.show_icon = !self.show_icon,
            1 => self.show_name = !self.show_name,
            2 => self.show_type = !self.show_type,
            3 => self.show_nullable = !self.show_nullable,
            4 => self.show_default = !self.show_default,
            5 => self.show_key = !self.show_key,
            _ => {}
        }
    }
}

/// Column visibility settings for Indexes tab
#[derive(Debug, Clone)]
pub struct IndexesVisibility {
    pub show_name: bool,
    pub show_type: bool,
    pub show_method: bool,
    pub show_columns: bool,
}

impl Default for IndexesVisibility {
    fn default() -> Self {
        Self {
            show_name: true,
            show_type: true,
            show_method: true,
            show_columns: true,
        }
    }
}

impl IndexesVisibility {
    pub fn all_columns() -> &'static [&'static str] {
        &["Name", "Type", "Method", "Columns"]
    }

    pub fn is_visible(&self, index: usize) -> bool {
        match index {
            0 => self.show_name,
            1 => self.show_type,
            2 => self.show_method,
            3 => self.show_columns,
            _ => false,
        }
    }

    pub fn toggle(&mut self, index: usize) {
        match index {
            0 => self.show_name = !self.show_name,
            1 => self.show_type = !self.show_type,
            2 => self.show_method = !self.show_method,
            3 => self.show_columns = !self.show_columns,
            _ => {}
        }
    }
}

/// Column visibility settings for Foreign Keys tab
#[derive(Debug, Clone)]
pub struct ForeignKeysVisibility {
    pub show_name: bool,
    pub show_column: bool,
    pub show_references: bool,
    pub show_on_delete: bool,
    pub show_on_update: bool,
}

impl Default for ForeignKeysVisibility {
    fn default() -> Self {
        Self {
            show_name: true,
            show_column: true,
            show_references: true,
            show_on_delete: true,
            show_on_update: true,
        }
    }
}

impl ForeignKeysVisibility {
    pub fn all_columns() -> &'static [&'static str] {
        &["Name", "Column", "References", "ON DELETE", "ON UPDATE"]
    }

    pub fn is_visible(&self, index: usize) -> bool {
        match index {
            0 => self.show_name,
            1 => self.show_column,
            2 => self.show_references,
            3 => self.show_on_delete,
            4 => self.show_on_update,
            _ => false,
        }
    }

    pub fn toggle(&mut self, index: usize) {
        match index {
            0 => self.show_name = !self.show_name,
            1 => self.show_column = !self.show_column,
            2 => self.show_references = !self.show_references,
            3 => self.show_on_delete = !self.show_on_delete,
            4 => self.show_on_update = !self.show_on_update,
            _ => {}
        }
    }
}

/// Column visibility settings for Constraints tab
#[derive(Debug, Clone)]
pub struct ConstraintsVisibility {
    pub show_name: bool,
    pub show_type: bool,
    pub show_columns: bool,
    pub show_definition: bool,
}

impl Default for ConstraintsVisibility {
    fn default() -> Self {
        Self {
            show_name: true,
            show_type: true,
            show_columns: true,
            show_definition: true,
        }
    }
}

impl ConstraintsVisibility {
    pub fn all_columns() -> &'static [&'static str] {
        &["Name", "Type", "Columns", "Definition"]
    }

    pub fn is_visible(&self, index: usize) -> bool {
        match index {
            0 => self.show_name,
            1 => self.show_type,
            2 => self.show_columns,
            3 => self.show_definition,
            _ => false,
        }
    }

    pub fn toggle(&mut self, index: usize) {
        match index {
            0 => self.show_name = !self.show_name,
            1 => self.show_type = !self.show_type,
            2 => self.show_columns = !self.show_columns,
            3 => self.show_definition = !self.show_definition,
            _ => {}
        }
    }
}

/// Column visibility settings for all schema sub-tabs
#[derive(Debug, Clone, Default)]
pub struct ColumnVisibilitySettings {
    pub columns: ColumnsVisibility,
    pub indexes: IndexesVisibility,
    pub foreign_keys: ForeignKeysVisibility,
    pub constraints: ConstraintsVisibility,
}
