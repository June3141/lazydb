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

    /// Create a new PostgresProvider from connection parameters
    pub fn connect(
        host: &str,
        port: u16,
        database: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, ProviderError> {
        let connection_string = format!(
            "host={} port={} dbname={} user={} password={}",
            host, port, database, username, password
        );

        let client = Client::connect(&connection_string, NoTls)
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

        let mut client = self
            .client
            .lock()
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        let rows = client
            .query(query, &[])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        Ok(rows.iter().map(|row| row.get::<_, String>(0)).collect())
    }

    fn get_tables(&self, schema: Option<&str>) -> Result<Vec<Table>, ProviderError> {
        let schema = schema.unwrap_or("public");

        let query = r#"
            SELECT
                t.table_name,
                t.table_type,
                obj_description((quote_ident(t.table_schema) || '.' || quote_ident(t.table_name))::regclass, 'pg_class') as comment
            FROM information_schema.tables t
            WHERE t.table_schema = $1
            AND t.table_type IN ('BASE TABLE', 'VIEW')
            ORDER BY t.table_name
        "#;

        let mut client = self
            .client
            .lock()
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        let rows = client
            .query(query, &[&schema])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let tables = rows
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let table_type_str: String = row.get(1);
                let comment: Option<String> = row.get(2);

                let table_type = match table_type_str.as_str() {
                    "VIEW" => TableType::View,
                    _ => TableType::BaseTable,
                };

                let mut table = Table::new(&name).with_schema(schema);
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

        let mut client = self
            .client
            .lock()
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        // Create owned Strings for proper serialization
        let schema_owned = schema_str.to_string();
        let table_owned = table_name.to_string();

        let table_rows = client
            .query(table_query, &[&schema_owned, &table_owned])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        if table_rows.is_empty() {
            return Err(ProviderError::NotFound(format!(
                "Table {}.{} not found",
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

        let mut client = self
            .client
            .lock()
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        let rows = client
            .query(query, &[])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let execution_time_ms = start.elapsed().as_millis() as u64;

        if rows.is_empty() {
            return Ok(QueryResult {
                columns: Vec::new(),
                rows: Vec::new(),
                execution_time_ms,
            });
        }

        // Get column names
        let columns: Vec<String> = rows[0]
            .columns()
            .iter()
            .map(|c| c.name().to_string())
            .collect();

        // Convert rows to strings
        let result_rows: Vec<Vec<String>> = rows
            .iter()
            .map(|row| {
                (0..row.len())
                    .map(|i| {
                        // Try to get as various types and convert to string
                        if let Ok(v) = row.try_get::<_, Option<String>>(i) {
                            v.unwrap_or_else(|| "NULL".to_string())
                        } else if let Ok(v) = row.try_get::<_, Option<i32>>(i) {
                            v.map(|n| n.to_string())
                                .unwrap_or_else(|| "NULL".to_string())
                        } else if let Ok(v) = row.try_get::<_, Option<i64>>(i) {
                            v.map(|n| n.to_string())
                                .unwrap_or_else(|| "NULL".to_string())
                        } else if let Ok(v) = row.try_get::<_, Option<f64>>(i) {
                            v.map(|n| n.to_string())
                                .unwrap_or_else(|| "NULL".to_string())
                        } else if let Ok(v) = row.try_get::<_, Option<bool>>(i) {
                            v.map(|b| b.to_string())
                                .unwrap_or_else(|| "NULL".to_string())
                        } else {
                            "<unknown>".to_string()
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(QueryResult {
            columns,
            rows: result_rows,
            execution_time_ms,
        })
    }

    fn get_row_count(
        &self,
        table_name: &str,
        schema: Option<&str>,
    ) -> Result<usize, ProviderError> {
        let schema = schema.unwrap_or("public");
        let query = format!(
            "SELECT COUNT(*) FROM {}.{}",
            quote_identifier(schema),
            quote_identifier(table_name)
        );

        let mut client = self
            .client
            .lock()
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        let rows = client
            .query(&query, &[])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let count: i64 = rows[0].get(0);
        Ok(count as usize)
    }

    fn get_table_size(&self, table_name: &str, schema: Option<&str>) -> Result<u64, ProviderError> {
        let schema = schema.unwrap_or("public");

        // Use format string directly since regclass needs special handling
        let query = format!(
            "SELECT pg_total_relation_size('{}.{}'::regclass)",
            schema.replace('\'', "''"),
            table_name.replace('\'', "''")
        );

        let mut client = self
            .client
            .lock()
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        let rows = client
            .query(&query, &[])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let size: i64 = rows[0].get(0);
        Ok(size as u64)
    }

    fn test_connection(&self) -> Result<(), ProviderError> {
        let mut client = self
            .client
            .lock()
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        client
            .query("SELECT 1", &[])
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;
        Ok(())
    }

    fn get_version(&self) -> Result<String, ProviderError> {
        let mut client = self
            .client
            .lock()
            .map_err(|e| ProviderError::ConnectionFailed(e.to_string()))?;

        let rows = client
            .query("SELECT version()", &[])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

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

        // Explicitly create owned Strings to ensure proper serialization
        let schema_owned = schema.to_string();
        let table_owned = table_name.to_string();

        let rows = client
            .query(query, &[&schema_owned, &table_owned])
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

        // Explicitly create owned Strings to ensure proper serialization
        let schema_owned = schema.to_string();
        let table_owned = table_name.to_string();

        let rows = client
            .query(query, &[&schema_owned, &table_owned])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let indexes = rows
            .iter()
            .map(|row| {
                let name: String = row.get(0);
                let method_str: String = row.get(1);
                let is_unique: bool = row.get(2);
                let is_primary: bool = row.get(3);
                let column_names: Vec<String> = row.get(4);

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
                    .map(|name| IndexColumn {
                        name,
                        order: SortOrder::Asc, // Default, could be parsed from index_def
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

        // Explicitly create owned Strings to ensure proper serialization
        let schema_owned = schema.to_string();
        let table_owned = table_name.to_string();

        let rows = client
            .query(query, &[&schema_owned, &table_owned])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        // Group by constraint name
        let mut fk_map: std::collections::HashMap<String, ForeignKey> =
            std::collections::HashMap::new();

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

        // Explicitly create owned Strings to ensure proper serialization
        let schema_owned = schema.to_string();
        let table_owned = table_name.to_string();

        let rows = client
            .query(query, &[&schema_owned, &table_owned])
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

        // Explicitly create owned Strings to ensure proper serialization
        let schema_owned = schema.to_string();
        let table_owned = table_name.to_string();

        let count_rows = client
            .query(count_query, &[&schema_owned, &table_owned])
            .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

        let row_count: i64 = if count_rows.is_empty() {
            0
        } else {
            count_rows[0].get(0)
        };

        // Get table size - use format string directly since regclass needs special handling
        let size_query = format!(
            "SELECT pg_total_relation_size('{}.{}'::regclass)",
            schema.replace('\'', "''"),
            table_name.replace('\'', "''")
        );

        let size_rows = client
            .query(&size_query, &[])
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

fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
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
}
