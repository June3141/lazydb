//! Constraint metadata query

use postgres::Client;

use crate::db::postgres::ProviderError;
use crate::model::schema::{Constraint, ConstraintType};

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
            AND tc.table_name = kcu.table_name
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
