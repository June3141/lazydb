//! Trigger types and structures

/// Trigger timing (when the trigger fires)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerTiming {
    Before,
    After,
    InsteadOf,
}

impl std::fmt::Display for TriggerTiming {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerTiming::Before => write!(f, "BEFORE"),
            TriggerTiming::After => write!(f, "AFTER"),
            TriggerTiming::InsteadOf => write!(f, "INSTEAD OF"),
        }
    }
}

/// Trigger event (what operation fires the trigger)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerEvent {
    Insert,
    Update,
    Delete,
    Truncate,
}

impl std::fmt::Display for TriggerEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerEvent::Insert => write!(f, "INSERT"),
            TriggerEvent::Update => write!(f, "UPDATE"),
            TriggerEvent::Delete => write!(f, "DELETE"),
            TriggerEvent::Truncate => write!(f, "TRUNCATE"),
        }
    }
}

/// Trigger orientation (row-level or statement-level)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerOrientation {
    Row,
    Statement,
}

impl std::fmt::Display for TriggerOrientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerOrientation::Row => write!(f, "ROW"),
            TriggerOrientation::Statement => write!(f, "STATEMENT"),
        }
    }
}

/// Database trigger information
#[derive(Debug, Clone, PartialEq)]
pub struct Trigger {
    /// Trigger name
    pub name: String,
    /// Timing when the trigger fires
    pub timing: TriggerTiming,
    /// Events that fire the trigger
    pub events: Vec<TriggerEvent>,
    /// Whether this is a row-level or statement-level trigger
    pub orientation: TriggerOrientation,
    /// The function called by the trigger
    pub function_name: String,
    /// Full trigger definition (CREATE TRIGGER statement)
    pub definition: Option<String>,
    /// Whether the trigger is enabled
    pub enabled: bool,
}

impl Trigger {
    /// Create a new trigger with required fields
    pub fn new(
        name: impl Into<String>,
        timing: TriggerTiming,
        function_name: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            timing,
            events: Vec::new(),
            orientation: TriggerOrientation::Row,
            function_name: function_name.into(),
            definition: None,
            enabled: true,
        }
    }

    /// Add an event that fires this trigger
    pub fn with_event(mut self, event: TriggerEvent) -> Self {
        self.events.push(event);
        self
    }

    /// Set multiple events
    pub fn with_events(mut self, events: Vec<TriggerEvent>) -> Self {
        self.events = events;
        self
    }

    /// Set the orientation
    pub fn with_orientation(mut self, orientation: TriggerOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Set the trigger definition
    pub fn with_definition(mut self, definition: impl Into<String>) -> Self {
        self.definition = Some(definition.into());
        self
    }

    /// Set whether the trigger is enabled
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Get a formatted string of events (e.g., "INSERT OR UPDATE")
    pub fn events_display(&self) -> String {
        self.events
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(" OR ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== TriggerTiming tests =====

    #[test]
    fn test_trigger_timing_display_before() {
        assert_eq!(TriggerTiming::Before.to_string(), "BEFORE");
    }

    #[test]
    fn test_trigger_timing_display_after() {
        assert_eq!(TriggerTiming::After.to_string(), "AFTER");
    }

    #[test]
    fn test_trigger_timing_display_instead_of() {
        assert_eq!(TriggerTiming::InsteadOf.to_string(), "INSTEAD OF");
    }

    // ===== TriggerEvent tests =====

    #[test]
    fn test_trigger_event_display_insert() {
        assert_eq!(TriggerEvent::Insert.to_string(), "INSERT");
    }

    #[test]
    fn test_trigger_event_display_update() {
        assert_eq!(TriggerEvent::Update.to_string(), "UPDATE");
    }

    #[test]
    fn test_trigger_event_display_delete() {
        assert_eq!(TriggerEvent::Delete.to_string(), "DELETE");
    }

    #[test]
    fn test_trigger_event_display_truncate() {
        assert_eq!(TriggerEvent::Truncate.to_string(), "TRUNCATE");
    }

    // ===== TriggerOrientation tests =====

    #[test]
    fn test_trigger_orientation_display_row() {
        assert_eq!(TriggerOrientation::Row.to_string(), "ROW");
    }

    #[test]
    fn test_trigger_orientation_display_statement() {
        assert_eq!(TriggerOrientation::Statement.to_string(), "STATEMENT");
    }

    // ===== Trigger struct tests =====

    #[test]
    fn test_trigger_new() {
        let trigger = Trigger::new("audit_trigger", TriggerTiming::After, "audit_function");

        assert_eq!(trigger.name, "audit_trigger");
        assert_eq!(trigger.timing, TriggerTiming::After);
        assert_eq!(trigger.function_name, "audit_function");
        assert!(trigger.events.is_empty());
        assert_eq!(trigger.orientation, TriggerOrientation::Row);
        assert!(trigger.definition.is_none());
        assert!(trigger.enabled);
    }

    #[test]
    fn test_trigger_with_event() {
        let trigger = Trigger::new("audit_trigger", TriggerTiming::After, "audit_function")
            .with_event(TriggerEvent::Insert);

        assert_eq!(trigger.events.len(), 1);
        assert_eq!(trigger.events[0], TriggerEvent::Insert);
    }

    #[test]
    fn test_trigger_with_multiple_events() {
        let trigger = Trigger::new("audit_trigger", TriggerTiming::After, "audit_function")
            .with_event(TriggerEvent::Insert)
            .with_event(TriggerEvent::Update)
            .with_event(TriggerEvent::Delete);

        assert_eq!(trigger.events.len(), 3);
        assert!(trigger.events.contains(&TriggerEvent::Insert));
        assert!(trigger.events.contains(&TriggerEvent::Update));
        assert!(trigger.events.contains(&TriggerEvent::Delete));
    }

    #[test]
    fn test_trigger_with_events_vec() {
        let events = vec![TriggerEvent::Insert, TriggerEvent::Update];
        let trigger = Trigger::new("audit_trigger", TriggerTiming::After, "audit_function")
            .with_events(events);

        assert_eq!(trigger.events.len(), 2);
    }

    #[test]
    fn test_trigger_with_orientation() {
        let trigger = Trigger::new("audit_trigger", TriggerTiming::After, "audit_function")
            .with_orientation(TriggerOrientation::Statement);

        assert_eq!(trigger.orientation, TriggerOrientation::Statement);
    }

    #[test]
    fn test_trigger_with_definition() {
        let definition = "CREATE TRIGGER audit_trigger AFTER INSERT ON users FOR EACH ROW EXECUTE FUNCTION audit_function()";
        let trigger = Trigger::new("audit_trigger", TriggerTiming::After, "audit_function")
            .with_definition(definition);

        assert_eq!(trigger.definition, Some(definition.to_string()));
    }

    #[test]
    fn test_trigger_with_enabled_false() {
        let trigger = Trigger::new("disabled_trigger", TriggerTiming::Before, "some_function")
            .with_enabled(false);

        assert!(!trigger.enabled);
    }

    #[test]
    fn test_trigger_events_display_single() {
        let trigger = Trigger::new("t", TriggerTiming::After, "f").with_event(TriggerEvent::Insert);

        assert_eq!(trigger.events_display(), "INSERT");
    }

    #[test]
    fn test_trigger_events_display_multiple() {
        let trigger = Trigger::new("t", TriggerTiming::After, "f")
            .with_event(TriggerEvent::Insert)
            .with_event(TriggerEvent::Update);

        assert_eq!(trigger.events_display(), "INSERT OR UPDATE");
    }

    #[test]
    fn test_trigger_events_display_all() {
        let trigger = Trigger::new("t", TriggerTiming::After, "f")
            .with_event(TriggerEvent::Insert)
            .with_event(TriggerEvent::Update)
            .with_event(TriggerEvent::Delete)
            .with_event(TriggerEvent::Truncate);

        assert_eq!(
            trigger.events_display(),
            "INSERT OR UPDATE OR DELETE OR TRUNCATE"
        );
    }

    #[test]
    fn test_trigger_events_display_empty() {
        let trigger = Trigger::new("t", TriggerTiming::After, "f");

        assert_eq!(trigger.events_display(), "");
    }

    // ===== Builder pattern test =====

    #[test]
    fn test_trigger_full_builder() {
        let trigger = Trigger::new("complex_trigger", TriggerTiming::Before, "validate_data")
            .with_events(vec![TriggerEvent::Insert, TriggerEvent::Update])
            .with_orientation(TriggerOrientation::Row)
            .with_definition("CREATE TRIGGER complex_trigger...")
            .with_enabled(true);

        assert_eq!(trigger.name, "complex_trigger");
        assert_eq!(trigger.timing, TriggerTiming::Before);
        assert_eq!(trigger.function_name, "validate_data");
        assert_eq!(trigger.events.len(), 2);
        assert_eq!(trigger.orientation, TriggerOrientation::Row);
        assert!(trigger.definition.is_some());
        assert!(trigger.enabled);
    }
}
