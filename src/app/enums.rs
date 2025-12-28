//! Core enumeration types for app state management

/// Focus areas in the application
#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Sidebar,
    QueryEditor,
    MainPanel,
}

/// Main panel tab selection
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainPanelTab {
    Schema,
    Data,
    Relations,
}

/// Sub-tabs for the Schema tab
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SchemaSubTab {
    #[default]
    Columns,
    Indexes,
    ForeignKeys,
    Constraints,
    /// Triggers defined on the table
    Triggers,
    /// View/Materialized View definition (SELECT statement)
    Definition,
}

/// Sidebar display mode - switches between Projects list and Connections list
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarMode {
    Projects,
    Connections(usize), // project index
}
