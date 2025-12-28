//! Column metadata query

use postgres::Client;

use crate::db::postgres::ProviderError;
use crate::model::schema::Column;

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
