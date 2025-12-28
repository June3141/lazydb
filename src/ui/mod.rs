mod help_bar;
mod modal;
mod panel;
mod sidebar;
mod status_bar;
pub mod utils;

use crate::app::{App, SidebarMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use help_bar::draw_help_bar;
use modal::draw_modal;
use panel::{draw_panel, draw_query_editor};
use sidebar::{draw_sidebar, draw_table_summary};
use status_bar::draw_status_bar;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let size = frame.area();

    // Top-level layout: Content area | Help bar
    let outer_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),    // Content area
            Constraint::Length(1), // Help bar
        ])
        .split(size);

    let content_area = outer_chunks[0];

    // Main layout: Sidebar | Main area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28), // Sidebar width
            Constraint::Min(40),    // Main area
        ])
        .split(content_area);

    // Sidebar layout: Connections tree | Table info summary
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),    // Connections tree
            Constraint::Length(7), // Table info summary (compact)
        ])
        .split(main_chunks[0]);

    // Main area layout: Query editor | Main panel | Status
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // Query editor
            Constraint::Min(10),   // Main panel (Schema/Data)
            Constraint::Length(3), // Status bar
        ])
        .split(main_chunks[1]);

    draw_sidebar(frame, app, sidebar_chunks[0]);
    draw_table_summary(frame, app, sidebar_chunks[1]);
    draw_query_editor(frame, app, right_chunks[0]);
    draw_panel(frame, app, right_chunks[1]);
    draw_status_bar(frame, app, right_chunks[2]);
    draw_help_bar(frame, outer_chunks[1]);

    // Draw modal on top if open
    // Get current project's connections for SearchConnection modal
    let connections = match app.sidebar_mode {
        SidebarMode::Connections(proj_idx) => app
            .projects
            .get(proj_idx)
            .map(|p| p.connections.as_slice())
            .unwrap_or(&[]),
        SidebarMode::Projects => &[],
    };
    let tables = app.current_connection_tables();
    draw_modal(
        frame,
        &app.modal_state,
        &app.projects,
        connections,
        tables,
        &app.query_history,
        &app.column_visibility,
    );
}
