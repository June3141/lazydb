//! Relations tab rendering with ER diagram

use crate::app::App;
use crate::model::{ForeignKey, Table};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
    Frame,
};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn draw_relations_content(frame: &mut Frame, app: &App, area: Rect) {
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
