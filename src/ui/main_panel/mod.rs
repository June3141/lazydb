//! Main panel rendering
//!
//! This module handles the main content panel including Schema, Data, and Relations tabs.

mod data_tab;
mod query_editor;
mod relations_tab;
mod schema_tab;

use crate::app::{App, Focus, MainPanelTab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
    Frame,
};

// Re-export for external use
pub use query_editor::draw_query_editor;

pub fn draw_main_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focus == Focus::MainPanel;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Split area for tabs and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Tabs
            Constraint::Min(1),    // Content
        ])
        .split(area);

    // Draw tabs
    let tab_titles = vec!["Schema [s]", "Data [d]", "Relations [r]"];
    let selected_tab = match app.main_panel_tab {
        MainPanelTab::Schema => 0,
        MainPanelTab::Data => 1,
        MainPanelTab::Relations => 2,
    };

    let tabs = Tabs::new(tab_titles)
        .select(selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");

    frame.render_widget(tabs, chunks[0]);

    // Draw content based on selected tab
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_area = content_block.inner(chunks[1]);
    frame.render_widget(content_block, chunks[1]);

    match app.main_panel_tab {
        MainPanelTab::Schema => schema_tab::draw_schema_content(frame, app, inner_area),
        MainPanelTab::Data => data_tab::draw_data_content(frame, app, inner_area),
        MainPanelTab::Relations => relations_tab::draw_relations_content(frame, app, inner_area),
    }
}
