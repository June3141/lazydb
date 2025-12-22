//! Extended schema information for database objects

#![allow(dead_code)]

/// Represents a database column with full metadata
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub data_type: String,
    pub is_nullable: bool,
    pub default_value: Option<String>,
    pub is_primary_key: bool,
    pub is_unique: bool,
    pub is_auto_increment: bool,
    pub comment: Option<String>,
    pub ordinal_position: usize,
}

impl Column {
    pub fn new(name: impl Into<String>, data_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            is_nullable: true,
            default_value: None,
            is_primary_key: false,
            is_unique: false,
            is_auto_increment: false,
            comment: None,
            ordinal_position: 0,
        }
    }

    pub fn primary_key(mut self) -> Self {
        self.is_primary_key = true;
        self.is_nullable = false;
        self
    }

    pub fn not_null(mut self) -> Self {
        self.is_nullable = false;
        self
    }

    pub fn default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    pub fn unique(mut self) -> Self {
        self.is_unique = true;
        self
    }

    pub fn auto_increment(mut self) -> Self {
        self.is_auto_increment = true;
        self
    }

    pub fn comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    pub fn position(mut self, pos: usize) -> Self {
        self.ordinal_position = pos;
        self
    }
}

/// Index type enumeration
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexType {
    Primary,
    Unique,
    Index,
    Fulltext,
    Spatial,
}

impl std::fmt::Display for IndexType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexType::Primary => write!(f, "PRIMARY"),
            IndexType::Unique => write!(f, "UNIQUE"),
            IndexType::Index => write!(f, "INDEX"),
            IndexType::Fulltext => write!(f, "FULLTEXT"),
            IndexType::Spatial => write!(f, "SPATIAL"),
        }
    }
}

/// Index method (B-tree, Hash, etc.)
///
/// Note: Some variants are database-specific:
/// - `BTree`: Supported by all databases (PostgreSQL, MySQL, SQLite)
/// - `Hash`: Supported by PostgreSQL and MySQL
/// - `Gist`: PostgreSQL only (Generalized Search Tree)
/// - `Gin`: PostgreSQL only (Generalized Inverted Index)
/// - `Brin`: PostgreSQL only (Block Range Index)
/// - `Other`: For database-specific methods not listed above
#[derive(Debug, Clone, PartialEq)]
pub enum IndexMethod {
    /// B-tree index - supported by all databases
    BTree,
    /// Hash index - supported by PostgreSQL and MySQL
    Hash,
    /// GiST (Generalized Search Tree) - PostgreSQL only
    Gist,
    /// GIN (Generalized Inverted Index) - PostgreSQL only
    Gin,
    /// BRIN (Block Range Index) - PostgreSQL only
    Brin,
    /// Other database-specific index methods
    Other(String),
}

impl std::fmt::Display for IndexMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexMethod::BTree => write!(f, "BTREE"),
            IndexMethod::Hash => write!(f, "HASH"),
            IndexMethod::Gist => write!(f, "GIST"),
            IndexMethod::Gin => write!(f, "GIN"),
            IndexMethod::Brin => write!(f, "BRIN"),
            IndexMethod::Other(s) => write!(f, "{}", s),
        }
    }
}

/// Sort order for index columns
#[derive(Debug, Clone, PartialEq)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl std::fmt::Display for SortOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortOrder::Asc => write!(f, "ASC"),
            SortOrder::Desc => write!(f, "DESC"),
        }
    }
}

/// Column in an index with optional sort order
#[derive(Debug, Clone)]
pub struct IndexColumn {
    pub name: String,
    pub order: SortOrder,
}

impl IndexColumn {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            order: SortOrder::Asc,
        }
    }

    pub fn desc(mut self) -> Self {
        self.order = SortOrder::Desc;
        self
    }
}

/// Database index
#[derive(Debug, Clone)]
pub struct Index {
    pub name: String,
    pub index_type: IndexType,
    pub method: IndexMethod,
    pub columns: Vec<IndexColumn>,
    pub is_unique: bool,
    pub comment: Option<String>,
}

impl Index {
    pub fn new(name: impl Into<String>, index_type: IndexType) -> Self {
        Self {
            name: name.into(),
            index_type,
            method: IndexMethod::BTree,
            columns: Vec::new(),
            is_unique: matches!(index_type, IndexType::Primary | IndexType::Unique),
            comment: None,
        }
    }

    pub fn with_columns(mut self, columns: Vec<IndexColumn>) -> Self {
        self.columns = columns;
        self
    }

    pub fn method(mut self, method: IndexMethod) -> Self {
        self.method = method;
        self
    }
}

/// Foreign key action on update/delete
#[derive(Debug, Clone, PartialEq)]
pub enum ForeignKeyAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull,
    SetDefault,
}

impl std::fmt::Display for ForeignKeyAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ForeignKeyAction::NoAction => write!(f, "NO ACTION"),
            ForeignKeyAction::Restrict => write!(f, "RESTRICT"),
            ForeignKeyAction::Cascade => write!(f, "CASCADE"),
            ForeignKeyAction::SetNull => write!(f, "SET NULL"),
            ForeignKeyAction::SetDefault => write!(f, "SET DEFAULT"),
        }
    }
}

/// Foreign key constraint
#[derive(Debug, Clone)]
pub struct ForeignKey {
    pub name: String,
    pub columns: Vec<String>,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
    pub on_update: ForeignKeyAction,
    pub on_delete: ForeignKeyAction,
}

impl ForeignKey {
    pub fn new(
        name: impl Into<String>,
        columns: Vec<String>,
        referenced_table: impl Into<String>,
        referenced_columns: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            columns,
            referenced_table: referenced_table.into(),
            referenced_columns,
            on_update: ForeignKeyAction::NoAction,
            on_delete: ForeignKeyAction::NoAction,
        }
    }

    pub fn on_update(mut self, action: ForeignKeyAction) -> Self {
        self.on_update = action;
        self
    }

    pub fn on_delete(mut self, action: ForeignKeyAction) -> Self {
        self.on_delete = action;
        self
    }
}

/// Constraint type
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintType {
    PrimaryKey,
    Unique,
    ForeignKey,
    Check,
    NotNull,
    Default,
    Exclusion,
}

impl std::fmt::Display for ConstraintType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintType::PrimaryKey => write!(f, "PRIMARY KEY"),
            ConstraintType::Unique => write!(f, "UNIQUE"),
            ConstraintType::ForeignKey => write!(f, "FOREIGN KEY"),
            ConstraintType::Check => write!(f, "CHECK"),
            ConstraintType::NotNull => write!(f, "NOT NULL"),
            ConstraintType::Default => write!(f, "DEFAULT"),
            ConstraintType::Exclusion => write!(f, "EXCLUSION"),
        }
    }
}

/// Generic constraint
#[derive(Debug, Clone)]
pub struct Constraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub columns: Vec<String>,
    pub definition: Option<String>,
}

impl Constraint {
    pub fn new(name: impl Into<String>, constraint_type: ConstraintType) -> Self {
        Self {
            name: name.into(),
            constraint_type,
            columns: Vec::new(),
            definition: None,
        }
    }

    pub fn with_columns(mut self, columns: Vec<String>) -> Self {
        self.columns = columns;
        self
    }

    pub fn with_definition(mut self, definition: impl Into<String>) -> Self {
        self.definition = Some(definition.into());
        self
    }
}

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

    /// Get primary key columns
    pub fn primary_key_columns(&self) -> Vec<&Column> {
        self.columns.iter().filter(|c| c.is_primary_key).collect()
    }

    /// Get foreign key references to this table
    pub fn incoming_references<'a>(
        &self,
        all_tables: &'a [Table],
    ) -> Vec<(&'a Table, &'a ForeignKey)> {
        all_tables
            .iter()
            .flat_map(|t| {
                t.foreign_keys
                    .iter()
                    .filter(|fk| fk.referenced_table == self.name)
                    .map(move |fk| (t, fk))
            })
            .collect()
    }

    /// Get full qualified name
    pub fn full_name(&self) -> String {
        match &self.schema {
            Some(schema) => format!("{}.{}", schema, self.name),
            None => self.name.clone(),
        }
    }
}
