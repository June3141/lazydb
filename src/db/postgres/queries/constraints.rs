//! Constraint metadata query

use postgres::Client;

use crate::db::postgres::ProviderError;
use crate::model::schema::{Constraint, ConstraintType};

pub fn get_constraints(
    client: &mut Client,
    table_name: &str,
    schema: &str,
) -> Result<Vec<Constraint>, ProviderError> {
    // Use PostgreSQL system catalogs for more reliable constraint info
    let query = r#"
        SELECT
            con.conname AS constraint_name,
            con.contype AS constraint_type,
            COALESCE(
                (
                    SELECT array_agg(a.attname ORDER BY array_position(con.conkey, a.attnum))
                    FROM unnest(con.conkey) AS k(num)
                    JOIN pg_attribute a ON a.attrelid = con.conrelid AND a.attnum = k.num
                ),
                ARRAY[]::varchar[]
            ) AS columns,
            pg_get_constraintdef(con.oid) AS definition
        FROM pg_constraint con
        JOIN pg_class rel ON rel.oid = con.conrelid
        JOIN pg_namespace nsp ON nsp.oid = rel.relnamespace
        WHERE nsp.nspname = $1
        AND rel.relname = $2
        ORDER BY con.conname
    "#;

    let rows = client
        .query(query, &[&schema, &table_name])
        .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

    let constraints = rows
        .iter()
        .map(|row| {
            let name: String = row.get(0);
            // pg_constraint.contype is a single character:
            // p = primary key, u = unique, f = foreign key, c = check, x = exclusion
            let constraint_type_char: i8 = row.get(1);
            let columns: Vec<String> = row.try_get::<_, Vec<String>>(2).unwrap_or_default();
            let definition: Option<String> = row.get(3);

            let constraint_type = match constraint_type_char as u8 as char {
                'p' => ConstraintType::PrimaryKey,
                'u' => ConstraintType::Unique,
                'f' => ConstraintType::ForeignKey,
                'c' => ConstraintType::Check,
                _ => ConstraintType::Check,
            };

            Constraint {
                name,
                constraint_type,
                columns,
                definition,
            }
        })
        .collect();

    Ok(constraints)
}
