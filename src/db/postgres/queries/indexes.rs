//! Index metadata query

use postgres::Client;

use crate::db::postgres::helpers::parse_column_sort_order;
use crate::db::postgres::ProviderError;
use crate::model::schema::{Index, IndexColumn, IndexMethod, IndexType};

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
