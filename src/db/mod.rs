//! Database provider abstraction layer
//!
//! This module provides a database-agnostic interface for interacting with
//! different database systems. Currently supports PostgreSQL, with planned
//! support for MySQL and SQLite.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐     ┌──────────────┐     ┌──────────────────┐
//! │     App     │────▶│  DbWorker    │────▶│ DatabaseProvider │
//! │  (main UI)  │◀────│  (thread)    │◀────│   (PostgreSQL)   │
//! └─────────────┘     └──────────────┘     └──────────────────┘
//!       │                    │
//!       │   DbCommand        │   sync DB operations
//!       └───────────────────▶│
//!                            │
//!       ◀────────────────────┘
//!           DbResponse
//! ```
//!
//! # Key Components
//!
//! - [`DatabaseProvider`] - Trait defining the database interface
//! - [`DbWorkerHandle`] - Handle for sending commands to the background worker
//! - [`DbCommand`] / [`DbResponse`] - Message types for async operations
//! - [`PostgresProvider`] - PostgreSQL implementation

#![allow(dead_code)]
#![allow(unused_imports)]

mod async_bridge;
mod postgres;
mod provider;
mod worker;

pub use async_bridge::{ConnectionParams, DbCommand, DbResponse};
pub use postgres::PostgresProvider;
pub use provider::{DatabaseProvider, DatabaseType, ProviderError};
pub use worker::{spawn_db_worker, DbWorkerHandle};
