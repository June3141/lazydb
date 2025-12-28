//! Database index types and structures

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
