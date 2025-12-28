//! Table statistics query

use postgres::Client;

use crate::db::postgres::ProviderError;

pub fn get_table_stats(
    client: &mut Client,
    table_name: &str,
    schema: &str,
) -> Result<(usize, u64), ProviderError> {
    // Get estimated row count from pg_stat_user_tables (faster than COUNT(*))
    let count_query = r#"
        SELECT COALESCE(n_live_tup, 0)::bigint
        FROM pg_stat_user_tables
        WHERE schemaname = $1 AND relname = $2
    "#;

    let count_rows = client
        .query(count_query, &[&schema, &table_name])
        .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

    let row_count: i64 = if count_rows.is_empty() {
        0
    } else {
        count_rows[0].get(0)
    };

    // Get table size - use quote_ident to safely handle identifiers and prevent SQL injection
    let size_query = r#"
        SELECT pg_total_relation_size(
            (quote_ident($1) || '.' || quote_ident($2))::regclass
        )
    "#;

    let size_rows = client
        .query(size_query, &[&schema, &table_name])
        .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

    let size: i64 = if size_rows.is_empty() {
        0
    } else {
        size_rows[0].get(0)
    };

    Ok((row_count as usize, size as u64))
}
