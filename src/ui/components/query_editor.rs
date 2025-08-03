use crate::app::{App, QueryEditorPane};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Query input
            Constraint::Percentage(60), // Query results
        ])
        .split(area);

    render_query_input(frame, chunks[0], app);
    render_query_results(frame, chunks[1], app);
}

fn render_query_input(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.query_editor_state.focused_pane == QueryEditorPane::Editor;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let title = "Query Editor";

    let query_input = Paragraph::new("SELECT * FROM users WHERE id = 1;")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style)
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    frame.render_widget(query_input, area);
}

fn render_query_results(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.query_editor_state.focused_pane == QueryEditorPane::Results;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let title = "Results";

    let query_results = Paragraph::new("Query results will appear here...")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style)
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(query_results, area);
}