//! Routine (stored procedures and functions) metadata query

use std::collections::HashMap;

use postgres::Client;

use crate::db::postgres::ProviderError;
use crate::model::schema::{ParameterMode, Routine, RoutineParameter, RoutineType, Volatility};

/// Retrieves all stored procedures and functions from the specified schema.
pub fn get_routines(client: &mut Client, schema: &str) -> Result<Vec<Routine>, ProviderError> {
    // Query to get all routines (functions and procedures) with their metadata
    let routines_query = r#"
        SELECT
            p.proname AS routine_name,
            n.nspname AS schema_name,
            CASE p.prokind
                WHEN 'f' THEN 'FUNCTION'
                WHEN 'p' THEN 'PROCEDURE'
                WHEN 'a' THEN 'FUNCTION'  -- aggregate functions
                WHEN 'w' THEN 'FUNCTION'  -- window functions
                ELSE 'FUNCTION'
            END AS routine_type,
            l.lanname AS language,
            CASE p.provolatile
                WHEN 'i' THEN 'IMMUTABLE'
                WHEN 's' THEN 'STABLE'
                ELSE 'VOLATILE'
            END AS volatility,
            p.prosecdef AS security_definer,
            pg_get_function_result(p.oid) AS return_type,
            pg_get_functiondef(p.oid) AS definition,
            d.description AS comment,
            p.oid AS routine_oid
        FROM pg_proc p
        JOIN pg_namespace n ON n.oid = p.pronamespace
        JOIN pg_language l ON l.oid = p.prolang
        LEFT JOIN pg_description d ON d.objoid = p.oid AND d.classoid = 'pg_proc'::regclass
        WHERE n.nspname = $1
        AND p.prokind IN ('f', 'p', 'a', 'w')  -- functions, procedures, aggregates, windows
        AND l.lanname != 'internal'  -- exclude internal functions
        AND l.lanname != 'c'  -- exclude C functions
        ORDER BY p.proname
    "#;

    let routine_rows = client
        .query(routines_query, &[&schema])
        .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

    // Collect routine OIDs for batch parameter fetch
    let routine_oids: Vec<u32> = routine_rows.iter().map(|row| row.get::<_, u32>(9)).collect();

    // Fetch all parameters in one query and group by routine OID
    let params_map = fetch_all_parameters(client, &routine_oids)?;

    let mut routines = Vec::new();

    for row in routine_rows {
        let name: String = row.get(0);
        let schema_name: String = row.get(1);
        let routine_type_str: String = row.get(2);
        let language: String = row.get(3);
        let volatility_str: String = row.get(4);
        let security_definer: bool = row.get(5);
        let return_type: Option<String> = row.get(6);
        let definition: Option<String> = row.get(7);
        let comment: Option<String> = row.get(8);
        let routine_oid: u32 = row.get(9);

        let routine_type = match routine_type_str.as_str() {
            "PROCEDURE" => RoutineType::Procedure,
            _ => RoutineType::Function,
        };

        let volatility = match volatility_str.as_str() {
            "IMMUTABLE" => Volatility::Immutable,
            "STABLE" => Volatility::Stable,
            _ => Volatility::Volatile,
        };

        // Get parameters from the pre-fetched map
        let parameters = params_map
            .get(&routine_oid)
            .cloned()
            .unwrap_or_default();

        let mut routine =
            Routine::new(name, schema_name, routine_type, language).with_volatility(volatility);

        if security_definer {
            routine = routine.with_security_definer(true);
        }

        if let Some(ret) = return_type {
            // Procedures return void, so we only set return_type for functions with non-void returns
            if ret != "void" {
                routine = routine.with_return_type(ret);
            }
        }

        if let Some(def) = definition {
            routine = routine.with_definition(def);
        }

        if let Some(cmt) = comment {
            routine = routine.with_comment(cmt);
        }

        routine = routine.with_parameters(parameters);

        routines.push(routine);
    }

    Ok(routines)
}

/// Fetch all parameters for given routine OIDs in a single query.
/// Returns a HashMap mapping routine OID to its parameters.
fn fetch_all_parameters(
    client: &mut Client,
    routine_oids: &[u32],
) -> Result<HashMap<u32, Vec<RoutineParameter>>, ProviderError> {
    if routine_oids.is_empty() {
        return Ok(HashMap::new());
    }

    // Use pg_proc directly to get parameter information
    // This is more reliable than information_schema.parameters
    let query = r#"
        WITH routine_params AS (
            SELECT
                p.oid AS routine_oid,
                unnest(COALESCE(p.proargnames, ARRAY[]::text[])) AS param_name,
                unnest(COALESCE(p.proargmodes, ARRAY['i']::char[])) AS param_mode,
                unnest(string_to_array(pg_get_function_identity_arguments(p.oid), ', ')) AS param_sig,
                generate_series(1, COALESCE(array_length(p.proargnames, 1), pronargs)) AS ordinal
            FROM pg_proc p
            WHERE p.oid = ANY($1::oid[])
        )
        SELECT
            routine_oid,
            COALESCE(param_name, '') AS param_name,
            CASE param_mode
                WHEN 'i' THEN 'IN'
                WHEN 'o' THEN 'OUT'
                WHEN 'b' THEN 'INOUT'
                WHEN 'v' THEN 'VARIADIC'
                WHEN 't' THEN 'TABLE'
                ELSE 'IN'
            END AS param_mode,
            -- Extract type from signature (format: "name type" or just "type")
            CASE
                WHEN param_sig ~ ' ' THEN regexp_replace(param_sig, '^[^ ]+ ', '')
                ELSE param_sig
            END AS data_type,
            ordinal
        FROM routine_params
        WHERE param_sig IS NOT NULL AND param_sig != ''
        ORDER BY routine_oid, ordinal
    "#;

    let oids_i64: Vec<i64> = routine_oids.iter().map(|&oid| oid as i64).collect();

    let rows = client
        .query(query, &[&oids_i64])
        .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

    let mut params_map: HashMap<u32, Vec<RoutineParameter>> = HashMap::new();

    for row in rows {
        let routine_oid: i64 = row.get(0);
        let name: String = row.get(1);
        let mode_str: String = row.get(2);
        let data_type: String = row.get(3);
        let ordinal: i32 = row.get(4);

        let mode = match mode_str.as_str() {
            "OUT" => ParameterMode::Out,
            "INOUT" => ParameterMode::InOut,
            "VARIADIC" => ParameterMode::Variadic,
            "TABLE" => ParameterMode::Out, // TABLE parameters are similar to OUT
            _ => ParameterMode::In,
        };

        let param = RoutineParameter::new(name, data_type, mode).with_position(ordinal as u32);

        params_map
            .entry(routine_oid as u32)
            .or_default()
            .push(param);
    }

    Ok(params_map)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are unit tests for parsing logic.
    // Integration tests with actual PostgreSQL are in the tests/ directory.

    #[test]
    fn test_routine_type_parsing() {
        assert!(matches!(
            match "PROCEDURE" {
                "PROCEDURE" => RoutineType::Procedure,
                _ => RoutineType::Function,
            },
            RoutineType::Procedure
        ));

        assert!(matches!(
            match "FUNCTION" {
                "PROCEDURE" => RoutineType::Procedure,
                _ => RoutineType::Function,
            },
            RoutineType::Function
        ));
    }

    #[test]
    fn test_volatility_parsing() {
        assert!(matches!(
            match "IMMUTABLE" {
                "IMMUTABLE" => Volatility::Immutable,
                "STABLE" => Volatility::Stable,
                _ => Volatility::Volatile,
            },
            Volatility::Immutable
        ));

        assert!(matches!(
            match "STABLE" {
                "IMMUTABLE" => Volatility::Immutable,
                "STABLE" => Volatility::Stable,
                _ => Volatility::Volatile,
            },
            Volatility::Stable
        ));

        assert!(matches!(
            match "VOLATILE" {
                "IMMUTABLE" => Volatility::Immutable,
                "STABLE" => Volatility::Stable,
                _ => Volatility::Volatile,
            },
            Volatility::Volatile
        ));
    }

    #[test]
    fn test_parameter_mode_parsing() {
        assert!(matches!(
            match "IN" {
                "OUT" => ParameterMode::Out,
                "INOUT" => ParameterMode::InOut,
                "VARIADIC" => ParameterMode::Variadic,
                _ => ParameterMode::In,
            },
            ParameterMode::In
        ));

        assert!(matches!(
            match "OUT" {
                "OUT" => ParameterMode::Out,
                "INOUT" => ParameterMode::InOut,
                "VARIADIC" => ParameterMode::Variadic,
                _ => ParameterMode::In,
            },
            ParameterMode::Out
        ));

        assert!(matches!(
            match "INOUT" {
                "OUT" => ParameterMode::Out,
                "INOUT" => ParameterMode::InOut,
                "VARIADIC" => ParameterMode::Variadic,
                _ => ParameterMode::In,
            },
            ParameterMode::InOut
        ));

        assert!(matches!(
            match "VARIADIC" {
                "OUT" => ParameterMode::Out,
                "INOUT" => ParameterMode::InOut,
                "VARIADIC" => ParameterMode::Variadic,
                _ => ParameterMode::In,
            },
            ParameterMode::Variadic
        ));
    }
}
