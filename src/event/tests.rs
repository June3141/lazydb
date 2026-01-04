//! Tests for event handling
//!
//! This module contains tests for keyboard event handling,
//! including connection editing functionality.

use crossterm::event::{KeyCode, KeyModifiers};

use crate::app::{App, Focus, SidebarMode};
use crate::message::Message;
use crate::model::{Connection, Project};

use super::handle_normal_input;

/// Helper function to create an App with test data
fn create_test_app_with_connections() -> App {
    let mut project = Project::new("Test Project");
    project.connections.push(Connection {
        name: "test-conn".to_string(),
        host: "localhost".to_string(),
        port: 5432,
        database: "testdb".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        expanded: false,
        tables: vec![],
    });

    let mut app = App::new(vec![project]);
    // Navigate into connections mode
    app.sidebar_mode = SidebarMode::Connections(0);
    app.selected_connection_idx = 0;
    app.focus = Focus::Sidebar;
    app
}

// =============================================================================
// Connection Edit Tests
// =============================================================================

#[test]
fn test_edit_connection_key_opens_modal_in_connections_mode() {
    // Given: App is in Connections mode with sidebar focused
    let app = create_test_app_with_connections();
    assert!(matches!(app.sidebar_mode, SidebarMode::Connections(_)));
    assert_eq!(app.focus, Focus::Sidebar);

    // When: User presses 'e' key
    let result = handle_normal_input(&app, KeyCode::Char('e'), KeyModifiers::NONE);

    // Then: OpenEditConnectionModal message should be returned
    assert_eq!(result, Some(Message::OpenEditConnectionModal));
}

#[test]
fn test_edit_connection_key_does_not_open_modal_in_projects_mode() {
    // Given: App is in Projects mode
    let mut app = create_test_app_with_connections();
    app.sidebar_mode = SidebarMode::Projects;

    // When: User presses 'e' key
    let result = handle_normal_input(&app, KeyCode::Char('e'), KeyModifiers::NONE);

    // Then: OpenEditProjectModal should be returned (not connection)
    assert_eq!(result, Some(Message::OpenEditProjectModal));
}

#[test]
fn test_edit_connection_key_does_nothing_when_main_panel_focused() {
    // Given: App is in Connections mode but MainPanel is focused
    let mut app = create_test_app_with_connections();
    app.focus = Focus::MainPanel;

    // When: User presses 'e' key
    let result = handle_normal_input(&app, KeyCode::Char('e'), KeyModifiers::NONE);

    // Then: No message should be returned (key not handled)
    assert_eq!(result, None);
}

#[test]
fn test_edit_connection_key_does_nothing_when_query_editor_focused() {
    // Given: App is in Connections mode but QueryEditor is focused
    let mut app = create_test_app_with_connections();
    app.focus = Focus::QueryEditor;

    // When: User presses 'e' key
    let result = handle_normal_input(&app, KeyCode::Char('e'), KeyModifiers::NONE);

    // Then: No message should be returned (key not handled)
    assert_eq!(result, None);
}
