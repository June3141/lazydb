//! Foreign key metadata query

use std::collections::BTreeMap;

use postgres::Client;

use crate::db::postgres::helpers::parse_fk_action;
use crate::db::postgres::ProviderError;
use crate::model::schema::ForeignKey;

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
    let mut fk_map: BTreeMap<String, ForeignKey> = BTreeMap::new();

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
