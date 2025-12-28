//! Data models for lazydb
//!
//! This module contains all data structures used throughout the application:
//! - [`Connection`] - Database connection information
//! - [`Project`] - Project containing multiple connections
//! - [`QueryResult`] - Results from SQL query execution
//! - [`QueryHistory`] - Persisted query history
//! - [`schema`] - Database schema models (tables, columns, indexes, etc.)

mod connection;
pub mod history;
mod project;
mod query;
pub mod schema;

pub use connection::Connection;
pub use history::{HistoryEntry, QueryHistory};
pub use project::Project;
pub use query::{Pagination, QueryResult};
pub use schema::{ConstraintType, ForeignKey, IndexType, SortOrder, Table, TriggerTiming};
