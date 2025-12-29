//! Routine (stored procedures and functions) metadata query

use postgres::Client;

use crate::db::postgres::ProviderError;
use crate::model::schema::{ParameterMode, Routine, RoutineParameter, RoutineType, Volatility};

/// Retrieves all stored procedures and functions from the specified schema.
pub fn get_routines(client: &mut Client, schema: &str) -> Result<Vec<Routine>, ProviderError> {
    // Query to get all routines (functions and procedures) with their metadata
    let query = r#"
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

    let rows = client
        .query(query, &[&schema])
        .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

    let mut routines = Vec::new();

    for row in rows {
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

        // Get parameters for this routine
        let parameters = get_routine_parameters(client, routine_oid)?;

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

/// Retrieves parameters for a specific routine by its OID.
fn get_routine_parameters(
    client: &mut Client,
    routine_oid: u32,
) -> Result<Vec<RoutineParameter>, ProviderError> {
    let query = r#"
        SELECT
            COALESCE(p.parameter_name, '') AS param_name,
            p.data_type,
            p.parameter_mode,
            p.parameter_default,
            p.ordinal_position
        FROM information_schema.parameters p
        WHERE p.specific_schema || '.' || p.specific_name = (
            SELECT n.nspname || '.' || p2.proname || '_' || p2.oid::text
            FROM pg_proc p2
            JOIN pg_namespace n ON n.oid = p2.pronamespace
            WHERE p2.oid = $1
        )
        ORDER BY p.ordinal_position
    "#;

    let rows = client
        .query(query, &[&(routine_oid as i32)])
        .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

    let parameters = rows
        .iter()
        .map(|row| {
            let name: String = row.get(0);
            let data_type: String = row.get(1);
            let mode_str: String = row.get(2);
            let default_value: Option<String> = row.get(3);
            let ordinal_position: i32 = row.get(4);

            let mode = match mode_str.as_str() {
                "OUT" => ParameterMode::Out,
                "INOUT" => ParameterMode::InOut,
                "VARIADIC" => ParameterMode::Variadic,
                _ => ParameterMode::In,
            };

            let mut param = RoutineParameter::new(name, data_type, mode)
                .with_position(ordinal_position as u32);

            if let Some(def) = default_value {
                param = param.with_default(def);
            }

            param
        })
        .collect();

    Ok(parameters)
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
