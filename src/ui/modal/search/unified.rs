//! Unified search modal rendering (connections + tables)

use crate::app::{UnifiedSearchModal, UnifiedSearchSection};
use crate::model::{Connection, Table};
use crate::ui::modal::helpers::{centered_rect, highlight_match};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw_unified_search_modal(
    frame: &mut Frame,
    modal: &UnifiedSearchModal,
    connections: &[Connection],
    tables: &[Table],
) {
    let area = centered_rect(55, 70, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal container
    let block = Block::default()
        .title(" Search ")
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

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Min(4),    // First section (dynamic)
            Constraint::Min(4),    // Second section (dynamic)
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

    // Determine section positions based on tables_first
    // chunks[1] = top section, chunks[2] = bottom section
    let (conn_area, table_area) = if modal.tables_first {
        (chunks[2], chunks[1]) // tables on top, connections on bottom
    } else {
        (chunks[1], chunks[2]) // connections on top, tables on bottom
    };

    // Collect filtered items by iterating through filtered indices (O(n) instead of O(n²))
    let conn_items: Vec<&str> = modal
        .filtered_connection_indices
        .iter()
        .filter_map(|&idx| connections.get(idx).map(|c| c.name.as_str()))
        .collect();

    let table_items: Vec<&str> = modal
        .filtered_table_indices
        .iter()
        .filter_map(|&idx| tables.get(idx).map(|t| t.name.as_str()))
        .collect();

    // Draw connections section
    let conn_is_active = modal.active_section == UnifiedSearchSection::Connections;
    draw_unified_section(
        frame,
        conn_area,
        &format!(" Connections ({}) ", modal.connection_count()),
        conn_items,
        modal.selected_connection_idx,
        &modal.query,
        conn_is_active,
    );

    // Draw tables section
    let table_is_active = modal.active_section == UnifiedSearchSection::Tables;
    draw_unified_section(
        frame,
        table_area,
        &format!(" Tables ({}) ", modal.table_count()),
        table_items,
        modal.selected_table_idx,
        &modal.query,
        table_is_active,
    );

    // Draw help text
    let help_text = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(": select  "),
        Span::styled("Tab", Style::default().fg(Color::Magenta)),
        Span::raw(": switch  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(": cancel  "),
        Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
        Span::raw(": navigate"),
    ]);
    let help = Paragraph::new(help_text).alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
}

fn draw_unified_section(
    frame: &mut Frame,
    area: Rect,
    title: &str,
    items: Vec<&str>,
    selected_idx: usize,
    query: &str,
    is_active: bool,
) {
    let border_color = if is_active {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if items.is_empty() {
        let empty_msg = Paragraph::new("No results")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, inner);
        return;
    }

    let visible_height = inner.height as usize;
    let scroll_offset = if selected_idx >= visible_height {
        selected_idx - visible_height + 1
    } else {
        0
    };

    for (display_idx, item_name) in items
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .enumerate()
    {
        let actual_idx = scroll_offset + display_idx;
        let is_selected = is_active && actual_idx == selected_idx;

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if is_active {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let line = if !query.is_empty() && is_active {
            highlight_match(item_name, query, is_selected)
        } else {
            Line::from(Span::styled(item_name.to_string(), style))
        };

        let item_area = Rect {
            x: inner.x,
            y: inner.y + display_idx as u16,
            width: inner.width,
            height: 1,
        };

        let paragraph = Paragraph::new(line).style(style);
        frame.render_widget(paragraph, item_area);
    }
}
