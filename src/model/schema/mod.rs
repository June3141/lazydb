//! Extended schema information for database objects

#![allow(dead_code)]

mod column;
mod constraint;
mod foreign_key;
mod index;
mod table;

pub use column::Column;
pub use constraint::{Constraint, ConstraintType};
pub use foreign_key::{ForeignKey, ForeignKeyAction};
pub use index::{Index, IndexColumn, IndexMethod, IndexType, SortOrder};
pub use table::{Table, TableType};
