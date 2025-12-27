//! Utility functions for PostgreSQL operations

use crate::model::schema::{ForeignKeyAction, SortOrder};

/// Parses a foreign key action string into the corresponding enum variant.
pub fn parse_fk_action(action: &str) -> ForeignKeyAction {
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
pub fn quote_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

/// Validates that an identifier only contains safe characters for PostgreSQL identifiers.
/// Prevents SQL injection by rejecting identifiers with potentially dangerous characters.
pub fn is_valid_identifier(identifier: &str) -> bool {
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
pub fn parse_column_sort_order(index_def: &str, column_name: &str) -> SortOrder {
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
///
/// # Warning: NUMERIC Precision Loss
///
/// PostgreSQL `NUMERIC` values are converted to `f64`, which **will lose precision**
/// for high-precision decimal values. PostgreSQL `NUMERIC` can store up to 131,072
/// digits before the decimal point and 16,383 after, while `f64` has only ~15-17
/// significant digits.
///
/// **Do not rely on this function when exact `NUMERIC` values are required**
/// (e.g., financial calculations, scientific computations, or cryptographic operations).
/// For such use cases, consider:
/// - Using the `rust_decimal` crate with `BigDecimal` type
/// - Fetching `NUMERIC` columns as `TEXT` and parsing in higher-level code
/// - Using a specialized decimal library appropriate for your precision requirements
pub fn convert_value_to_string(
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
        // WARNING: NUMERIC is converted to f64 here, which can lose precision for
        // high-precision decimal values. PostgreSQL NUMERIC can have up to 131072 digits
        // before the decimal point and 16383 after, while f64 has only ~15-17 significant
        // digits. Do not rely on this helper when exact NUMERIC values are required
        // (for example, in financial or scientific calculations); instead, fetch these
        // columns using a decimal/BigDecimal type or as text in higher-level code.
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
