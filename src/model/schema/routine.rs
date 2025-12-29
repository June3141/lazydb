//! Routine types and structures (stored procedures and functions)

/// Routine type (procedure or function)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutineType {
    Procedure,
    Function,
}

impl std::fmt::Display for RoutineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutineType::Procedure => write!(f, "PROCEDURE"),
            RoutineType::Function => write!(f, "FUNCTION"),
        }
    }
}

/// Parameter mode (IN, OUT, INOUT, VARIADIC)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParameterMode {
    In,
    Out,
    InOut,
    Variadic,
}

impl std::fmt::Display for ParameterMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParameterMode::In => write!(f, "IN"),
            ParameterMode::Out => write!(f, "OUT"),
            ParameterMode::InOut => write!(f, "INOUT"),
            ParameterMode::Variadic => write!(f, "VARIADIC"),
        }
    }
}

/// Routine parameter information
#[derive(Debug, Clone, PartialEq)]
pub struct RoutineParameter {
    /// Parameter name (may be empty for unnamed parameters)
    pub name: String,
    /// Parameter data type
    pub data_type: String,
    /// Parameter mode
    pub mode: ParameterMode,
    /// Default value (if any)
    pub default_value: Option<String>,
    /// Ordinal position (1-based)
    pub ordinal_position: u32,
}

impl RoutineParameter {
    /// Create a new parameter with required fields
    pub fn new(name: impl Into<String>, data_type: impl Into<String>, mode: ParameterMode) -> Self {
        Self {
            name: name.into(),
            data_type: data_type.into(),
            mode,
            default_value: None,
            ordinal_position: 0,
        }
    }

    /// Set the default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default_value = Some(default.into());
        self
    }

    /// Set the ordinal position
    pub fn with_position(mut self, position: u32) -> Self {
        self.ordinal_position = position;
        self
    }
}

/// Routine volatility category
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Volatility {
    Immutable,
    Stable,
    Volatile,
}

impl std::fmt::Display for Volatility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Volatility::Immutable => write!(f, "IMMUTABLE"),
            Volatility::Stable => write!(f, "STABLE"),
            Volatility::Volatile => write!(f, "VOLATILE"),
        }
    }
}

/// Database routine (stored procedure or function)
#[derive(Debug, Clone, PartialEq)]
pub struct Routine {
    /// Routine name
    pub name: String,
    /// Schema name
    pub schema: String,
    /// Routine type (procedure or function)
    pub routine_type: RoutineType,
    /// Parameters
    pub parameters: Vec<RoutineParameter>,
    /// Return type (for functions; None for procedures)
    pub return_type: Option<String>,
    /// Language (e.g., plpgsql, sql, python)
    pub language: String,
    /// Volatility category
    pub volatility: Volatility,
    /// Whether the routine is security definer
    pub security_definer: bool,
    /// Full routine definition (CREATE FUNCTION/PROCEDURE statement)
    pub definition: Option<String>,
    /// Description/comment
    pub comment: Option<String>,
}

impl Routine {
    /// Create a new routine with required fields
    pub fn new(
        name: impl Into<String>,
        schema: impl Into<String>,
        routine_type: RoutineType,
        language: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            schema: schema.into(),
            routine_type,
            parameters: Vec::new(),
            return_type: None,
            language: language.into(),
            volatility: Volatility::Volatile,
            security_definer: false,
            definition: None,
            comment: None,
        }
    }

    /// Add a parameter
    pub fn with_parameter(mut self, param: RoutineParameter) -> Self {
        self.parameters.push(param);
        self
    }

    /// Set multiple parameters
    pub fn with_parameters(mut self, params: Vec<RoutineParameter>) -> Self {
        self.parameters = params;
        self
    }

    /// Set the return type
    pub fn with_return_type(mut self, return_type: impl Into<String>) -> Self {
        self.return_type = Some(return_type.into());
        self
    }

    /// Set volatility
    pub fn with_volatility(mut self, volatility: Volatility) -> Self {
        self.volatility = volatility;
        self
    }

    /// Set security definer
    pub fn with_security_definer(mut self, security_definer: bool) -> Self {
        self.security_definer = security_definer;
        self
    }

    /// Set the definition
    pub fn with_definition(mut self, definition: impl Into<String>) -> Self {
        self.definition = Some(definition.into());
        self
    }

    /// Set the comment
    pub fn with_comment(mut self, comment: impl Into<String>) -> Self {
        self.comment = Some(comment.into());
        self
    }

    /// Get display name with schema (e.g., "public.my_function")
    pub fn qualified_name(&self) -> String {
        format!("{}.{}", self.schema, self.name)
    }

    /// Get parameter signature string (e.g., "(id integer, name text)")
    pub fn parameters_signature(&self) -> String {
        if self.parameters.is_empty() {
            return "()".to_string();
        }

        let params: Vec<String> = self
            .parameters
            .iter()
            .filter(|p| p.mode != ParameterMode::Out)
            .map(|p| {
                if p.name.is_empty() {
                    p.data_type.clone()
                } else {
                    format!("{} {}", p.name, p.data_type)
                }
            })
            .collect();

        format!("({})", params.join(", "))
    }

    /// Check if this is a function
    pub fn is_function(&self) -> bool {
        self.routine_type == RoutineType::Function
    }

    /// Check if this is a procedure
    pub fn is_procedure(&self) -> bool {
        self.routine_type == RoutineType::Procedure
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== RoutineType tests =====

    #[test]
    fn test_routine_type_display_procedure() {
        assert_eq!(RoutineType::Procedure.to_string(), "PROCEDURE");
    }

    #[test]
    fn test_routine_type_display_function() {
        assert_eq!(RoutineType::Function.to_string(), "FUNCTION");
    }

    // ===== ParameterMode tests =====

    #[test]
    fn test_parameter_mode_display_in() {
        assert_eq!(ParameterMode::In.to_string(), "IN");
    }

    #[test]
    fn test_parameter_mode_display_out() {
        assert_eq!(ParameterMode::Out.to_string(), "OUT");
    }

    #[test]
    fn test_parameter_mode_display_inout() {
        assert_eq!(ParameterMode::InOut.to_string(), "INOUT");
    }

    #[test]
    fn test_parameter_mode_display_variadic() {
        assert_eq!(ParameterMode::Variadic.to_string(), "VARIADIC");
    }

    // ===== RoutineParameter tests =====

    #[test]
    fn test_routine_parameter_new() {
        let param = RoutineParameter::new("user_id", "integer", ParameterMode::In);

        assert_eq!(param.name, "user_id");
        assert_eq!(param.data_type, "integer");
        assert_eq!(param.mode, ParameterMode::In);
        assert!(param.default_value.is_none());
        assert_eq!(param.ordinal_position, 0);
    }

    #[test]
    fn test_routine_parameter_with_default() {
        let param = RoutineParameter::new("limit", "integer", ParameterMode::In).with_default("10");

        assert_eq!(param.default_value, Some("10".to_string()));
    }

    #[test]
    fn test_routine_parameter_with_position() {
        let param =
            RoutineParameter::new("user_id", "integer", ParameterMode::In).with_position(1);

        assert_eq!(param.ordinal_position, 1);
    }

    // ===== Volatility tests =====

    #[test]
    fn test_volatility_display_immutable() {
        assert_eq!(Volatility::Immutable.to_string(), "IMMUTABLE");
    }

    #[test]
    fn test_volatility_display_stable() {
        assert_eq!(Volatility::Stable.to_string(), "STABLE");
    }

    #[test]
    fn test_volatility_display_volatile() {
        assert_eq!(Volatility::Volatile.to_string(), "VOLATILE");
    }

    // ===== Routine struct tests =====

    #[test]
    fn test_routine_new_function() {
        let routine = Routine::new("get_user", "public", RoutineType::Function, "plpgsql");

        assert_eq!(routine.name, "get_user");
        assert_eq!(routine.schema, "public");
        assert_eq!(routine.routine_type, RoutineType::Function);
        assert_eq!(routine.language, "plpgsql");
        assert!(routine.parameters.is_empty());
        assert!(routine.return_type.is_none());
        assert_eq!(routine.volatility, Volatility::Volatile);
        assert!(!routine.security_definer);
        assert!(routine.definition.is_none());
        assert!(routine.comment.is_none());
    }

    #[test]
    fn test_routine_new_procedure() {
        let routine = Routine::new("update_user", "public", RoutineType::Procedure, "plpgsql");

        assert_eq!(routine.routine_type, RoutineType::Procedure);
    }

    #[test]
    fn test_routine_with_parameter() {
        let param = RoutineParameter::new("user_id", "integer", ParameterMode::In);
        let routine = Routine::new("get_user", "public", RoutineType::Function, "plpgsql")
            .with_parameter(param);

        assert_eq!(routine.parameters.len(), 1);
        assert_eq!(routine.parameters[0].name, "user_id");
    }

    #[test]
    fn test_routine_with_multiple_parameters() {
        let params = vec![
            RoutineParameter::new("user_id", "integer", ParameterMode::In),
            RoutineParameter::new("user_name", "text", ParameterMode::In),
        ];
        let routine = Routine::new("get_user", "public", RoutineType::Function, "plpgsql")
            .with_parameters(params);

        assert_eq!(routine.parameters.len(), 2);
    }

    #[test]
    fn test_routine_with_return_type() {
        let routine = Routine::new("get_user", "public", RoutineType::Function, "plpgsql")
            .with_return_type("SETOF users");

        assert_eq!(routine.return_type, Some("SETOF users".to_string()));
    }

    #[test]
    fn test_routine_with_volatility() {
        let routine = Routine::new("get_user", "public", RoutineType::Function, "plpgsql")
            .with_volatility(Volatility::Stable);

        assert_eq!(routine.volatility, Volatility::Stable);
    }

    #[test]
    fn test_routine_with_security_definer() {
        let routine = Routine::new("get_user", "public", RoutineType::Function, "plpgsql")
            .with_security_definer(true);

        assert!(routine.security_definer);
    }

    #[test]
    fn test_routine_with_definition() {
        let definition =
            "CREATE FUNCTION get_user(id integer) RETURNS users AS $$ SELECT * FROM users WHERE id = $1 $$ LANGUAGE sql";
        let routine = Routine::new("get_user", "public", RoutineType::Function, "sql")
            .with_definition(definition);

        assert_eq!(routine.definition, Some(definition.to_string()));
    }

    #[test]
    fn test_routine_with_comment() {
        let routine = Routine::new("get_user", "public", RoutineType::Function, "plpgsql")
            .with_comment("Retrieves a user by ID");

        assert_eq!(
            routine.comment,
            Some("Retrieves a user by ID".to_string())
        );
    }

    #[test]
    fn test_routine_qualified_name() {
        let routine = Routine::new("get_user", "public", RoutineType::Function, "plpgsql");

        assert_eq!(routine.qualified_name(), "public.get_user");
    }

    #[test]
    fn test_routine_parameters_signature_empty() {
        let routine = Routine::new("get_all_users", "public", RoutineType::Function, "sql");

        assert_eq!(routine.parameters_signature(), "()");
    }

    #[test]
    fn test_routine_parameters_signature_single() {
        let routine = Routine::new("get_user", "public", RoutineType::Function, "sql")
            .with_parameter(RoutineParameter::new("user_id", "integer", ParameterMode::In));

        assert_eq!(routine.parameters_signature(), "(user_id integer)");
    }

    #[test]
    fn test_routine_parameters_signature_multiple() {
        let routine = Routine::new("get_users", "public", RoutineType::Function, "sql")
            .with_parameter(RoutineParameter::new("offset_val", "integer", ParameterMode::In))
            .with_parameter(RoutineParameter::new("limit_val", "integer", ParameterMode::In));

        assert_eq!(
            routine.parameters_signature(),
            "(offset_val integer, limit_val integer)"
        );
    }

    #[test]
    fn test_routine_parameters_signature_excludes_out_params() {
        let routine = Routine::new("get_user_with_count", "public", RoutineType::Function, "sql")
            .with_parameter(RoutineParameter::new("user_id", "integer", ParameterMode::In))
            .with_parameter(RoutineParameter::new("total_count", "integer", ParameterMode::Out));

        assert_eq!(routine.parameters_signature(), "(user_id integer)");
    }

    #[test]
    fn test_routine_parameters_signature_unnamed() {
        let routine = Routine::new("add_numbers", "public", RoutineType::Function, "sql")
            .with_parameter(RoutineParameter::new("", "integer", ParameterMode::In))
            .with_parameter(RoutineParameter::new("", "integer", ParameterMode::In));

        assert_eq!(routine.parameters_signature(), "(integer, integer)");
    }

    #[test]
    fn test_routine_is_function() {
        let func = Routine::new("get_user", "public", RoutineType::Function, "sql");
        let proc = Routine::new("update_user", "public", RoutineType::Procedure, "sql");

        assert!(func.is_function());
        assert!(!func.is_procedure());
        assert!(!proc.is_function());
        assert!(proc.is_procedure());
    }

    // ===== Full builder pattern test =====

    #[test]
    fn test_routine_full_builder() {
        let routine = Routine::new("calculate_total", "billing", RoutineType::Function, "plpgsql")
            .with_parameters(vec![
                RoutineParameter::new("order_id", "integer", ParameterMode::In).with_position(1),
                RoutineParameter::new("include_tax", "boolean", ParameterMode::In)
                    .with_default("true")
                    .with_position(2),
            ])
            .with_return_type("numeric")
            .with_volatility(Volatility::Stable)
            .with_security_definer(false)
            .with_definition("CREATE FUNCTION calculate_total...")
            .with_comment("Calculates the total amount for an order");

        assert_eq!(routine.name, "calculate_total");
        assert_eq!(routine.schema, "billing");
        assert!(routine.is_function());
        assert_eq!(routine.parameters.len(), 2);
        assert_eq!(routine.return_type, Some("numeric".to_string()));
        assert_eq!(routine.volatility, Volatility::Stable);
        assert!(!routine.security_definer);
        assert!(routine.definition.is_some());
        assert!(routine.comment.is_some());
        assert_eq!(
            routine.parameters_signature(),
            "(order_id integer, include_tax boolean)"
        );
    }
}
