pub mod components;
pub mod events;
pub mod terminal;

use crate::app::{App, ViewState};
use ratatui::prelude::*;

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Main content
            Constraint::Length(3),  // Footer
        ])
        .split(frame.area());

    render_header(frame, chunks[0], app);
    render_main_content(frame, chunks[1], app);
    render_footer(frame, chunks[2], app);
}

fn render_header(frame: &mut Frame, area: Rect, app: &App) {
    let title = match app.current_view {
        ViewState::ConnectionList => "LazyDB - Connections",
        ViewState::DatabaseExplorer => "LazyDB - Database Explorer",
        ViewState::QueryEditor => "LazyDB - Query Editor",
    };

    let header = ratatui::widgets::Paragraph::new(title)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Center)
        .block(
            ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("LazyDB")
        );

    frame.render_widget(header, area);
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