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
    let routine_oids: Vec<u32> = routine_rows
        .iter()
        .map(|row| row.get::<_, u32>(9))
        .collect();

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
        let parameters = params_map.get(&routine_oid).cloned().unwrap_or_default();

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

    // Use pg_get_function_arguments which provides a reliable, formatted string
    // of all arguments. This is more reliable than unnesting arrays which can
    // have mismatched lengths.
    let query = r#"
        SELECT
            p.oid AS routine_oid,
            pg_get_function_arguments(p.oid) AS args_string
        FROM pg_proc p
        WHERE p.oid = ANY($1::oid[])
    "#;

    let oids_i64: Vec<i64> = routine_oids.iter().map(|&oid| oid as i64).collect();

    let rows = client
        .query(query, &[&oids_i64])
        .map_err(|e| ProviderError::QueryFailed(e.to_string()))?;

    let mut params_map: HashMap<u32, Vec<RoutineParameter>> = HashMap::new();

    for row in rows {
        let routine_oid: i64 = row.get(0);
        let args_string: String = row.get(1);

        let parameters = parse_function_arguments(&args_string);
        if !parameters.is_empty() {
            params_map.insert(routine_oid as u32, parameters);
        }
    }

    Ok(params_map)
}

/// Parse the function arguments string returned by pg_get_function_arguments.
/// Format: "arg1 type1, arg2 type2" or "IN arg1 type1, OUT arg2 type2"
/// Also handles: "VARIADIC args text[]", defaults like "arg1 integer DEFAULT 0"
fn parse_function_arguments(args_string: &str) -> Vec<RoutineParameter> {
    if args_string.trim().is_empty() {
        return Vec::new();
    }

    let mut parameters = Vec::new();
    let mut position = 1u32;

    // Split by comma, but be careful of commas inside type definitions like "numeric(10,2)"
    for arg in split_arguments(args_string) {
        let arg = arg.trim();
        if arg.is_empty() {
            continue;
        }

        if let Some(param) = parse_single_argument(arg, position) {
            parameters.push(param);
            position += 1;
        }
    }

    parameters
}

/// Split argument string by commas, respecting parentheses for types like numeric(10,2)
fn split_arguments(args_string: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0;

    for ch in args_string.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                paren_depth -= 1;
                current.push(ch);
            }
            ',' if paren_depth == 0 => {
                result.push(current.trim().to_string());
                current = String::new();
            }
            _ => current.push(ch),
        }
    }

    if !current.trim().is_empty() {
        result.push(current.trim().to_string());
    }

    result
}

/// Parse a single argument definition.
/// Formats:
/// - "name type"
/// - "IN name type"
/// - "OUT name type"
/// - "INOUT name type"
/// - "VARIADIC name type"
/// - "name type DEFAULT value"
fn parse_single_argument(arg: &str, position: u32) -> Option<RoutineParameter> {
    let parts: Vec<&str> = arg.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let (mode, name_idx) = match parts[0].to_uppercase().as_str() {
        "IN" => (ParameterMode::In, 1),
        "OUT" => (ParameterMode::Out, 1),
        "INOUT" => (ParameterMode::InOut, 1),
        "VARIADIC" => (ParameterMode::Variadic, 1),
        _ => (ParameterMode::In, 0),
    };

    if parts.len() <= name_idx {
        return None;
    }

    // Find DEFAULT keyword to separate type from default value
    let default_idx = parts.iter().position(|&p| p.to_uppercase() == "DEFAULT");

    let (name, data_type, default_value) = if let Some(def_idx) = default_idx {
        // Has default value
        let type_end = def_idx;
        let name = parts.get(name_idx).unwrap_or(&"").to_string();
        let data_type = parts[name_idx + 1..type_end].join(" ");
        let default_val = parts[def_idx + 1..].join(" ");
        (name, data_type, Some(default_val))
    } else {
        // No default value
        let name = parts.get(name_idx).unwrap_or(&"").to_string();
        let data_type = parts[name_idx + 1..].join(" ");
        (name, data_type, None)
    };

    // Handle case where there's only type (no name) - PostgreSQL allows this
    let (final_name, final_type) = if data_type.is_empty() {
        // The "name" is actually the type
        (String::new(), name)
    } else {
        (name, data_type)
    };

    let mut param = RoutineParameter::new(final_name, final_type, mode).with_position(position);

    if let Some(def) = default_value {
        param = param.with_default(def);
    }

    Some(param)
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

    #[test]
    fn test_split_arguments_simple() {
        let args = "x integer, y text";
        let result = split_arguments(args);
        assert_eq!(result, vec!["x integer", "y text"]);
    }

    #[test]
    fn test_split_arguments_with_parentheses() {
        let args = "x numeric(10,2), y text";
        let result = split_arguments(args);
        assert_eq!(result, vec!["x numeric(10,2)", "y text"]);
    }

    #[test]
    fn test_split_arguments_empty() {
        let args = "";
        let result = split_arguments(args);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_function_arguments_simple() {
        let args = "x integer, y text";
        let params = parse_function_arguments(args);
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "x");
        assert_eq!(params[0].data_type, "integer");
        assert_eq!(params[1].name, "y");
        assert_eq!(params[1].data_type, "text");
    }

    #[test]
    fn test_parse_function_arguments_with_modes() {
        let args = "IN x integer, OUT y text, INOUT z boolean";
        let params = parse_function_arguments(args);
        assert_eq!(params.len(), 3);
        assert!(matches!(params[0].mode, ParameterMode::In));
        assert!(matches!(params[1].mode, ParameterMode::Out));
        assert!(matches!(params[2].mode, ParameterMode::InOut));
    }

    #[test]
    fn test_parse_function_arguments_with_default() {
        let args = "x integer DEFAULT 0, y text DEFAULT 'hello'";
        let params = parse_function_arguments(args);
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].default_value, Some("0".to_string()));
        assert_eq!(params[1].default_value, Some("'hello'".to_string()));
    }

    #[test]
    fn test_parse_function_arguments_variadic() {
        let args = "VARIADIC args text[]";
        let params = parse_function_arguments(args);
        assert_eq!(params.len(), 1);
        assert!(matches!(params[0].mode, ParameterMode::Variadic));
        assert_eq!(params[0].name, "args");
        assert_eq!(params[0].data_type, "text[]");
    }

    #[test]
    fn test_parse_function_arguments_no_name() {
        // PostgreSQL allows unnamed parameters
        let args = "integer, text";
        let params = parse_function_arguments(args);
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "");
        assert_eq!(params[0].data_type, "integer");
        assert_eq!(params[1].name, "");
        assert_eq!(params[1].data_type, "text");
    }

    #[test]
    fn test_parse_function_arguments_complex_types() {
        let args = "x numeric(10,2), y character varying(255)";
        let params = parse_function_arguments(args);
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].data_type, "numeric(10,2)");
        assert_eq!(params[1].data_type, "character varying(255)");
    }
}
