//! History modal rendering

use crate::app::HistoryModal;
use crate::model::QueryHistory;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

use super::helpers::centered_rect;

pub fn draw_history_modal(frame: &mut Frame, modal: &HistoryModal, history: &QueryHistory) {
    let area = centered_rect(70, 70, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal container
    let block = Block::default()
        .title(" Query History ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(block, area);

    // Inner area for content
    let inner = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(2),
    };

    // Layout for history list and help text
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // History list
            Constraint::Length(2), // Help text
        ])
        .split(inner);

    // Create list items from history entries
    let items: Vec<ListItem> = history
        .entries
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let status_icon = if entry.is_success() { "+" } else { "x" };
            let time_str = entry.executed_at.format("%m/%d %H:%M").to_string();

            // Truncate query if too long (use chars for UTF-8 safety)
            let min_query_len = 10;
            let max_query_len =
                std::cmp::max((chunks[0].width as usize).saturating_sub(30), min_query_len);
            let query_display = if entry.query.chars().count() > max_query_len {
                let safe_trunc: String = entry
                    .query
                    .chars()
                    .take(max_query_len.saturating_sub(3))
                    .collect();
                format!("{}...", safe_trunc)
            } else {
                entry.query.clone()
            };

            let style = if idx == modal.selected_idx {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else if entry.is_success() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::Red)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("[{}] ", status_icon), style),
                Span::styled(format!("{} ", time_str), style),
                Span::styled(query_display, style),
            ]))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray))
            .title(format!(" {} queries ", history.len())),
    );

    // Render with stateful list to show selection
    let mut list_state = ListState::default();
    list_state.select(Some(modal.selected_idx));
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    // Help text
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Yellow)),
        Span::raw(": select  "),
        Span::styled("j/k", Style::default().fg(Color::Yellow)),
        Span::raw(": navigate  "),
        Span::styled("c", Style::default().fg(Color::Yellow)),
        Span::raw(": clear  "),
        Span::styled("Esc/q", Style::default().fg(Color::Yellow)),
        Span::raw(": close"),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(help, chunks[1]);
}
