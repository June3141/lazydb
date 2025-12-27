//! Query editor rendering

use crate::app::{App, Focus};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

pub fn draw_query_editor(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::QueryEditor;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" SQL Query ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let query_text = Paragraph::new(app.query.as_str())
        .block(block)
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: false });

    frame.render_widget(query_text, area);
}
