//! Constraint types and structures

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
