//! PostgreSQL database provider implementation

use postgres::{Client, NoTls};
use std::sync::Mutex;
use std::time::Instant;

use crate::config::ConnectionConfig;
use crate::model::schema::{
    Column, Constraint, ConstraintType, ForeignKey, ForeignKeyAction, Index, IndexColumn,
    IndexMethod, IndexType, SortOrder, Table, TableType,
};
use crate::model::QueryResult;

use super::provider::{DatabaseProvider, DatabaseType, ProviderError};

/// PostgreSQL database provider
pub struct PostgresProvider {
    client: Mutex<Client>,
}

impl PostgresProvider {
    /// Create a new PostgresProvider from connection configuration
    pub fn new(config: &ConnectionConfig) -> Result<Self, ProviderError> {
        let password = config.get_password().unwrap_or_default();
        let username = config.username.as_deref().unwrap_or("postgres");

        let connection_string = format!(
            "host={} port={} dbname={} user={} password={}",
            config.host, config.port, config.database, username, password
        );

        let client = Client::connect(&connection_string, NoTls)
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            client: Mutex::new(client),
        })
    }

    /// Create a new PostgresProvider from connection parameters.
    ///
    /// Uses the `postgres::Config` builder API to safely handle passwords
    /// containing special characters (like `@`, `#`, spaces, or quotes).
    pub fn connect(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, ProviderError> {
        let mut config = postgres::Config::new();
        config
            .host(host)
            .port(port)
            .dbname(database)
            .user(username)
            .password(password);

        let client = config
            .connect(NoTls)
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            client: Mutex::new(client),
        })
    }
}

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
        let columns = Self::get_columns_internal(&mut client, table_name, schema_str)?;

        // Get indexes
        let indexes = Self::get_indexes_internal(&mut client, table_name, schema_str)?;

        // Get foreign keys
        let foreign_keys = Self::get_foreign_keys_internal(&mut client, table_name, schema_str)?;

        // Get constraints
        let constraints = Self::get_constraints_internal(&mut client, table_name, schema_str)?;

        // Get row count and size
        let (row_count, size_bytes) = Self::get_table_stats(&mut client, table_name, schema_str)?;

        let mut table = Table::new(table_name)
            .with_schema(schema_str)
            .with_columns(columns)
            .with_indexes(indexes)
            .with_foreign_keys(foreign_keys)
            .with_constraints(constraints)
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

// Private helper methods
impl PostgresProvider {
    fn get_columns_internal(
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

    fn get_indexes_internal(
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

    fn get_foreign_keys_internal(
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

    fn get_constraints_internal(
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
    /// For exact row counts, use [`get_row_count`](Self::get_row_count) instead
    /// (with the caveat that it performs a full table scan).
    fn get_table_stats(
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
}

fn parse_fk_action(action: &str) -> ForeignKeyAction {
    match action {
        "CASCADE" => ForeignKeyAction::Cascade,
        "SET NULL" => ForeignKeyAction::SetNull,
        "SET DEFAULT" => ForeignKeyAction::SetDefault,
        "RESTRICT" => ForeignKeyAction::Restrict,
        _ => ForeignKeyAction::NoAction,
    }
}

/// Quotes a PostgreSQL identifier by wrapping it in double quotes and escaping internal quotes.
///
/// # Security Warning
/// This function provides basic identifier quoting but is **not sufficient for untrusted input**.
/// It only escapes double quotes by doubling them, and does not:
/// - Validate identifier length (PostgreSQL has a 63-byte limit)
/// - Check for reserved keywords
/// - Validate character encoding
///
/// For user-provided identifiers, always use [`is_valid_identifier`] to validate first,
/// or use PostgreSQL's built-in `quote_ident()` function via parameterized queries.
///
/// # Example
/// ```ignore
/// assert_eq!(quote_identifier("my_table"), "\"my_table\"");
/// assert_eq!(quote_identifier("table\"name"), "\"table\"\"name\"");
/// ```
fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

/// Validates that an identifier only contains safe characters for PostgreSQL identifiers.
/// Prevents SQL injection by rejecting identifiers with potentially dangerous characters.
fn is_valid_identifier(identifier: &str) -> bool {
    if identifier.is_empty() || identifier.len() > 63 {
        return false;
    }

    // PostgreSQL identifiers must start with a letter or underscore
    let first_char = identifier.chars().next().unwrap();
    if !first_char.is_ascii_alphabetic() && first_char != '_' {
        return false;
    }

    // Remaining characters can be letters, digits, underscores, or dollar signs
    identifier
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Parses the sort order for a column from a PostgreSQL index definition.
///
/// The index definition from `pg_get_indexdef()` has the format:
/// `CREATE INDEX idx_name ON schema.table USING btree (col1, col2 DESC, col3 ASC)`
///
/// This function looks for the column name followed by optional sort order keywords.
/// If DESC is found after the column name, returns `SortOrder::Desc`.
/// Otherwise, returns `SortOrder::Asc` (PostgreSQL default).
fn parse_column_sort_order(index_def: &str, column_name: &str) -> SortOrder {
    // Find the columns part within parentheses
    let columns_part = match (index_def.rfind('('), index_def.rfind(')')) {
        (Some(start), Some(end)) if start < end => &index_def[start + 1..end],
        _ => return SortOrder::Asc,
    };

    // Split by comma and find the column
    for col_spec in columns_part.split(',') {
        let col_spec = col_spec.trim();
        if col_spec.is_empty() {
            continue;
        }

        // Parse the column specification, handling quoted identifiers
        // Column spec could be: "col_name", "col_name DESC", '"Column Name" DESC', etc.
        let (spec_col_name, remainder) = if let Some(stripped) = col_spec.strip_prefix('"') {
            // Quoted identifier - find the closing quote
            if let Some(end_quote) = stripped.find('"') {
                let col_name = &stripped[..end_quote];
                let rest = stripped[end_quote + 1..].trim();
                (col_name, rest)
            } else {
                // Malformed, skip
                continue;
            }
        } else {
            // Unquoted identifier - take until whitespace
            let parts: Vec<&str> = col_spec.splitn(2, char::is_whitespace).collect();
            let col_name = parts[0];
            let rest = parts.get(1).map(|s| s.trim()).unwrap_or("");
            (col_name, rest)
        };

        if spec_col_name == column_name {
            // Check if DESC appears in the remainder
            let upper = remainder.to_uppercase();
            if upper.contains("DESC") {
                return SortOrder::Desc;
            }
            return SortOrder::Asc;
        }
    }

    // Column not found in definition, default to Asc
    SortOrder::Asc
}

/// Converts a PostgreSQL row value to a string based on the column type.
///
/// Uses the pre-fetched column type information to efficiently convert values
/// without trial-and-error type checking on each row.
fn convert_value_to_string(
    row: &postgres::Row,
    index: usize,
    col_type: &postgres::types::Type,
) -> String {
    use postgres::types::Type;

    // Match on PostgreSQL type and use the appropriate Rust type for extraction
    match *col_type {
        Type::BOOL => row
            .try_get::<_, Option<bool>>(index)
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        Type::INT2 => row
            .try_get::<_, Option<i16>>(index)
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        Type::INT4 => row
            .try_get::<_, Option<i32>>(index)
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        Type::INT8 => row
            .try_get::<_, Option<i64>>(index)
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        Type::FLOAT4 => row
            .try_get::<_, Option<f32>>(index)
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        // Note: NUMERIC is converted to f64 which may lose precision for high-precision
        // decimal values. PostgreSQL NUMERIC can have up to 131072 digits before the decimal
        // and 16383 after, while f64 has only ~15-17 significant digits.
        Type::FLOAT8 | Type::NUMERIC => row
            .try_get::<_, Option<f64>>(index)
            .ok()
            .flatten()
            .map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string()),
        Type::TEXT | Type::VARCHAR | Type::CHAR | Type::BPCHAR | Type::NAME => row
            .try_get::<_, Option<String>>(index)
            .ok()
            .flatten()
            .unwrap_or_else(|| "NULL".to_string()),
        _ => {
            // Fallback: try common types in order of likelihood
            // If all attempts fail, assume NULL (or unsupported type)
            if let Ok(Some(v)) = row.try_get::<_, Option<String>>(index) {
                v
            } else if let Ok(Some(v)) = row.try_get::<_, Option<i64>>(index) {
                v.to_string()
            } else if let Ok(Some(v)) = row.try_get::<_, Option<f64>>(index) {
                v.to_string()
            } else if let Ok(Some(v)) = row.try_get::<_, Option<bool>>(index) {
                v.to_string()
            } else {
                "NULL".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_provider() -> PostgresProvider {
        PostgresProvider::connect("localhost", 5432, "lazydb_dev", "lazydb", "lazydb")
            .expect("Failed to connect")
    }

    #[test]
    #[ignore] // Run manually with: cargo test --package lazydb -- --ignored
    fn test_postgres_connection() {
        let provider = create_test_provider();

        provider.test_connection().expect("Connection test failed");

        let version = provider.get_version().expect("Failed to get version");
        println!("PostgreSQL version: {}", version);

        let schemas = provider.get_schemas().expect("Failed to get schemas");
        println!("Schemas: {:?}", schemas);

        let tables = provider
            .get_tables(Some("public"))
            .expect("Failed to get tables");
        println!(
            "Tables: {:?}",
            tables.iter().map(|t| &t.name).collect::<Vec<_>>()
        );
    }

    #[test]
    #[ignore]
    fn test_parameterized_query() {
        let provider = create_test_provider();

        // Test parameterized query
        let mut client = provider.client.lock().unwrap();
        let schema = "public".to_string();
        let table_name = "users".to_string();

        println!("Testing simple parameterized query...");
        let result = client
            .query(
                "SELECT $1::text as schema_name, $2::text as table_name",
                &[&schema, &table_name],
            )
            .expect("Parameterized query failed");
        println!("Simple query OK: {:?}", result[0].get::<_, String>(0));

        println!("Testing table info query...");
        let table_query = r#"
            SELECT
                t.table_type,
                obj_description((quote_ident(t.table_schema) || '.' || quote_ident(t.table_name))::regclass, 'pg_class') as comment
            FROM information_schema.tables t
            WHERE t.table_schema = $1 AND t.table_name = $2
        "#;
        let table_rows = client
            .query(table_query, &[&schema, &table_name])
            .expect("Table info query failed");
        println!("Table info query OK: {} rows", table_rows.len());

        println!("Testing columns query...");
        let columns_query = r#"
            SELECT
                c.column_name,
                c.data_type,
                c.is_nullable
            FROM information_schema.columns c
            WHERE c.table_schema = $1 AND c.table_name = $2
            ORDER BY c.ordinal_position
        "#;
        let column_rows = client
            .query(columns_query, &[&schema, &table_name])
            .expect("Columns query failed");
        println!("Columns query OK: {} rows", column_rows.len());
    }

    #[test]
    #[ignore]
    fn test_get_table_details() {
        let provider = create_test_provider();

        let table = provider
            .get_table_details("users", Some("public"))
            .expect("Failed to get table details");

        println!("Table: {} ({})", table.name, table.table_type);
        println!("Comment: {:?}", table.comment);
        println!("Columns:");
        for col in &table.columns {
            println!(
                "  - {} ({}) PK={} NULL={} DEFAULT={:?}",
                col.name, col.data_type, col.is_primary_key, col.is_nullable, col.default_value
            );
        }
        println!("Indexes:");
        for idx in &table.indexes {
            println!(
                "  - {} ({:?}) columns: {:?}",
                idx.name,
                idx.index_type,
                idx.columns.iter().map(|c| &c.name).collect::<Vec<_>>()
            );
        }
        println!("Constraints:");
        for con in &table.constraints {
            println!("  - {} ({:?})", con.name, con.constraint_type);
        }

        assert!(!table.columns.is_empty());
        assert!(table
            .columns
            .iter()
            .any(|c| c.name == "id" && c.is_primary_key));
    }

    #[test]
    #[ignore]
    fn test_get_foreign_keys() {
        let provider = create_test_provider();

        let table = provider
            .get_table_details("posts", Some("public"))
            .expect("Failed to get table details");

        println!("Foreign keys for posts:");
        for fk in &table.foreign_keys {
            println!(
                "  - {} ({:?} -> {} ({:?}))",
                fk.name, fk.columns, fk.referenced_table, fk.referenced_columns
            );
            println!(
                "    ON UPDATE: {:?}, ON DELETE: {:?}",
                fk.on_update, fk.on_delete
            );
        }

        assert!(!table.foreign_keys.is_empty());
    }

    #[test]
    #[ignore]
    fn test_execute_query() {
        let provider = create_test_provider();

        let result = provider
            .execute_query("SELECT id, username, email FROM users ORDER BY id")
            .expect("Failed to execute query");

        println!("Query result:");
        println!("Columns: {:?}", result.columns);
        println!("Execution time: {}ms", result.execution_time_ms);
        for row in &result.rows {
            println!("  {:?}", row);
        }

        assert_eq!(result.columns, vec!["id", "username", "email"]);
        assert!(!result.rows.is_empty());
    }

    #[test]
    #[ignore]
    fn test_get_row_count() {
        let provider = create_test_provider();

        let count = provider
            .get_row_count("users", Some("public"))
            .expect("Failed to get row count");

        println!("Row count for users: {}", count);
        assert!(count > 0);
    }

    // ==================== Error Case Tests ====================
    // These tests verify proper error handling for various failure scenarios

    #[test]
    fn test_connection_failure_invalid_host() {
        let result = PostgresProvider::connect(
            "nonexistent.invalid.host.example.com",
            5432,
            "testdb",
            "user",
            "pass",
        );

        assert!(result.is_err());
        match result {
            Err(ProviderError::ConnectionFailed(msg)) => {
                println!("Expected connection failure: {}", msg);
            }
            Err(other) => panic!("Expected ConnectionFailed, got: {:?}", other),
            Ok(_) => panic!("Expected connection to fail"),
        }
    }

    #[test]
    fn test_connection_failure_invalid_port() {
        // Port 1 is unlikely to have a PostgreSQL server
        let result = PostgresProvider::connect("localhost", 1, "testdb", "user", "pass");

        assert!(result.is_err());
        match result {
            Err(ProviderError::ConnectionFailed(msg)) => {
                println!("Expected connection failure: {}", msg);
            }
            Err(other) => panic!("Expected ConnectionFailed, got: {:?}", other),
            Ok(_) => panic!("Expected connection to fail"),
        }
    }

    #[test]
    fn test_is_valid_identifier() {
        // Valid identifiers
        assert!(is_valid_identifier("users"));
        assert!(is_valid_identifier("my_table"));
        assert!(is_valid_identifier("_private"));
        assert!(is_valid_identifier("Table123"));
        assert!(is_valid_identifier("user$data"));

        // Invalid identifiers - empty or too long
        assert!(!is_valid_identifier(""));
        assert!(!is_valid_identifier(&"a".repeat(64))); // > 63 chars

        // Invalid identifiers - starts with invalid character
        assert!(!is_valid_identifier("123table")); // starts with digit
        assert!(!is_valid_identifier("$money")); // starts with $
        assert!(!is_valid_identifier("-dashed")); // starts with dash

        // Invalid identifiers - SQL injection attempts
        assert!(!is_valid_identifier("users; DROP TABLE users;--"));
        assert!(!is_valid_identifier("users' OR '1'='1"));
        assert!(!is_valid_identifier("table\nname"));
        assert!(!is_valid_identifier("schema.table"));
        assert!(!is_valid_identifier("table("));
        assert!(!is_valid_identifier("table)"));
    }

    #[test]
    fn test_quote_identifier() {
        assert_eq!(quote_identifier("users"), "\"users\"");
        assert_eq!(quote_identifier("my_table"), "\"my_table\"");
        assert_eq!(quote_identifier("Table Name"), "\"Table Name\"");
        // Double quotes are escaped by doubling them
        assert_eq!(quote_identifier("table\"name"), "\"table\"\"name\"");
    }

    #[test]
    #[ignore] // Requires database connection
    fn test_invalid_table_name_get_row_count() {
        let provider = create_test_provider();

        // Test with SQL injection attempt
        let result = provider.get_row_count("users; DROP TABLE users;--", Some("public"));
        assert!(result.is_err());
        match result {
            Err(ProviderError::InvalidConfiguration(msg)) => {
                assert!(msg.contains("Invalid"));
                println!("Correctly rejected invalid table name: {}", msg);
            }
            Err(other) => panic!("Expected InvalidConfiguration, got: {:?}", other),
            Ok(_) => panic!("Should have rejected invalid table name"),
        }

        // Test with invalid schema
        let result = provider.get_row_count("users", Some("public' OR '1'='1"));
        assert!(result.is_err());
        match result {
            Err(ProviderError::InvalidConfiguration(_)) => {
                println!("Correctly rejected invalid schema name");
            }
            Err(other) => panic!("Expected InvalidConfiguration, got: {:?}", other),
            Ok(_) => panic!("Should have rejected invalid schema name"),
        }
    }

    #[test]
    #[ignore] // Requires database connection
    fn test_nonexistent_table() {
        let provider = create_test_provider();

        let result = provider.get_table_details("this_table_does_not_exist_12345", Some("public"));
        assert!(result.is_err());
        match result {
            Err(ProviderError::NotFound(msg)) => {
                println!("Correctly reported not found: {}", msg);
            }
            Err(other) => panic!("Expected NotFound, got: {:?}", other),
            Ok(_) => panic!("Should have returned NotFound for nonexistent table"),
        }
    }

    #[test]
    #[ignore] // Requires database connection
    fn test_nonexistent_schema() {
        let provider = create_test_provider();

        let result = provider.get_tables(Some("nonexistent_schema_12345"));
        // This should succeed but return empty
        match result {
            Ok(tables) => {
                assert!(
                    tables.is_empty(),
                    "Should return empty for nonexistent schema"
                );
            }
            Err(e) => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn test_parse_column_sort_order() {
        // Test ASC (default)
        let index_def = "CREATE INDEX idx ON schema.table USING btree (col1)";
        assert_eq!(parse_column_sort_order(index_def, "col1"), SortOrder::Asc);

        // Test explicit ASC
        let index_def = "CREATE INDEX idx ON schema.table USING btree (col1 ASC)";
        assert_eq!(parse_column_sort_order(index_def, "col1"), SortOrder::Asc);

        // Test DESC
        let index_def = "CREATE INDEX idx ON schema.table USING btree (col1 DESC)";
        assert_eq!(parse_column_sort_order(index_def, "col1"), SortOrder::Desc);

        // Test multiple columns with mixed order
        let index_def = "CREATE INDEX idx ON schema.table USING btree (col1, col2 DESC, col3 ASC)";
        assert_eq!(parse_column_sort_order(index_def, "col1"), SortOrder::Asc);
        assert_eq!(parse_column_sort_order(index_def, "col2"), SortOrder::Desc);
        assert_eq!(parse_column_sort_order(index_def, "col3"), SortOrder::Asc);

        // Test quoted identifiers
        let index_def = "CREATE INDEX idx ON schema.table USING btree (\"Column Name\" DESC)";
        assert_eq!(
            parse_column_sort_order(index_def, "Column Name"),
            SortOrder::Desc
        );

        // Test column not in index (fallback to Asc)
        let index_def = "CREATE INDEX idx ON schema.table USING btree (col1)";
        assert_eq!(
            parse_column_sort_order(index_def, "nonexistent"),
            SortOrder::Asc
        );

        // Test malformed index definition (no parentheses)
        let index_def = "CREATE INDEX idx ON schema.table";
        assert_eq!(parse_column_sort_order(index_def, "col1"), SortOrder::Asc);
    }
}
