//! PostgreSQL internal query modules
//!
//! Each module contains query logic for specific table metadata types.

mod columns;
mod constraints;
mod foreign_keys;
mod indexes;
mod routines;
mod stats;
mod triggers;

use postgres::Client;

use crate::model::schema::{Column, Constraint, ForeignKey, Index, Routine, Trigger};

use super::ProviderError;

/// Internal helper methods for fetching table metadata
pub struct InternalQueries;

impl InternalQueries {
    pub fn get_columns(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<Vec<Column>, ProviderError> {
        columns::get_columns(client, table_name, schema)
    }

    pub fn get_indexes(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<Vec<Index>, ProviderError> {
        indexes::get_indexes(client, table_name, schema)
    }

    pub fn get_foreign_keys(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<Vec<ForeignKey>, ProviderError> {
        foreign_keys::get_foreign_keys(client, table_name, schema)
    }

    pub fn get_constraints(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<Vec<Constraint>, ProviderError> {
        constraints::get_constraints(client, table_name, schema)
    }

    /// Retrieves table statistics including row count and size.
    ///
    /// # Returns
    /// A tuple of `(row_count, size_in_bytes)`.
    ///
    /// # Important: Row Count is an Estimate
    /// The row count returned is an **estimate** from `pg_stat_user_tables.n_live_tup`,
    /// not an exact count from `COUNT(*)`. This provides significantly better performance
    /// for large tables but may be inaccurate if:
    /// - The table was recently created or heavily modified
    /// - `ANALYZE` has not been run recently
    /// - Autovacuum is disabled or hasn't processed the table
    ///
    /// For exact row counts, use `get_row_count` instead
    /// (with the caveat that it performs a full table scan).
    pub fn get_table_stats(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<(usize, u64), ProviderError> {
        stats::get_table_stats(client, table_name, schema)
    }

    /// Retrieves triggers defined on a table.
    pub fn get_triggers(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<Vec<Trigger>, ProviderError> {
        triggers::get_triggers(client, table_name, schema)
    }

    /// Retrieves all stored procedures and functions from the specified schema.
    pub fn get_routines(
        client: &mut Client,
        schema: &str,
    ) -> Result<Vec<Routine>, ProviderError> {
        routines::get_routines(client, schema)
    }
}
