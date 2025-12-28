//! Tests for PostgreSQL provider

use std::env;

use super::helpers::{is_valid_identifier, parse_column_sort_order, quote_identifier};
use super::{PostgresProvider, ProviderError};
use crate::db::provider::DatabaseProvider;
use crate::model::schema::SortOrder;

fn create_test_provider() -> PostgresProvider {
    let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port: u16 = env::var("POSTGRES_PORT")
        .unwrap_or_else(|_| "15432".to_string())
        .parse()
        .expect("POSTGRES_PORT must be a valid port number");
    let database = env::var("POSTGRES_DB").unwrap_or_else(|_| "lazydb_dev".to_string());
    let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "lazydb".to_string());
    let password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "lazydb".to_string());

    PostgresProvider::connect(&host, port, &database, &user, &password).expect("Failed to connect")
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
        .get_table_details("orders", Some("public"))
        .expect("Failed to get table details");

    println!("Foreign keys for orders:");
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

// ==================== Query Module Tests ====================
// Tests for internal query functions (get_columns, get_constraints)

use super::queries::InternalQueries;

#[test]
#[ignore] // Requires database connection
fn test_get_columns_returns_all_columns() {
    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let columns = InternalQueries::get_columns(&mut client, "users", "public")
        .expect("Failed to get columns");

    assert!(!columns.is_empty(), "Should return at least one column");

    // Verify column names exist
    let column_names: Vec<&str> = columns.iter().map(|c| c.name.as_str()).collect();
    assert!(column_names.contains(&"id"), "Should contain 'id' column");
}

#[test]
#[ignore] // Requires database connection
fn test_get_columns_ordinal_position() {
    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let columns = InternalQueries::get_columns(&mut client, "users", "public")
        .expect("Failed to get columns");

    // Verify ordinal positions are sequential starting from 1
    for (i, col) in columns.iter().enumerate() {
        assert_eq!(
            col.ordinal_position,
            i + 1,
            "Column '{}' should have ordinal_position {}",
            col.name,
            i + 1
        );
    }
}

#[test]
#[ignore] // Requires database connection
fn test_get_columns_primary_key_detection() {
    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let columns = InternalQueries::get_columns(&mut client, "users", "public")
        .expect("Failed to get columns");

    let id_column = columns.iter().find(|c| c.name == "id");
    assert!(id_column.is_some(), "Should find 'id' column");

    let id_column = id_column.unwrap();
    assert!(
        id_column.is_primary_key,
        "'id' column should be marked as primary key"
    );
}

#[test]
#[ignore] // Requires database connection
fn test_get_columns_nullable_detection() {
    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let columns = InternalQueries::get_columns(&mut client, "users", "public")
        .expect("Failed to get columns");

    // Primary key columns should not be nullable
    let id_column = columns.iter().find(|c| c.name == "id").unwrap();
    assert!(
        !id_column.is_nullable,
        "Primary key 'id' should not be nullable"
    );
}

#[test]
#[ignore] // Requires database connection
fn test_get_columns_auto_increment_detection() {
    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let columns = InternalQueries::get_columns(&mut client, "users", "public")
        .expect("Failed to get columns");

    // Check if 'id' column has auto-increment (serial type)
    let id_column = columns.iter().find(|c| c.name == "id").unwrap();
    // Serial columns have default value starting with "nextval("
    if id_column.default_value.is_some() {
        assert!(
            id_column.is_auto_increment,
            "'id' with nextval default should be auto_increment"
        );
    }
}

#[test]
#[ignore] // Requires database connection
fn test_get_columns_nonexistent_table() {
    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let columns = InternalQueries::get_columns(&mut client, "nonexistent_table_12345", "public")
        .expect("Query should succeed even for nonexistent table");

    assert!(
        columns.is_empty(),
        "Should return empty for nonexistent table"
    );
}

#[test]
#[ignore] // Requires database connection
fn test_get_constraints_returns_constraints() {
    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let constraints = InternalQueries::get_constraints(&mut client, "users", "public")
        .expect("Failed to get constraints");

    assert!(
        !constraints.is_empty(),
        "Should return at least one constraint (primary key)"
    );
}

#[test]
#[ignore] // Requires database connection
fn test_get_constraints_primary_key() {
    use crate::model::schema::ConstraintType;

    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let constraints = InternalQueries::get_constraints(&mut client, "users", "public")
        .expect("Failed to get constraints");

    let pk_constraint = constraints
        .iter()
        .find(|c| c.constraint_type == ConstraintType::PrimaryKey);

    assert!(
        pk_constraint.is_some(),
        "Should have a PRIMARY KEY constraint"
    );

    let pk = pk_constraint.unwrap();
    assert!(
        pk.columns.contains(&"id".to_string()),
        "Primary key should include 'id' column"
    );
}

#[test]
#[ignore] // Requires database connection
fn test_get_constraints_unique() {
    use crate::model::schema::ConstraintType;

    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let constraints = InternalQueries::get_constraints(&mut client, "users", "public")
        .expect("Failed to get constraints");

    // Check if there are any UNIQUE constraints
    let unique_constraints: Vec<_> = constraints
        .iter()
        .filter(|c| c.constraint_type == ConstraintType::Unique)
        .collect();

    // users table may have unique constraint on email or username
    for constraint in &unique_constraints {
        assert!(
            !constraint.columns.is_empty(),
            "Unique constraint '{}' should have columns",
            constraint.name
        );
    }
}

#[test]
#[ignore] // Requires database connection
fn test_get_constraints_foreign_key() {
    use crate::model::schema::ConstraintType;

    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    // orders table should have foreign key to users
    let constraints = InternalQueries::get_constraints(&mut client, "orders", "public")
        .expect("Failed to get constraints");

    let fk_constraints: Vec<_> = constraints
        .iter()
        .filter(|c| c.constraint_type == ConstraintType::ForeignKey)
        .collect();

    assert!(
        !fk_constraints.is_empty(),
        "orders table should have foreign key constraint"
    );
}

#[test]
#[ignore] // Requires database connection
fn test_get_constraints_nonexistent_table() {
    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let constraints =
        InternalQueries::get_constraints(&mut client, "nonexistent_table_12345", "public")
            .expect("Query should succeed even for nonexistent table");

    assert!(
        constraints.is_empty(),
        "Should return empty for nonexistent table"
    );
}

#[test]
#[ignore] // Requires database connection
fn test_get_constraints_columns_ordering() {
    use crate::model::schema::ConstraintType;

    let provider = create_test_provider();
    let mut client = provider.client.lock().unwrap();

    let constraints = InternalQueries::get_constraints(&mut client, "users", "public")
        .expect("Failed to get constraints");

    // Verify constraint names are sorted
    let names: Vec<&str> = constraints.iter().map(|c| c.name.as_str()).collect();
    let mut sorted_names = names.clone();
    sorted_names.sort();
    assert_eq!(names, sorted_names, "Constraints should be sorted by name");

    // Verify PRIMARY KEY constraint has non-empty columns
    if let Some(pk) = constraints
        .iter()
        .find(|c| c.constraint_type == ConstraintType::PrimaryKey)
    {
        assert!(
            !pk.columns.is_empty(),
            "Primary key constraint should have columns"
        );
    }
}
