//! Trigger metadata query

use postgres::Client;

use crate::db::postgres::ProviderError;
use crate::model::schema::{Trigger, TriggerEvent, TriggerOrientation, TriggerTiming};

pub fn get_triggers(
    client: &mut Client,
    table_name: &str,
    schema: &str,
) -> Result<Vec<Trigger>, ProviderError> {
    let query = r#"
        SELECT
            t.tgname AS trigger_name,
            CASE
                WHEN t.tgtype & 2 = 2 THEN 'BEFORE'
                WHEN t.tgtype & 64 = 64 THEN 'INSTEAD OF'
                ELSE 'AFTER'
            END AS timing,
            CASE WHEN t.tgtype & 4 = 4 THEN true ELSE false END AS is_insert,
            CASE WHEN t.tgtype & 8 = 8 THEN true ELSE false END AS is_delete,
            CASE WHEN t.tgtype & 16 = 16 THEN true ELSE false END AS is_update,
            CASE WHEN t.tgtype & 32 = 32 THEN true ELSE false END AS is_truncate,
            CASE WHEN t.tgtype & 1 = 1 THEN 'ROW' ELSE 'STATEMENT' END AS orientation,
            p.proname AS function_name,
            pg_get_triggerdef(t.oid) AS definition,
            t.tgenabled != 'D' AS is_enabled
        FROM pg_trigger t
        JOIN pg_class c ON c.oid = t.tgrelid
        JOIN pg_namespace n ON n.oid = c.relnamespace
        JOIN pg_proc p ON p.oid = t.tgfoid
        WHERE n.nspname = $1
        AND c.relname = $2
        AND NOT t.tgisinternal
        ORDER BY t.tgname
    "#;

    let rows = client
        .query(query, &[&schema, &table_name])
        .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

    let triggers = rows
        .iter()
        .map(|row| {
            let name: String = row.get(0);
            let timing_str: String = row.get(1);
            let is_insert: bool = row.get(2);
            let is_delete: bool = row.get(3);
            let is_update: bool = row.get(4);
            let is_truncate: bool = row.get(5);
            let orientation_str: String = row.get(6);
            let function_name: String = row.get(7);
            let definition: String = row.get(8);
            let is_enabled: bool = row.get(9);

            let timing = match timing_str.as_str() {
                "BEFORE" => TriggerTiming::Before,
                "INSTEAD OF" => TriggerTiming::InsteadOf,
                _ => TriggerTiming::After,
            };

            let mut events = Vec::new();
            if is_insert {
                events.push(TriggerEvent::Insert);
            }
            if is_update {
                events.push(TriggerEvent::Update);
            }
            if is_delete {
                events.push(TriggerEvent::Delete);
            }
            if is_truncate {
                events.push(TriggerEvent::Truncate);
            }

            let orientation = match orientation_str.as_str() {
                "ROW" => TriggerOrientation::Row,
                _ => TriggerOrientation::Statement,
            };

            Trigger {
                name,
                timing,
                events,
                orientation,
                function_name,
                definition: Some(definition),
                enabled: is_enabled,
            }
        })
        .collect();

    Ok(triggers)
}
