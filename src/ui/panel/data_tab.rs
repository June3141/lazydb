//! Data tab rendering with pagination

use crate::app::App;
use crate::ui::theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
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
            let empty = Paragraph::new("Query returned no rows").style(theme::muted());
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
        let header_cells = result
            .columns
            .iter()
            .map(|col| Cell::from(col.clone()).style(theme::header()));
        let header = Row::new(header_cells).height(1);

        // Create data rows (paginated)
        let rows: Vec<Row> = page_rows
            .iter()
            .map(|row_data| {
                let cells = row_data
                    .iter()
                    .map(|cell| Cell::from(cell.clone()).style(theme::text()));
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
            .row_highlight_style(theme::row_highlight())
            .highlight_symbol("▶ ");

        // Render table with state for scrolling
        frame.render_stateful_widget(table, table_chunks[0], &mut app.data_table_state);

        // Render scrollbar (use page-relative index, not absolute)
        let page_relative_idx = page_relative_index(selected_idx, start);
        let mut scrollbar_state = ScrollbarState::new(page_row_count).position(page_relative_idx);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");
        frame.render_stateful_widget(scrollbar, table_chunks[1], &mut scrollbar_state);

        // Render info bar showing row position
        let info_text = format_info_bar_text(selected_idx, start, page_row_count);
        let info_bar = Paragraph::new(info_text).style(theme::muted());
        frame.render_widget(info_bar, chunks[1]);

        // Draw pagination bar
        draw_pagination_bar(frame, app, chunks[2]);
    } else {
        let empty = Paragraph::new("No data to display").style(theme::muted());
        frame.render_widget(empty, area);
    }
}

/// Calculate page-relative index for scrollbar position.
///
/// The scrollbar should show position within the current page, not the absolute
/// index across all pages. For example, on page 2 with 50 rows per page,
/// if selected_idx is 75 (absolute), the page-relative index should be 25.
fn page_relative_index(selected_idx: usize, page_start: usize) -> usize {
    selected_idx.saturating_sub(page_start)
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
        theme::selected()
    } else {
        theme::muted()
    };

    let next_style = if pagination.has_next() {
        theme::selected()
    } else {
        theme::muted()
    };

    let current_page = pagination.current_page + 1;
    let total_pages = pagination.total_pages();
    let start_row = pagination.start_index() + 1;
    let end_row = pagination.end_index();

    let spans = vec![
        Span::styled(" ◀ ", prev_style),
        Span::styled("[p]", theme::muted()),
        Span::styled(" Prev ", prev_style),
        Span::styled("│", theme::muted()),
        Span::styled(format!(" Page {}/{} ", current_page, total_pages), theme::text()),
        Span::styled("│", theme::muted()),
        Span::styled(" Next ", next_style),
        Span::styled("[n]", theme::muted()),
        Span::styled(" ▶ ", next_style),
        Span::styled("│", theme::muted()),
        Span::styled(format!(" Rows {}-{} ", start_row, end_row), theme::selected()),
        Span::styled("│", theme::muted()),
        Span::styled(format!(" Total: {} ", pagination.total_rows), theme::header()),
        Span::styled("│", theme::muted()),
        Span::styled(format!(" Size: {} ", pagination.page_size), theme::muted()),
        Span::styled("[z]", theme::muted()),
        Span::styled("│", theme::muted()),
        Span::styled(" [g]", theme::muted()),
        Span::styled(" First ", theme::selected()),
        Span::styled("[G]", theme::muted()),
        Span::styled(" Last ", theme::selected()),
    ];

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line).block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(theme::border_inactive()),
    );

    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_relative_index_first_page() {
        // Page 1: start=0, selected=0 -> relative=0
        assert_eq!(page_relative_index(0, 0), 0);
        // Page 1: start=0, selected=25 -> relative=25
        assert_eq!(page_relative_index(25, 0), 25);
        // Page 1: start=0, selected=49 -> relative=49
        assert_eq!(page_relative_index(49, 0), 49);
    }

    #[test]
    fn test_page_relative_index_second_page() {
        // Page 2: start=50, selected=50 -> relative=0
        assert_eq!(page_relative_index(50, 50), 0);
        // Page 2: start=50, selected=75 -> relative=25
        assert_eq!(page_relative_index(75, 50), 25);
        // Page 2: start=50, selected=99 -> relative=49
        assert_eq!(page_relative_index(99, 50), 49);
    }

    #[test]
    fn test_page_relative_index_later_pages() {
        // Page 5: start=200, selected=225 -> relative=25
        assert_eq!(page_relative_index(225, 200), 25);
        // Page 10: start=450, selected=475 -> relative=25
        assert_eq!(page_relative_index(475, 450), 25);
    }

    #[test]
    fn test_page_relative_index_saturating_sub() {
        // Edge case: if somehow selected_idx < page_start, should return 0
        assert_eq!(page_relative_index(10, 50), 0);
    }

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
