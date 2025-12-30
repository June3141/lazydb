//! Query input modal rendering

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::QueryInputModal;
use crate::ui::modal::helpers::centered_rect;
use crate::ui::theme;

/// Draw the query input modal
pub fn draw_query_input_modal(frame: &mut Frame, modal: &QueryInputModal) {
    // Modal takes 70% width, 50% height
    let area = centered_rect(70, 50, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Create the modal layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title area (included in block)
            Constraint::Min(5),    // Query input area
            Constraint::Length(2), // Help text
        ])
        .margin(1)
        .split(area);

    // Draw the modal border
    let block = Block::default()
        .title(" Execute Query ")
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    frame.render_widget(block, area);

    // Draw the query input area with cursor
    draw_query_input(frame, chunks[1], modal);

    // Draw help text
    draw_help_text(frame, chunks[2]);
}

/// Draw the query input text area with cursor
fn draw_query_input(frame: &mut Frame, area: Rect, modal: &QueryInputModal) {
    let query = &modal.query;
    let cursor_pos = modal.cursor_pos;

    // Split the query at cursor position for display
    let (before_cursor, after_cursor) = if cursor_pos <= query.len() {
        (&query[..cursor_pos], &query[cursor_pos..])
    } else {
        (query.as_str(), "")
    };

    // Build lines with cursor
    let mut lines: Vec<Line> = Vec::new();
    let mut current_line_spans: Vec<Span> = Vec::new();

    // Process text before cursor
    for ch in before_cursor.chars() {
        if ch == '\n' {
            lines.push(Line::from(current_line_spans.clone()));
            current_line_spans.clear();
        } else {
            current_line_spans.push(Span::styled(ch.to_string(), theme::text()));
        }
    }

    // Add cursor (blinking underscore)
    current_line_spans.push(Span::styled(
        "_",
        Style::default()
            .fg(theme::ACCENT)
            .add_modifier(Modifier::SLOW_BLINK),
    ));

    // Process text after cursor
    for ch in after_cursor.chars() {
        if ch == '\n' {
            lines.push(Line::from(current_line_spans.clone()));
            current_line_spans.clear();
        } else {
            current_line_spans.push(Span::styled(ch.to_string(), theme::text()));
        }
    }

    // Add the last line if not empty or if we have no lines yet
    if !current_line_spans.is_empty() || lines.is_empty() {
        lines.push(Line::from(current_line_spans));
    }

    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme::input_border_focused())
        .title(" SQL ")
        .title_style(theme::text());

    let paragraph = Paragraph::new(lines)
        .block(input_block)
        .style(theme::text())
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Draw help text at the bottom
fn draw_help_text(frame: &mut Frame, area: Rect) {
    let help_spans = vec![
        Span::styled("Ctrl+Enter", theme::header()),
        Span::styled(": Execute  ", theme::muted()),
        Span::styled("Esc", theme::header()),
        Span::styled(": Cancel  ", theme::muted()),
        Span::styled("Ctrl+r", theme::header()),
        Span::styled(": History  ", theme::muted()),
        Span::styled("Ctrl+u", theme::header()),
        Span::styled(": Clear", theme::muted()),
    ];

    let help = Paragraph::new(Line::from(help_spans))
        .style(theme::muted())
        .alignment(ratatui::layout::Alignment::Center);

    frame.render_widget(help, area);
}
