//! Data tab rendering with pagination

use crate::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table as RatatuiTable,
    },
    Frame,
};

pub fn draw_data_content(frame: &mut Frame, app: &mut App, area: Rect) {
    if let Some(result) = &app.result {
        if result.rows.is_empty() {
            let empty = Paragraph::new("Query returned no rows")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
            return;
        }

        // Initialize selection if not set
        if app.data_table_state.selected().is_none() {
            app.data_table_state.select(Some(0));
        }

        let selected_idx = app.data_table_state.selected().unwrap_or(0);

        // Get paginated data
        let start = app.pagination.start_index();
        let end = app.pagination.end_index();
        let page_rows = &result.rows[start..end.min(result.rows.len())];
        let page_row_count = page_rows.len();

        // Split area for data table, info bar, and pagination bar
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Data table
                Constraint::Length(1), // Info bar
                Constraint::Length(2), // Pagination bar
            ])
            .split(area);

        // Create horizontal layout for table and scrollbar
        let table_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(1),    // Table
                Constraint::Length(1), // Scrollbar
            ])
            .split(chunks[0]);

        // Create header row
        let header_cells = result.columns.iter().map(|col| {
            Cell::from(col.clone()).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells).height(1);

        // Create data rows (paginated)
        let rows: Vec<Row> = page_rows
            .iter()
            .map(|row_data| {
                let cells = row_data
                    .iter()
                    .map(|cell| Cell::from(cell.clone()).style(Style::default().fg(Color::White)));
                Row::new(cells).height(1)
            })
            .collect();

        // Calculate column widths using Ratio for accurate distribution
        // (Percentage would result in 0% width when columns > 100)
        let widths: Vec<Constraint> = if result.columns.is_empty() {
            vec![]
        } else {
            result
                .columns
                .iter()
                .map(|_| Constraint::Ratio(1, result.columns.len() as u32))
                .collect()
        };

        let table = RatatuiTable::new(rows, widths)
            .header(header)
            .row_highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");

        // Render table with state for scrolling
        frame.render_stateful_widget(table, table_chunks[0], &mut app.data_table_state);

        // Render scrollbar
        let mut scrollbar_state = ScrollbarState::new(page_row_count).position(selected_idx);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");
        frame.render_stateful_widget(scrollbar, table_chunks[1], &mut scrollbar_state);

        // Render info bar showing row position
        let info_text = format_info_bar_text(selected_idx, start, page_row_count);
        let info_bar = Paragraph::new(info_text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(info_bar, chunks[1]);

        // Draw pagination bar
        draw_pagination_bar(frame, app, chunks[2]);
    } else {
        let empty =
            Paragraph::new("No data to display").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

/// Formats the info bar text showing the current row position within the page.
///
/// # Arguments
/// * `selected_idx` - The absolute index of the selected row (0-based)
/// * `start` - The start index of the current page (0-based)
/// * `page_row_count` - The number of rows in the current page
///
/// # Returns
/// A formatted string showing "Row X/Y" where X is the 1-based position within the page
fn format_info_bar_text(selected_idx: usize, start: usize, page_row_count: usize) -> String {
    let page_relative_idx = selected_idx - start;
    format!(
        " Row {}/{} │ ↑↓/jk: navigate │ PgUp/PgDn: page │ g/G: first/last ",
        page_relative_idx + 1,
        page_row_count
    )
}

fn draw_pagination_bar(frame: &mut Frame, app: &App, area: Rect) {
    let pagination = &app.pagination;

    // Format: "< [p] Prev | Page 1/10 | Next [n] > | 50 rows | Total: 500 | Size: 50 [z]"
    let prev_style = if pagination.has_prev() {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let next_style = if pagination.has_next() {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let current_page = pagination.current_page + 1;
    let total_pages = pagination.total_pages();
    let start_row = pagination.start_index() + 1;
    let end_row = pagination.end_index();

    let spans = vec![
        Span::styled(" ◀ ", prev_style),
        Span::styled("[p]", Style::default().fg(Color::DarkGray)),
        Span::styled(" Prev ", prev_style),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" Page {}/{} ", current_page, total_pages),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(" Next ", next_style),
        Span::styled("[n]", Style::default().fg(Color::DarkGray)),
        Span::styled(" ▶ ", next_style),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" Rows {}-{} ", start_row, end_row),
            Style::default().fg(Color::Green),
        ),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" Total: {} ", pagination.total_rows),
            Style::default().fg(Color::Yellow),
        ),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!(" Size: {} ", pagination.page_size),
            Style::default().fg(Color::Magenta),
        ),
        Span::styled("[z]", Style::default().fg(Color::DarkGray)),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(" [g]", Style::default().fg(Color::DarkGray)),
        Span::styled(" First ", Style::default().fg(Color::Cyan)),
        Span::styled("[G]", Style::default().fg(Color::DarkGray)),
        Span::styled(" Last ", Style::default().fg(Color::Cyan)),
    ];

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_info_bar_text_first_page_first_row() {
        // Page 1: rows 0-49, selected_idx = 0, start = 0
        let result = format_info_bar_text(0, 0, 50);
        assert!(
            result.contains("Row 1/50"),
            "Expected 'Row 1/50' but got: {}",
            result
        );
    }

    #[test]
    fn test_format_info_bar_text_first_page_last_row() {
        // Page 1: rows 0-49, selected_idx = 49, start = 0
        let result = format_info_bar_text(49, 0, 50);
        assert!(
            result.contains("Row 50/50"),
            "Expected 'Row 50/50' but got: {}",
            result
        );
    }

    #[test]
    fn test_format_info_bar_text_second_page_first_row() {
        // Page 2: rows 50-99, selected_idx = 50, start = 50
        // Should show "Row 1/50" (first row of page 2)
        let result = format_info_bar_text(50, 50, 50);
        assert!(
            result.contains("Row 1/50"),
            "Expected 'Row 1/50' but got: {}",
            result
        );
    }

    #[test]
    fn test_format_info_bar_text_second_page_middle_row() {
        // Page 2: rows 50-99, selected_idx = 75, start = 50
        // Should show "Row 26/50" (26th row of page 2)
        let result = format_info_bar_text(75, 50, 50);
        assert!(
            result.contains("Row 26/50"),
            "Expected 'Row 26/50' but got: {}",
            result
        );
    }

    #[test]
    fn test_format_info_bar_text_partial_last_page() {
        // Last page with partial rows: rows 100-124, selected_idx = 110, start = 100
        // Should show "Row 11/25" (11th row of partial page with 25 rows)
        let result = format_info_bar_text(110, 100, 25);
        assert!(
            result.contains("Row 11/25"),
            "Expected 'Row 11/25' but got: {}",
            result
        );
    }
}
