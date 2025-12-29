use crate::app::App;
use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme;

pub fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Status ")
        .borders(Borders::ALL)
        .border_style(theme::border_inactive());

    let status_parts = if let Some(result) = &app.result {
        vec![
            Span::styled("✓ ", theme::selected()),
            Span::styled(format!("{} rows", result.rows.len()), theme::text()),
            Span::styled(" │ ", theme::muted()),
            Span::styled(format!("{}ms", result.execution_time_ms), theme::muted()),
            Span::styled(" │ ", theme::muted()),
            Span::styled(&app.status_message, theme::selected()),
        ]
    } else {
        vec![Span::styled(&app.status_message, theme::muted())]
    };

    let status = Paragraph::new(Line::from(status_parts)).block(block);
    frame.render_widget(status, area);
}
