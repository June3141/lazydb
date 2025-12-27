//! Table types and structures

#[cfg(test)]
use super::ForeignKeyAction;
use super::{Column, Constraint, ForeignKey, Index};

/// Table type
#[derive(Debug, Clone, PartialEq)]
pub enum TableType {
    BaseTable,
    View,
    MaterializedView,
    ForeignTable,
    Temporary,
}

impl std::fmt::Display for TableType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TableType::BaseTable => write!(f, "TABLE"),
            TableType::View => write!(f, "VIEW"),
            TableType::MaterializedView => write!(f, "MATERIALIZED VIEW"),
            TableType::ForeignTable => write!(f, "FOREIGN TABLE"),
            TableType::Temporary => write!(f, "TEMPORARY"),
        }
    }
}

impl TableType {
    /// Returns the icon for this table type (Nerd Font icons)
    pub fn icon(&self) -> &'static str {
        match self {
            TableType::BaseTable => "󰓫",      // table icon
            TableType::View => "󰈈",           // eye icon (view)
            TableType::MaterializedView => "󱁉", // cached/materialized icon
            TableType::ForeignTable => "󰌷",   // link/external icon
            TableType::Temporary => "󰔛",      // clock/temporary icon
        }
    }

    /// Returns true if this is a View or MaterializedView
    pub fn is_view(&self) -> bool {
        matches!(self, TableType::View | TableType::MaterializedView)
    }
}

/// Extended table information
#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub schema: Option<String>,
    pub table_type: TableType,
    pub columns: Vec<Column>,
    pub indexes: Vec<Index>,
    pub foreign_keys: Vec<ForeignKey>,
    pub constraints: Vec<Constraint>,
    pub row_count: usize,
    pub size_bytes: u64,
    pub comment: Option<String>,
    /// Whether detailed schema information has been loaded
    pub details_loaded: bool,
    /// View definition (SELECT statement) for Views and Materialized Views
    pub view_definition: Option<String>,
}

impl Table {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            schema: None,
            table_type: TableType::BaseTable,
            columns: Vec::new(),
            indexes: Vec::new(),
            foreign_keys: Vec::new(),
            constraints: Vec::new(),
            row_count: 0,
            size_bytes: 0,
            comment: None,
            details_loaded: false,
            view_definition: None,
        }
    }

    pub fn with_schema(mut self, schema: impl Into<String>) -> Self {
        self.schema = Some(schema.into());
        self
    }

    pub fn with_columns(mut self, columns: Vec<Column>) -> Self {
        self.columns = columns;
        self
    }

    pub fn with_indexes(mut self, indexes: Vec<Index>) -> Self {
        self.indexes = indexes;
        self
    }

    pub fn with_foreign_keys(mut self, fks: Vec<ForeignKey>) -> Self {
        self.foreign_keys = fks;
        self
    }

    pub fn with_constraints(mut self, constraints: Vec<Constraint>) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn with_stats(mut self, row_count: usize, size_bytes: u64) -> Self {
        self.row_count = row_count;
        self.size_bytes = size_bytes;
        self
    }

    pub fn with_table_type(mut self, table_type: TableType) -> Self {
        self.table_type = table_type;
        self
    }

    pub fn with_view_definition(mut self, definition: impl Into<String>) -> Self {
        self.view_definition = Some(definition.into());
        self
    }

    /// Get primary key columns
    pub fn primary_key_columns(&self) -> Vec<&Column> {
        self.columns.iter().filter(|c| c.is_primary_key).collect()
    }

    /// Get foreign key references to this table
    pub fn incoming_references<'a>(
        &self,
        all_tables: &'a [Table],
    ) -> Vec<(&'a Table, &'a ForeignKey)> {
        let full_name = self.full_name();
        all_tables
            .iter()
            .flat_map(|t| {
                t.foreign_keys
                    .iter()
                    .filter(|fk| self.matches_reference(&fk.referenced_table, &full_name))
                    .map(move |fk| (t, fk))
            })
            .collect()
    }

    /// Check if a reference matches this table (handles both qualified and unqualified names)
    fn matches_reference(&self, reference: &str, full_name: &str) -> bool {
        // Exact match with full qualified name (schema.table)
        if reference == full_name {
            return true;
        }
        // Exact match with table name only
        if reference == self.name {
            return true;
        }
        // Reference is qualified but self has no schema - extract table name from reference
        let ref_table = reference.rsplit('.').next().unwrap_or(reference);
        if self.schema.is_none() && ref_table == self.name {
            return true;
        }
        false
    }

    /// Get full qualified name
    pub fn full_name(&self) -> String {
        match &self.schema {
            Some(schema) => format!("{}.{}", schema, self.name),
            None => self.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_table(name: &str, schema: Option<&str>) -> Table {
        let mut table = Table::new(name);
        if let Some(s) = schema {
            table = table.with_schema(s);
        }
        table
    }

    fn create_table_with_fk(name: &str, fk_referenced_table: &str) -> Table {
        Table::new(name).with_foreign_keys(vec![ForeignKey {
            name: "fk_test".to_string(),
            columns: vec!["id".to_string()],
            referenced_table: fk_referenced_table.to_string(),
            referenced_columns: vec!["id".to_string()],
            on_update: ForeignKeyAction::NoAction,
            on_delete: ForeignKeyAction::NoAction,
        }])
    }

    #[test]
    fn test_matches_reference_exact_name_match() {
        let table = create_test_table("users", None);
        let full_name = table.full_name();
        assert!(table.matches_reference("users", &full_name));
    }

    #[test]
    fn test_matches_reference_qualified_table_exact_match() {
        let table = create_test_table("users", Some("public"));
        let full_name = table.full_name();
        assert!(table.matches_reference("public.users", &full_name));
    }

    #[test]
    fn test_matches_reference_unqualified_ref_to_qualified_table() {
        let table = create_test_table("users", Some("public"));
        let full_name = table.full_name();
        assert!(table.matches_reference("users", &full_name));
    }

    #[test]
    fn test_matches_reference_qualified_ref_to_unqualified_table() {
        let table = create_test_table("users", None);
        let full_name = table.full_name();
        assert!(table.matches_reference("public.users", &full_name));
    }

    #[test]
    fn test_matches_reference_different_schema() {
        let table = create_test_table("users", Some("public"));
        let full_name = table.full_name();
        assert!(!table.matches_reference("other.users", &full_name));
    }

    #[test]
    fn test_matches_reference_different_table() {
        let table = create_test_table("users", Some("public"));
        let full_name = table.full_name();
        assert!(!table.matches_reference("public.orders", &full_name));
    }

    #[test]
    fn test_matches_reference_multiple_dots() {
        let table = create_test_table("users", None);
        let full_name = table.full_name();
        // "catalog.schema.users" should match "users" when table has no schema
        assert!(table.matches_reference("catalog.schema.users", &full_name));
    }

    #[test]
    fn test_incoming_references_unqualified() {
        let users = create_test_table("users", None);
        let orders = create_table_with_fk("orders", "users");
        let all_tables = vec![users.clone(), orders];

        let refs = users.incoming_references(&all_tables);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].0.name, "orders");
    }

    #[test]
    fn test_incoming_references_qualified_ref_to_unqualified_table() {
        let users = create_test_table("users", None);
        let orders = create_table_with_fk("orders", "public.users");
        let all_tables = vec![users.clone(), orders];

        let refs = users.incoming_references(&all_tables);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].0.name, "orders");
    }

    #[test]
    fn test_incoming_references_qualified_table_with_matching_ref() {
        let users = create_test_table("users", Some("public"));
        let orders = create_table_with_fk("orders", "public.users");
        let all_tables = vec![users.clone(), orders];

        let refs = users.incoming_references(&all_tables);
        assert_eq!(refs.len(), 1);
        assert_eq!(refs[0].0.name, "orders");
    }

    #[test]
    fn test_incoming_references_no_match_different_schema() {
        let users = create_test_table("users", Some("public"));
        let orders = create_table_with_fk("orders", "other.users");
        let all_tables = vec![users.clone(), orders];

        let refs = users.incoming_references(&all_tables);
        assert_eq!(refs.len(), 0);
    }

    // ===== TableType icon tests =====

    #[test]
    fn test_table_type_icon_base_table() {
        assert_eq!(TableType::BaseTable.icon(), "󰓫");
    }

    #[test]
    fn test_table_type_icon_view() {
        assert_eq!(TableType::View.icon(), "󰈈");
    }

    #[test]
    fn test_table_type_icon_materialized_view() {
        assert_eq!(TableType::MaterializedView.icon(), "󱁉");
    }

    #[test]
    fn test_table_type_icon_foreign_table() {
        assert_eq!(TableType::ForeignTable.icon(), "󰌷");
    }

    #[test]
    fn test_table_type_icon_temporary() {
        assert_eq!(TableType::Temporary.icon(), "󰔛");
    }

    // ===== TableType is_view tests =====

    #[test]
    fn test_table_type_is_view() {
        assert!(!TableType::BaseTable.is_view());
        assert!(TableType::View.is_view());
        assert!(TableType::MaterializedView.is_view());
        assert!(!TableType::ForeignTable.is_view());
        assert!(!TableType::Temporary.is_view());
    }

    // ===== Table with_table_type and with_view_definition tests =====

    #[test]
    fn test_table_with_table_type() {
        let table = Table::new("my_view").with_table_type(TableType::View);
        assert_eq!(table.table_type, TableType::View);
    }

    #[test]
    fn test_table_with_view_definition() {
        let definition = "SELECT id, name FROM users WHERE active = true";
        let table = Table::new("active_users")
            .with_table_type(TableType::View)
            .with_view_definition(definition);

        assert_eq!(table.view_definition, Some(definition.to_string()));
    }

    #[test]
    fn test_table_view_definition_default_none() {
        let table = Table::new("users");
        assert_eq!(table.view_definition, None);
    }
}
