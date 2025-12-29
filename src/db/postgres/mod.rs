//! PostgreSQL database provider implementation
//!
//! This module provides a PostgreSQL implementation of the DatabaseProvider trait.

mod helpers;
mod pool;
mod provider;
mod queries;
mod trait_impl;

#[cfg(test)]
mod tests;

// Re-export from parent for internal use
pub(super) use super::provider::{DatabaseProvider, DatabaseType, ProviderError};

// Re-export the main types
pub use pool::{ConnectionPool, PoolConfig, PoolState};
pub use provider::PostgresProvider;
