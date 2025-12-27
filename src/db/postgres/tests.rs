//! Tests for PostgreSQL provider

use super::helpers::{is_valid_identifier, parse_column_sort_order, quote_identifier};
use super::{PostgresProvider, ProviderError};
use crate::db::provider::DatabaseProvider;
use crate::model::schema::SortOrder;

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
fn test_is_valid_identifier_unit() {
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
fn test_quote_identifier_unit() {
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
fn test_parse_column_sort_order_unit() {
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
