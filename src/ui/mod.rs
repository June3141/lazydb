pub mod components;
pub mod events;
pub mod terminal;

use crate::app::{App, ViewState};
use ratatui::prelude::*;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),  // Header (increased for tabs)
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(frame.area());

    render_header(frame, chunks[0], app);
    render_main_content(frame, chunks[1], app);
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    // Split header into title and tabs
    let header_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),  // Title
            Constraint::Length(3),  // Tabs
        ])
        .split(area);

    // Render title
    let title = ratatui::widgets::Paragraph::new("LazyDB")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    frame.render_widget(title, header_chunks[0]);

    // Render mode tabs
    render_mode_tabs(frame, header_chunks[1], app);
}

fn render_mode_tabs(frame: &mut Frame, area: Rect, app: &App) {
    let tabs = vec![
        ("Connections", ViewState::ConnectionList),
        ("Database Explorer", ViewState::DatabaseExplorer),
        ("Query Editor", ViewState::QueryEditor),
    ];

    let tab_width = area.width / 3;
    let mut tab_areas = Vec::new();
    
    for i in 0..3 {
        let x = i as u16 * tab_width;
        let width = if i == 2 { area.width - x } else { tab_width };
        tab_areas.push(Rect {
            x: area.x + x,
            y: area.y,
            width,
            height: area.height,
        });
    }

    for (i, (tab_name, view_state)) in tabs.iter().enumerate() {
        let is_active = app.current_view == *view_state;
        let style = if is_active {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default()
                .fg(Color::White)
        };

        let tab = ratatui::widgets::Paragraph::new(*tab_name)
            .style(style)
            .alignment(Alignment::Center)
            .block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
            );

        frame.render_widget(tab, tab_areas[i]);
    }
}

fn render_main_content(frame: &mut Frame, area: Rect, app: &App) {
    match app.current_view {
        ViewState::ConnectionList => {
            components::connection_list::render(frame, area, app);
        }
        ViewState::DatabaseExplorer => {
            components::database_explorer::render(frame, area, app);
        }
        ViewState::QueryEditor => {
            components::query_editor::render(frame, area, app);
        }
    }
}

fn render_footer(frame: &mut Frame, area: Rect, _app: &App) {
    let footer_text = "q: Quit | Tab: Switch View | Enter: Select";
    let footer = ratatui::widgets::Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
        );

    frame.render_widget(footer, area);
}