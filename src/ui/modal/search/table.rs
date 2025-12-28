//! Table search modal rendering

use crate::app::SearchTableModal;
use crate::model::Table;
use crate::ui::modal::helpers::{centered_rect, highlight_match};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::draw_search_help;

pub fn draw_search_table_modal(frame: &mut Frame, modal: &SearchTableModal, tables: &[Table]) {
    let area = centered_rect(50, 60, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal container
    let block = Block::default()
        .title(" Search Tables ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    frame.render_widget(block, area);

    // Inner area for content
    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Layout for search input and results
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Length(1), // Results count
            Constraint::Min(5),    // Results list
            Constraint::Length(2), // Help text
        ])
        .split(inner);

    // Draw search input field
    let search_display = format!("{}_", modal.query);
    let search_input = Paragraph::new(search_display)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Search "),
        );
    frame.render_widget(search_input, chunks[0]);

    // Draw results count
    let count_text = format!(
        " {} of {} tables ",
        modal.filtered_indices.len(),
        tables.len()
    );
    let count_paragraph = Paragraph::new(count_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(count_paragraph, chunks[1]);

    // Draw results list
    let results_area = chunks[2];
    let visible_height = results_area.height as usize;

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = if modal.selected_idx >= visible_height {
        modal.selected_idx - visible_height + 1
    } else {
        0
    };

    let results_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray))
        .title(" Results ");
    let results_inner = results_block.inner(results_area);
    frame.render_widget(results_block, results_area);

    // Render each visible result
    for (display_idx, &table_idx) in modal
        .filtered_indices
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .enumerate()
    {
        if let Some(table) = tables.get(table_idx) {
            let actual_idx = scroll_offset + display_idx;
            let is_selected = actual_idx == modal.selected_idx;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Highlight matching text
            let line = if !modal.query.is_empty() {
                highlight_match(&table.name, &modal.query, is_selected)
            } else {
                Line::from(Span::styled(&table.name, style))
            };

            let item_area = Rect {
                x: results_inner.x,
                y: results_inner.y + display_idx as u16,
                width: results_inner.width,
                height: 1,
            };

            let paragraph = Paragraph::new(line).style(style);
            frame.render_widget(paragraph, item_area);
        }
    }

    // Draw help text
    draw_search_help(frame, chunks[3]);
}
