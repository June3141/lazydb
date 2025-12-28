//! Modal rendering helper functions

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::ui::theme;

/// Create a centered rectangle with given percentage of width and height
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

/// Draw a text input field with label
pub fn draw_input_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    is_password: bool,
) {
    let style = if focused {
        theme::input_focused()
    } else {
        theme::text()
    };

    let border_style = if focused {
        theme::input_border_focused()
    } else {
        theme::input_border_inactive()
    };

    // Mask password field
    let masked_value = if is_password {
        "*".repeat(value.len())
    } else {
        value.to_string()
    };

    // Display value with cursor if focused
    let display_value = if focused {
        format!("{}_", masked_value)
    } else {
        masked_value
    };

    let input = Paragraph::new(display_value).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(format!(" {} ", label)),
    );

    frame.render_widget(input, area);
}

/// Highlight matching substring in text
pub fn highlight_match(text: &str, query: &str, is_selected: bool) -> Line<'static> {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    let base_style = if is_selected {
        theme::focused()
    } else {
        theme::text()
    };

    let highlight_style = if is_selected {
        theme::highlight_match_selected()
    } else {
        theme::highlight_match()
    };

    if let Some(start) = text_lower.find(&query_lower) {
        let end = start + query.len();
        let before = &text[..start];
        let matched = &text[start..end];
        let after = &text[end..];

        Line::from(vec![
            Span::styled(before.to_string(), base_style),
            Span::styled(matched.to_string(), highlight_style),
            Span::styled(after.to_string(), base_style),
        ])
    } else {
        Line::from(Span::styled(text.to_string(), base_style))
    }
}

/// Draw standard OK/Cancel buttons with custom styles
#[allow(dead_code)]
pub fn draw_ok_cancel_buttons(
    frame: &mut Frame,
    area: Rect,
    ok_focused: bool,
    cancel_focused: bool,
    _ok_style_base: (ratatui::style::Color, ratatui::style::Color),
    _cancel_style_base: (ratatui::style::Color, ratatui::style::Color),
) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // OK button
    let ok_style = if ok_focused {
        theme::focused()
    } else {
        theme::selected()
    };

    let ok_button = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("[ OK ]", ok_style),
        Span::raw(" "),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));

    // Cancel button
    let cancel_style = if cancel_focused {
        theme::button_cancel_focused()
    } else {
        theme::muted()
    };

    let cancel_button = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("[ Cancel ]", cancel_style),
        Span::raw(" "),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));

    frame.render_widget(ok_button, button_chunks[0]);
    frame.render_widget(cancel_button, button_chunks[1]);
}
