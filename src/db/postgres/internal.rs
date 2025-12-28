//! Internal helper methods for PostgreSQL operations

use postgres::Client;

use crate::model::schema::{
    Column, Constraint, ConstraintType, ForeignKey, Index, IndexColumn, IndexMethod, IndexType,
    Trigger, TriggerEvent, TriggerOrientation, TriggerTiming,
};

use super::helpers::{parse_column_sort_order, parse_fk_action};
use super::ProviderError;

/// Internal helper methods for fetching table metadata
pub struct InternalQueries;

impl InternalQueries {
    pub fn get_columns(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<Vec<Column>, ProviderError> {
        let query = r#"
            SELECT
                c.column_name,
                c.data_type,
                c.is_nullable,
                c.column_default,
                c.ordinal_position,
                col_description((quote_ident(c.table_schema) || '.' || quote_ident(c.table_name))::regclass, c.ordinal_position) as comment,
                CASE WHEN pk.column_name IS NOT NULL THEN true ELSE false END as is_primary_key,
                CASE WHEN uq.column_name IS NOT NULL THEN true ELSE false END as is_unique
            FROM information_schema.columns c
            LEFT JOIN (
                SELECT ku.column_name
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage ku
                    ON tc.constraint_name = ku.constraint_name
                    AND tc.table_schema = ku.table_schema
                WHERE tc.constraint_type = 'PRIMARY KEY'
                AND tc.table_schema = $1
                AND tc.table_name = $2
            ) pk ON c.column_name = pk.column_name
            LEFT JOIN (
                SELECT ku.column_name
                FROM information_schema.table_constraints tc
                JOIN information_schema.key_column_usage ku
                    ON tc.constraint_name = ku.constraint_name
                    AND tc.table_schema = ku.table_schema
                WHERE tc.constraint_type = 'UNIQUE'
                AND tc.table_schema = $1
                AND tc.table_name = $2
            ) uq ON c.column_name = uq.column_name
            WHERE c.table_schema = $1 AND c.table_name = $2
            ORDER BY c.ordinal_position
        "#;

        let rows = client
            .query(query, &[&schema, &table_name])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let columns = rows
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let data_type: String = row.get(1);
                let is_nullable_str: String = row.get(2);
                let default_value: Option<String> = row.get(3);
                let ordinal_position: i32 = row.get(4);
                let comment: Option<String> = row.get(5);
                let is_primary_key: bool = row.get(6);
                let is_unique: bool = row.get(7);

                let is_auto_increment = default_value
                    .as_ref()
                    .map(|d| d.starts_with("nextval("))
                    .unwrap_or(false);

                Column {
                    name,
                    data_type,
                    is_nullable: is_nullable_str == "YES",
                    default_value,
                    is_primary_key,
                    is_unique,
                    is_auto_increment,
                    comment,
                    ordinal_position: ordinal_position as usize,
                }
            })
            .collect();

        Ok(columns)
    }

    pub fn get_indexes(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<Vec<Index>, ProviderError> {
        let query = r#"
            SELECT
                i.relname as index_name,
                am.amname as index_method,
                ix.indisunique as is_unique,
                ix.indisprimary as is_primary,
                array_agg(a.attname ORDER BY array_position(ix.indkey, a.attnum)) as column_names,
                pg_get_indexdef(ix.indexrelid) as index_def
            FROM pg_index ix
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_class t ON t.oid = ix.indrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_am am ON am.oid = i.relam
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            WHERE n.nspname = $1
            AND t.relname = $2
            GROUP BY i.relname, am.amname, ix.indisunique, ix.indisprimary, ix.indexrelid
            ORDER BY i.relname
        "#;

        let rows = client
            .query(query, &[&schema, &table_name])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let indexes = rows
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let method_str: String = row.get(1);
                let is_unique: bool = row.get(2);
                let is_primary: bool = row.get(3);
                let column_names: Vec<String> = row.get(4);
                let index_def: String = row.get(5);

                let index_type = if is_primary {
                    IndexType::Primary
                } else if is_unique {
                    IndexType::Unique
                } else {
                    IndexType::Index
                };

                let method = match method_str.as_str() {
                    "btree" => IndexMethod::BTree,
                    "hash" => IndexMethod::Hash,
                    "gist" => IndexMethod::Gist,
                    "gin" => IndexMethod::Gin,
                    "brin" => IndexMethod::Brin,
                    other => IndexMethod::Other(other.to_string()),
                };

                let columns: Vec<IndexColumn> = column_names
                    .into_iter()
                    .map(|col_name| {
                        // Parse sort order from index definition
                        // Example index_def: "CREATE INDEX idx ON schema.table USING btree (col1, col2 DESC, col3 ASC)"
                        let order = parse_column_sort_order(&index_def, &col_name);
                        IndexColumn {
                            name: col_name,
                            order,
                        }
                    })
                    .collect();

                Index {
                    name,
                    index_type,
                    method,
                    columns,
                    is_unique,
                    comment: None,
                }
            })
            .collect();

        Ok(indexes)
    }

    pub fn get_foreign_keys(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<Vec<ForeignKey>, ProviderError> {
        let query = r#"
            SELECT
                tc.constraint_name,
                kcu.column_name,
                ccu.table_name AS foreign_table_name,
                ccu.column_name AS foreign_column_name,
                rc.update_rule,
                rc.delete_rule
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
            JOIN information_schema.referential_constraints AS rc
                ON rc.constraint_name = tc.constraint_name
                AND rc.constraint_schema = tc.table_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_schema = $1
            AND tc.table_name = $2
            ORDER BY tc.constraint_name, kcu.ordinal_position
        "#;

        let rows = client
            .query(query, &[&schema, &table_name])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        // Group by constraint name (using BTreeMap for deterministic ordering)
        let mut fk_map: std::collections::BTreeMap<String, ForeignKey> =
            std::collections::BTreeMap::new();

        for row in rows {
            let constraint_name: String = row.get(0);
            let column_name: String = row.get(1);
            let foreign_table: String = row.get(2);
            let foreign_column: String = row.get(3);
            let update_rule: String = row.get(4);
            let delete_rule: String = row.get(5);

            let entry = fk_map
                .entry(constraint_name.clone())
                .or_insert_with(|| ForeignKey {
                    name: constraint_name,
                    columns: Vec::new(),
                    referenced_table: foreign_table,
                    referenced_columns: Vec::new(),
                    on_update: parse_fk_action(&update_rule),
                    on_delete: parse_fk_action(&delete_rule),
                });

            if !entry.columns.contains(&column_name) {
                entry.columns.push(column_name);
            }
            if !entry.referenced_columns.contains(&foreign_column) {
                entry.referenced_columns.push(foreign_column);
            }
        }

        Ok(fk_map.into_values().collect())
    }

    pub fn get_constraints(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<Vec<Constraint>, ProviderError> {
        let query = r#"
            SELECT
                tc.constraint_name,
                tc.constraint_type,
                COALESCE(
                    array_remove(
                        array_agg(kcu.column_name ORDER BY kcu.ordinal_position),
                        NULL
                    ),
                    ARRAY[]::varchar[]
                ) as columns,
                cc.check_clause
            FROM information_schema.table_constraints tc
            LEFT JOIN information_schema.key_column_usage kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            LEFT JOIN information_schema.check_constraints cc
                ON tc.constraint_name = cc.constraint_name
                AND tc.constraint_schema = cc.constraint_schema
            WHERE tc.table_schema = $1
            AND tc.table_name = $2
            GROUP BY tc.constraint_name, tc.constraint_type, cc.check_clause
            ORDER BY tc.constraint_name
        "#;

        let rows = client
            .query(query, &[&schema, &table_name])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let constraints = rows
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let constraint_type_str: String = row.get(1);
                // Handle potential NULL array or deserialization issues
                let columns: Vec<String> = row.try_get::<_, Vec<String>>(2).unwrap_or_default();
                let check_clause: Option<String> = row.get(3);

                let constraint_type = match constraint_type_str.as_str() {
                    "PRIMARY KEY" => ConstraintType::PrimaryKey,
                    "UNIQUE" => ConstraintType::Unique,
                    "FOREIGN KEY" => ConstraintType::ForeignKey,
                    "CHECK" => ConstraintType::Check,
                    _ => ConstraintType::Check,
                };

                Constraint {
                    name,
                    constraint_type,
                    columns,
                    definition: check_clause,
                }
            })
            .collect();

        Ok(constraints)
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

    /// Retrieves triggers defined on a table.
    pub fn get_triggers(
        client: &mut Client,
        table_name: &str,
        schema: &str,
    ) -> Result<Vec<Trigger>, ProviderError> {
        let query = r#"
            SELECT
                t.tgname AS trigger_name,
                CASE
                    WHEN t.tgtype & 2 = 2 THEN 'BEFORE'
                    WHEN t.tgtype & 64 = 64 THEN 'INSTEAD OF'
                    ELSE 'AFTER'
                END AS timing,
                CASE WHEN t.tgtype & 4 = 4 THEN true ELSE false END AS is_insert,
                CASE WHEN t.tgtype & 8 = 8 THEN true ELSE false END AS is_delete,
                CASE WHEN t.tgtype & 16 = 16 THEN true ELSE false END AS is_update,
                CASE WHEN t.tgtype & 32 = 32 THEN true ELSE false END AS is_truncate,
                CASE WHEN t.tgtype & 1 = 1 THEN 'ROW' ELSE 'STATEMENT' END AS orientation,
                p.proname AS function_name,
                pg_get_triggerdef(t.oid) AS definition,
                t.tgenabled != 'D' AS is_enabled
            FROM pg_trigger t
            JOIN pg_class c ON c.oid = t.tgrelid
            JOIN pg_namespace n ON n.oid = c.relnamespace
            JOIN pg_proc p ON p.oid = t.tgfoid
            WHERE n.nspname = $1
            AND c.relname = $2
            AND NOT t.tgisinternal
            ORDER BY t.tgname
        "#;

        let rows = client
            .query(query, &[&schema, &table_name])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let triggers = rows
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let timing_str: String = row.get(1);
                let is_insert: bool = row.get(2);
                let is_delete: bool = row.get(3);
                let is_update: bool = row.get(4);
                let is_truncate: bool = row.get(5);
                let orientation_str: String = row.get(6);
                let function_name: String = row.get(7);
                let definition: String = row.get(8);
                let is_enabled: bool = row.get(9);

                let timing = match timing_str.as_str() {
                    "BEFORE" => TriggerTiming::Before,
                    "INSTEAD OF" => TriggerTiming::InsteadOf,
                    _ => TriggerTiming::After,
                };

                let mut events = Vec::new();
                if is_insert {
                    events.push(TriggerEvent::Insert);
                }
                if is_update {
                    events.push(TriggerEvent::Update);
                }
                if is_delete {
                    events.push(TriggerEvent::Delete);
                }
                if is_truncate {
                    events.push(TriggerEvent::Truncate);
                }

                let orientation = match orientation_str.as_str() {
                    "ROW" => TriggerOrientation::Row,
                    _ => TriggerOrientation::Statement,
                };

                Trigger {
                    name,
                    timing,
                    events,
                    orientation,
                    function_name,
                    definition: Some(definition),
                    enabled: is_enabled,
                }
            })
            .collect();

        Ok(triggers)
    }
}
