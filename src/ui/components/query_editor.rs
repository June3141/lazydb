use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40), // Query input
            Constraint::Percentage(60), // Query results
        ])
        .split(area);

    render_query_input(frame, chunks[0]);
    render_query_results(frame, chunks[1]);
}

fn render_query_input(frame: &mut Frame, area: Rect) {
    let query_input = Paragraph::new("SELECT * FROM users WHERE id = 1;")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Query Editor")
        )
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });

    frame.render_widget(query_input, area);
}

fn render_query_results(frame: &mut Frame, area: Rect) {
    let query_results = Paragraph::new("Query results will appear here...")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Results")
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(query_results, area);
}