//! PostgreSQL database provider implementation
//!
//! This module provides a PostgreSQL implementation of the DatabaseProvider trait.

mod helpers;
mod internal;
mod provider;
mod trait_impl;

#[cfg(test)]
mod tests;

// Re-export from parent for internal use
pub(super) use super::provider::{DatabaseProvider, DatabaseType, ProviderError};

// Re-export the main type
pub use provider::PostgresProvider;
