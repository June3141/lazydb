use crate::app::App;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Status ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let status_parts = if let Some(result) = &app.result {
        vec![
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::styled(
                format!("{} rows", result.rows.len()),
                Style::default().fg(Color::White),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}ms", result.execution_time_ms),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.status_message, Style::default().fg(Color::Cyan)),
        ]
    } else {
        vec![Span::styled(
            &app.status_message,
            Style::default().fg(Color::Gray),
        )]
    };

    let status = Paragraph::new(Line::from(status_parts)).block(block);
    frame.render_widget(status, area);
}
