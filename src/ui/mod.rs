mod help_bar;
mod main_panel;
mod modal;
mod sidebar;
mod status_bar;
pub mod utils;

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use help_bar::draw_help_bar;
use main_panel::{draw_main_panel, draw_query_editor};
use modal::draw_modal;
use sidebar::{draw_sidebar, draw_table_summary};
use status_bar::draw_status_bar;

pub fn draw(frame: &mut Frame, app: &App) {
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
    draw_main_panel(frame, app, right_chunks[1]);
    draw_status_bar(frame, app, right_chunks[2]);
    draw_help_bar(frame, outer_chunks[1]);

    // Draw modal on top if open
    draw_modal(frame, &app.modal_state, &app.projects, &app.query_history);
}
