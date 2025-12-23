use crate::app::{App, Focus, MainPanelTab, SchemaSubTab};
use crate::model::{ForeignKey, Table};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Table as RatatuiTable, Tabs, Wrap,
    },
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

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

pub fn draw_main_panel(frame: &mut Frame, app: &mut App, area: Rect) {
    let is_focused = app.focus == Focus::MainPanel;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Split area for tabs and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Tabs
            Constraint::Min(1),    // Content
        ])
        .split(area);

    // Draw tabs
    let tab_titles = vec!["Schema [s]", "Data [d]", "Relations [r]"];
    let selected_tab = match app.main_panel_tab {
        MainPanelTab::Schema => 0,
        MainPanelTab::Data => 1,
        MainPanelTab::Relations => 2,
    };

    let tabs = Tabs::new(tab_titles)
        .select(selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");

    frame.render_widget(tabs, chunks[0]);

    // Draw content based on selected tab
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_area = content_block.inner(chunks[1]);
    frame.render_widget(content_block, chunks[1]);

    match app.main_panel_tab {
        MainPanelTab::Schema => draw_schema_content(frame, app, inner_area),
        MainPanelTab::Data => draw_data_content(frame, app, inner_area),
        MainPanelTab::Relations => draw_relations_content(frame, app, inner_area),
    }
}

fn draw_schema_content(frame: &mut Frame, app: &App, area: Rect) {
    // Split for sub-tabs and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Sub-tabs
            Constraint::Min(1),    // Content
        ])
        .split(area);

    // Draw sub-tabs
    let sub_tab_titles = vec![
        "Columns [1]",
        "Indexes [2]",
        "Foreign Keys [3]",
        "Constraints [4]",
    ];
    let selected_sub_tab = match app.schema_sub_tab {
        SchemaSubTab::Columns => 0,
        SchemaSubTab::Indexes => 1,
        SchemaSubTab::ForeignKeys => 2,
        SchemaSubTab::Constraints => 3,
    };

    let sub_tabs = Tabs::new(sub_tab_titles)
        .select(selected_sub_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");

    frame.render_widget(sub_tabs, chunks[0]);

    // Draw content based on selected sub-tab
    match app.schema_sub_tab {
        SchemaSubTab::Columns => draw_columns_content(frame, app, chunks[1]),
        SchemaSubTab::Indexes => draw_indexes_content(frame, app, chunks[1]),
        SchemaSubTab::ForeignKeys => draw_foreign_keys_content(frame, app, chunks[1]),
        SchemaSubTab::Constraints => draw_constraints_content(frame, app, chunks[1]),
    }
}

fn draw_columns_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        // Create header
        let header_cells = ["", "Name", "Type", "Null", "Default", "Key"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            });
        let header = Row::new(header_cells).height(1);

        // Create rows
        let rows = table.columns.iter().map(|col| {
            let pk_marker = if col.is_primary_key {
                "🔑"
            } else if col.is_unique {
                "⚡"
            } else {
                ""
            };

            let key_info = if col.is_primary_key {
                "PK".to_string()
            } else if col.is_unique {
                "UQ".to_string()
            } else {
                "".to_string()
            };

            let null_str = if col.is_nullable { "YES" } else { "NO" };
            let default_str = col.default_value.as_deref().unwrap_or("-");

            let name_style = if col.is_primary_key {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                Cell::from(pk_marker),
                Cell::from(col.name.clone()).style(name_style),
                Cell::from(col.data_type.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(null_str).style(if col.is_nullable {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Red)
                }),
                Cell::from(default_str).style(Style::default().fg(Color::Green)),
                Cell::from(key_info).style(Style::default().fg(Color::Magenta)),
            ])
            .height(1)
        });

        let widths = [
            Constraint::Length(3),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Length(5),
            Constraint::Percentage(25),
            Constraint::Length(5),
        ];

        let table_widget = RatatuiTable::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view columns")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

fn draw_indexes_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.indexes.is_empty() {
            let empty =
                Paragraph::new("No indexes defined").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
            return;
        }

        // Create header
        let header_cells = ["Name", "Type", "Method", "Columns"].iter().map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells).height(1);

        // Create rows
        let rows = table.indexes.iter().map(|idx| {
            let columns_str = idx
                .columns
                .iter()
                .map(|c| {
                    if matches!(c.order, crate::model::SortOrder::Desc) {
                        format!("{} DESC", c.name)
                    } else {
                        c.name.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");

            let type_style = match idx.index_type {
                crate::model::IndexType::Primary => Style::default().fg(Color::Yellow),
                crate::model::IndexType::Unique => Style::default().fg(Color::Magenta),
                _ => Style::default().fg(Color::White),
            };

            Row::new(vec![
                Cell::from(idx.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(idx.index_type.to_string()).style(type_style),
                Cell::from(idx.method.to_string()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(columns_str),
            ])
            .height(1)
        });

        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(40),
        ];

        let table_widget = RatatuiTable::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view indexes")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

fn draw_foreign_keys_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.foreign_keys.is_empty() {
            let empty = Paragraph::new("No foreign keys defined")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
            return;
        }

        // Create header
        let header_cells = ["Name", "Column", "References", "ON DELETE", "ON UPDATE"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            });
        let header = Row::new(header_cells).height(1);

        // Create rows
        let rows = table.foreign_keys.iter().map(|fk| {
            let columns_str = fk.columns.join(", ");
            let ref_str = format!(
                "{}.{}",
                fk.referenced_table,
                fk.referenced_columns.join(", ")
            );

            Row::new(vec![
                Cell::from(fk.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(columns_str),
                Cell::from(ref_str).style(Style::default().fg(Color::Green)),
                Cell::from(fk.on_delete.to_string()).style(Style::default().fg(Color::Red)),
                Cell::from(fk.on_update.to_string()).style(Style::default().fg(Color::Magenta)),
            ])
            .height(1)
        });

        let widths = [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(17),
            Constraint::Percentage(18),
        ];

        let table_widget = RatatuiTable::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view foreign keys")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

fn draw_constraints_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.constraints.is_empty() {
            let empty = Paragraph::new("No constraints defined")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
            return;
        }

        // Create header
        let header_cells = ["Name", "Type", "Columns", "Definition"].iter().map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells).height(1);

        // Create rows
        let rows = table.constraints.iter().map(|c| {
            let columns_str = if c.columns.is_empty() {
                "-".to_string()
            } else {
                c.columns.join(", ")
            };
            let def_str = c.definition.as_deref().unwrap_or("-");

            let type_style = match c.constraint_type {
                crate::model::ConstraintType::PrimaryKey => Style::default().fg(Color::Yellow),
                crate::model::ConstraintType::Unique => Style::default().fg(Color::Magenta),
                crate::model::ConstraintType::ForeignKey => Style::default().fg(Color::Cyan),
                crate::model::ConstraintType::Check => Style::default().fg(Color::Green),
                crate::model::ConstraintType::NotNull => Style::default().fg(Color::LightBlue),
                crate::model::ConstraintType::Default => Style::default().fg(Color::LightYellow),
                crate::model::ConstraintType::Exclusion => Style::default().fg(Color::LightMagenta),
            };

            Row::new(vec![
                Cell::from(c.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(c.constraint_type.to_string()).style(type_style),
                Cell::from(columns_str),
                Cell::from(def_str).style(Style::default().fg(Color::DarkGray)),
            ])
            .height(1)
        });

        let widths = [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ];

        let table_widget = RatatuiTable::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view constraints")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

fn draw_data_content(frame: &mut Frame, app: &mut App, area: Rect) {
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
        let info_text = format!(
            " Row {}/{} │ ↑↓/jk: navigate │ PgUp/PgDn: page │ g/G: first/last ",
            selected_idx + 1,
            page_row_count
        );
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

fn draw_relations_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(tables) = app.current_connection_tables() {
        if tables.is_empty() {
            let empty = Paragraph::new("No tables in this connection")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
            return;
        }

        // Build ER diagram visualization
        let lines = build_er_diagram(tables);
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    } else {
        let empty = Paragraph::new("Select a connection to view relations")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

/// Calculate display width of a string using Unicode Standard Annex #11
fn display_width(s: &str) -> usize {
    s.width()
}

/// Truncate string to fit within max display width
fn truncate_to_width(s: &str, max_width: usize) -> String {
    let mut result = String::new();
    let mut width = 0;
    for c in s.chars() {
        let char_width = c.width().unwrap_or(0);
        if width + char_width > max_width {
            break;
        }
        result.push(c);
        width += char_width;
    }
    result
}

/// Pad string to exact display width
fn pad_to_width(s: &str, target_width: usize) -> String {
    let current_width = display_width(s);
    if current_width >= target_width {
        truncate_to_width(s, target_width)
    } else {
        format!("{}{}", s, " ".repeat(target_width - current_width))
    }
}

/// ER diagram layout configuration
mod er_layout {
    /// Marker width: "[PK]", "[FK]", "[PF]", or "    "
    pub const MARKER_WIDTH: usize = 4;
    /// Space between marker and column name
    pub const MARKER_SPACE: usize = 1;
    /// Column name display width
    pub const NAME_WIDTH: usize = 28;
    /// Data type display width
    pub const TYPE_WIDTH: usize = 16;
    /// Maximum table name width in header
    pub const TABLE_NAME_MAX_WIDTH: usize = 40;
    /// Maximum referenced table name width in FK display
    pub const REF_TABLE_MAX_WIDTH: usize = 30;

    /// Content width inside the box (between │ and │)
    /// = MARKER_WIDTH + MARKER_SPACE + NAME_WIDTH + TYPE_WIDTH
    pub const BOX_CONTENT_WIDTH: usize = MARKER_WIDTH + MARKER_SPACE + NAME_WIDTH + TYPE_WIDTH;

    /// Left border "│ " width
    pub const LEFT_BORDER_WIDTH: usize = 2;
    /// Right border "│" width
    pub const RIGHT_BORDER_WIDTH: usize = 1;

    /// Total box width including borders
    /// Column row: "│ " (2) + content (49) + "│" (1) = 52
    pub const BOX_TOTAL_WIDTH: usize = LEFT_BORDER_WIDTH + BOX_CONTENT_WIDTH + RIGHT_BORDER_WIDTH;

    // Compile-time assertions
    const _: () = assert!(BOX_CONTENT_WIDTH == 49, "BOX_CONTENT_WIDTH should be 49");
    const _: () = assert!(BOX_TOTAL_WIDTH == 52, "BOX_TOTAL_WIDTH should be 52");
    const _: () = assert!(
        MARKER_WIDTH == "[PK]".len(),
        "MARKER_WIDTH must match marker string length"
    );
}

/// Build a text-based ER diagram showing table relationships
fn build_er_diagram(tables: &[Table]) -> Vec<Line<'static>> {
    use er_layout::*;

    let mut lines: Vec<Line<'static>> = Vec::new();

    // Title
    lines.push(Line::from(vec![Span::styled(
        "  Entity Relationship Diagram",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    // Collect all foreign key relationships
    let relationships: Vec<(&Table, &ForeignKey)> = tables
        .iter()
        .flat_map(|t| t.foreign_keys.iter().map(move |fk| (t, fk)))
        .collect();

    // Draw each table as a box
    for table in tables {
        // Table header
        let pk_cols: Vec<&str> = table
            .columns
            .iter()
            .filter(|c| c.is_primary_key)
            .map(|c| c.name.as_str())
            .collect();

        let fk_cols: Vec<&str> = table
            .foreign_keys
            .iter()
            .flat_map(|fk| fk.columns.iter().map(|s| s.as_str()))
            .collect();

        // Truncate table name if too long
        let table_name_display = truncate_to_width(&table.name, TABLE_NAME_MAX_WIDTH);
        let table_name_width = display_width(&table_name_display);
        // Header row: "┌" (1) + "─ " (2) + table_name + " " (1) + dashes + "┐" (1)
        // Total should be BOX_TOTAL_WIDTH to match column rows
        let header_dashes = BOX_TOTAL_WIDTH.saturating_sub(1 + 2 + table_name_width + 1 + 1);

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled("┌─ ", Style::default().fg(Color::Cyan)),
            Span::styled(
                table_name_display,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ", Style::default().fg(Color::Cyan)),
            Span::styled("─".repeat(header_dashes), Style::default().fg(Color::Cyan)),
            Span::styled("┐", Style::default().fg(Color::Cyan)),
        ]));

        // Columns (limited to first 5 for brevity)
        for col in table.columns.iter().take(5) {
            let is_pk = pk_cols.contains(&col.name.as_str());
            let is_fk = fk_cols.contains(&col.name.as_str());

            // Use ASCII markers for consistent width
            let marker = if is_pk && is_fk {
                "[PF]"
            } else if is_pk {
                "[PK]"
            } else if is_fk {
                "[FK]"
            } else {
                "    "
            };

            let col_style = if is_pk {
                Style::default().fg(Color::Yellow)
            } else if is_fk {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::White)
            };

            let col_name_padded = pad_to_width(&col.name, NAME_WIDTH);
            let col_type_padded = pad_to_width(&col.data_type, TYPE_WIDTH);

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("│ ", Style::default().fg(Color::Cyan)),
                Span::raw(marker),
                Span::raw(" "),
                Span::styled(col_name_padded, col_style),
                Span::styled(col_type_padded, Style::default().fg(Color::DarkGray)),
                Span::styled("│", Style::default().fg(Color::Cyan)),
            ]));
        }

        if table.columns.len() > 5 {
            let more_text = format!("     ... and {} more columns", table.columns.len() - 5);
            // Use display_width for accurate width calculation with non-ASCII chars
            let padding = BOX_CONTENT_WIDTH.saturating_sub(display_width(&more_text));
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("│ ", Style::default().fg(Color::Cyan)),
                Span::styled(more_text, Style::default().fg(Color::DarkGray)),
                Span::raw(" ".repeat(padding)),
                Span::styled("│", Style::default().fg(Color::Cyan)),
            ]));
        }

        // Table footer
        // Footer row: "└" (1) + dashes + "┘" (1) = BOX_TOTAL_WIDTH
        // So we need BOX_TOTAL_WIDTH - 2 dashes
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("└{}┘", "─".repeat(BOX_TOTAL_WIDTH - 2)),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        // Draw relationships from this table
        for fk in &table.foreign_keys {
            lines.push(Line::from(vec![
                Span::raw("        "),
                Span::styled("│", Style::default().fg(Color::Green)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("        "),
                Span::styled(
                    format!(
                        "└──> {}.{}",
                        truncate_to_width(&fk.referenced_table, REF_TABLE_MAX_WIDTH),
                        fk.referenced_columns.join(", ")
                    ),
                    Style::default().fg(Color::Green),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::raw("             "),
                Span::styled(fk.name.clone(), Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("             "),
                Span::styled(
                    format!("ON DELETE: {} | ON UPDATE: {}", fk.on_delete, fk.on_update),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        lines.push(Line::from(""));
    }

    // Summary section
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Summary: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!(
                "{} tables, {} relationships",
                tables.len(),
                relationships.len()
            ),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    // Legend
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Legend: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("[PK] Primary Key  ", Style::default().fg(Color::Yellow)),
        Span::styled("[FK] Foreign Key  ", Style::default().fg(Color::Magenta)),
        Span::styled("[PF] Both", Style::default().fg(Color::Yellow)),
    ]));

    lines
}
