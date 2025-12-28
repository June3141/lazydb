//! DatabaseProvider trait implementation for PostgreSQL

use std::time::Instant;

use crate::model::schema::{Table, TableType};
use crate::model::QueryResult;

use super::helpers::{convert_value_to_string, is_valid_identifier, quote_identifier};
use super::internal::InternalQueries;
use super::{DatabaseProvider, DatabaseType, PostgresProvider, ProviderError};

impl DatabaseProvider for PostgresProvider {
    fn database_type(&self) -> DatabaseType {
        DatabaseType::PostgreSQL
    }

    fn get_schemas(&self) -> Result<Vec<String>, ProviderError> {
        let query = r#"
            SELECT schema_name
            FROM information_schema.schemata
            WHERE schema_name NOT IN ('pg_catalog', 'information_schema', 'pg_toast')
            ORDER BY schema_name
        "#;

        let mut client = self.client.lock().map_err(|e| {
            ProviderError::InternalError(format!("Failed to acquire client lock: {}", e))
        })?;

        let rows = client
            .query(query, &[])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get::<_, String>(0)).collect())
    }

    fn get_tables(&self, schema: Option<&str>) -> Result<Vec<Table>, ProviderError> {
        let schema = schema.unwrap_or("public");

        // Query to get tables with estimated row count and size
        let query = r#"
            SELECT
                t.table_name,
                t.table_type,
                obj_description((quote_ident(t.table_schema) || '.' || quote_ident(t.table_name))::regclass, 'pg_class') as comment,
                COALESCE(s.n_live_tup, 0)::bigint as row_count,
                COALESCE(pg_total_relation_size((quote_ident(t.table_schema) || '.' || quote_ident(t.table_name))::regclass), 0)::bigint as size_bytes
            FROM information_schema.tables t
            LEFT JOIN pg_stat_user_tables s
                ON s.schemaname = t.table_schema AND s.relname = t.table_name
            WHERE t.table_schema = $1
            AND t.table_type IN ('BASE TABLE', 'VIEW')
            ORDER BY t.table_name
        "#;

        let mut client = self.client.lock().map_err(|e| {
            ProviderError::InternalError(format!("Failed to acquire client lock: {}", e))
        })?;

        let rows = client
            .query(query, &[&schema])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let tables = rows
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let table_type_str: String = row.get(1);
                let comment: Option<String> = row.get(2);
                let row_count: i64 = row.get(3);
                let size_bytes: i64 = row.get(4);

                let table_type = match table_type_str.as_str() {
                    "VIEW" => TableType::View,
                    _ => TableType::BaseTable,
                };

                let mut table = Table::new(&name)
                    .with_schema(schema)
                    .with_stats(row_count as usize, size_bytes as u64);
                table.table_type = table_type;
                table.comment = comment;
                table
            })
            .collect();

        Ok(tables)
    }

    fn get_table_details(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<Table, ProviderError> {
        let schema_str = schema.unwrap_or("public");

        // Get basic table info
        let table_query = r#"
            SELECT
                t.table_type,
                obj_description((quote_ident(t.table_schema) || '.' || quote_ident(t.table_name))::regclass, 'pg_class') as comment
            FROM information_schema.tables t
            WHERE t.table_schema = $1 AND t.table_name = $2
        "#;

        let mut client = self.client.lock().map_err(|e| {
            ProviderError::InternalError(format!("Failed to acquire client lock: {}", e))
        })?;

        let table_rows = client
            .query(table_query, &[&schema_str, &table_name])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        if table_rows.is_empty() {
            return Err(ProviderError::NotFound(format!(
                "Table {}.{} not found. Verify the table name and schema, and ensure you have the necessary permissions.",
                schema_str, table_name
            )));
        }

        let table_type_str: String = table_rows[0].get(0);
        let comment: Option<String> = table_rows[0].get(1);

        let table_type = match table_type_str.as_str() {
            "VIEW" => TableType::View,
            _ => TableType::BaseTable,
        };

        // Get columns
        let columns = InternalQueries::get_columns(&mut client, table_name, schema_str)?;

        // Get indexes
        let indexes = InternalQueries::get_indexes(&mut client, table_name, schema_str)?;

        // Get foreign keys
        let foreign_keys = InternalQueries::get_foreign_keys(&mut client, table_name, schema_str)?;

        // Get constraints
        let constraints = InternalQueries::get_constraints(&mut client, table_name, schema_str)?;

        // Get triggers
        let triggers = InternalQueries::get_triggers(&mut client, table_name, schema_str)?;

        // Get row count and size
        let (row_count, size_bytes) =
            InternalQueries::get_table_stats(&mut client, table_name, schema_str)?;

        let mut table = Table::new(table_name)
            .with_schema(schema_str)
            .with_columns(columns)
            .with_indexes(indexes)
            .with_foreign_keys(foreign_keys)
            .with_constraints(constraints)
            .with_triggers(triggers)
            .with_stats(row_count, size_bytes);

        table.table_type = table_type;
        table.comment = comment;

        Ok(table)
    }

    fn execute_query(&self, query: &str) -> Result<QueryResult, ProviderError> {
        let start = Instant::now();

        let mut client = self.client.lock().map_err(|e| {
            ProviderError::InternalError(format!("Failed to acquire client lock: {}", e))
        })?;

        let rows = client
            .query(query, &[])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                execution_time_ms,
                total_rows: 0,
            });
        }

        // Get column names and types once, not per row
        let col_info: Vec<(String, &postgres::types::Type)> = rows[0]
            .columns()
            .iter()
            .map(|c| (c.name().to_string(), c.type_()))
            .collect();

        let columns: Vec<String> = col_info.iter().map(|(name, _)| name.clone()).collect();

        // Convert rows to strings using pre-fetched type information
        let result_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                col_info
                    .iter()
                    .enumerate()
                    .map(|(i, (_, col_type))| convert_value_to_string(row, i, col_type))
                    .collect()
            })
            .collect();

        let total_rows = result_rows.len();
        Ok(QueryResult {
            columns,
            rows: result_rows,
            execution_time_ms,
            total_rows,
        })
    }

    fn get_row_count(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<usize, ProviderError> {
        let schema = schema.unwrap_or("public");

        // Validate identifiers to prevent SQL injection
        // Only allow alphanumeric characters, underscores, and dollar signs (PostgreSQL identifier rules)
        if !is_valid_identifier(schema) || !is_valid_identifier(table_name) {
            return Err(ProviderError::InvalidConfiguration(
                "Invalid schema or table name".to_string(),
            ));
        }

        let query = format!(
            "SELECT COUNT(*) FROM {}.{}",
            quote_identifier(schema),
            quote_identifier(table_name)
        );

        let mut client = self.client.lock().map_err(|e| {
            ProviderError::InternalError(format!("Failed to acquire client lock: {}", e))
        })?;

        let rows = client
            .query(&query, &[])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let count: i64 = rows[0].get(0);
        Ok(count as usize)
    }

    fn get_table_size(&self, table_name: &str, schema: Option<&str>) -> Result<u64, ProviderError> {
        let schema = schema.unwrap_or("public");

        // Use quote_ident to safely handle identifiers and prevent SQL injection
        let query = r#"
            SELECT pg_total_relation_size(
                (quote_ident($1) || '.' || quote_ident($2))::regclass
            )
        "#;

        let mut client = self.client.lock().map_err(|e| {
            ProviderError::InternalError(format!("Failed to acquire client lock: {}", e))
        })?;

        let rows = client
            .query(query, &[&schema, &table_name])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        if rows.is_empty() {
            return Err(ProviderError::QueryFailed(format!(
                "No rows returned when querying size for table '{}.{}'",
                schema, table_name
            )));
        }
        let size: i64 = rows[0].get(0);
        Ok(size as u64)
    }

    fn test_connection(&self) -> Result<(), ProviderError> {
        let mut client = self.client.lock().map_err(|e| {
            ProviderError::InternalError(format!("Failed to acquire client lock: {}", e))
        })?;

        client
            .query("SELECT 1", &[])
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;
        Ok(())
    }

    fn get_version(&self) -> Result<String, ProviderError> {
        let mut client = self.client.lock().map_err(|e| {
            ProviderError::InternalError(format!("Failed to acquire client lock: {}", e))
        })?;

        let rows = client
            .query("SELECT version()", &[])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        if rows.is_empty() {
            return Err(ProviderError::QueryFailed(
                "No rows returned from SELECT version()".to_string(),
            ));
        }
        let version: String = rows[0].get(0);
        Ok(version)
    }
}
