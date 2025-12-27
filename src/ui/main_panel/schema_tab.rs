//! Schema tab rendering (Columns, Indexes, Foreign Keys, Constraints)

use crate::app::{App, SchemaSubTab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Cell, Paragraph, Row, Table as RatatuiTable, Tabs},
    Frame,
};

pub fn draw_schema_content(frame: &mut Frame, app: &App, area: Rect) {
    // Split for sub-tabs and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Sub-tabs
            Constraint::Min(1),    // Content
        ])
        .split(area);

    // Check if selected table is a view (to show Definition tab)
    let is_view = app
        .selected_table_info()
        .map(|t| t.table_type.is_view())
        .unwrap_or(false);

    // Draw sub-tabs (Definition tab only shown for views)
    let sub_tab_titles: Vec<&str> = if is_view {
        vec![
            "Columns [1]",
            "Indexes [2]",
            "Foreign Keys [3]",
            "Constraints [4]",
            "Definition [5]",
        ]
    } else {
        vec![
            "Columns [1]",
            "Indexes [2]",
            "Foreign Keys [3]",
            "Constraints [4]",
        ]
    };

    let selected_sub_tab = match app.schema_sub_tab {
        SchemaSubTab::Columns => 0,
        SchemaSubTab::Indexes => 1,
        SchemaSubTab::ForeignKeys => 2,
        SchemaSubTab::Constraints => 3,
        SchemaSubTab::Definition => {
            if is_view {
                4
            } else {
                0
            }
        }
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
    // For non-views with Definition tab selected, fall back to Columns content
    let effective_sub_tab = if !is_view && app.schema_sub_tab == SchemaSubTab::Definition {
        SchemaSubTab::Columns
    } else {
        app.schema_sub_tab
    };

    match effective_sub_tab {
        SchemaSubTab::Columns => draw_columns_content(frame, app, chunks[1]),
        SchemaSubTab::Indexes => draw_indexes_content(frame, app, chunks[1]),
        SchemaSubTab::ForeignKeys => draw_foreign_keys_content(frame, app, chunks[1]),
        SchemaSubTab::Constraints => draw_constraints_content(frame, app, chunks[1]),
        SchemaSubTab::Definition => draw_definition_content(frame, app, chunks[1]),
    }
}

fn draw_columns_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        let vis = &app.column_visibility.columns;

        // Build visible header cells
        let all_headers = ["", "Name", "Type", "Null", "Default", "Key"];
        let visibility_flags = [
            vis.show_icon,
            vis.show_name,
            vis.show_type,
            vis.show_nullable,
            vis.show_default,
            vis.show_key,
        ];

        let header_cells: Vec<Cell> = all_headers
            .iter()
            .zip(visibility_flags.iter())
            .filter(|(_, &visible)| visible)
            .map(|(h, _)| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect();
        let header = Row::new(header_cells).height(1);

        // Create rows with visibility filtering
        let rows: Vec<Row> = table
            .columns
            .iter()
            .map(|col| {
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

                let all_cells = [
                    (Cell::from(pk_marker), vis.show_icon),
                    (
                        Cell::from(col.name.clone()).style(name_style),
                        vis.show_name,
                    ),
                    (
                        Cell::from(col.data_type.clone()).style(Style::default().fg(Color::Cyan)),
                        vis.show_type,
                    ),
                    (
                        Cell::from(null_str).style(if col.is_nullable {
                            Style::default().fg(Color::DarkGray)
                        } else {
                            Style::default().fg(Color::Red)
                        }),
                        vis.show_nullable,
                    ),
                    (
                        Cell::from(default_str).style(Style::default().fg(Color::Green)),
                        vis.show_default,
                    ),
                    (
                        Cell::from(key_info).style(Style::default().fg(Color::Magenta)),
                        vis.show_key,
                    ),
                ];

                let visible_cells: Vec<Cell> = all_cells
                    .into_iter()
                    .filter(|(_, visible)| *visible)
                    .map(|(cell, _)| cell)
                    .collect();

                Row::new(visible_cells).height(1)
            })
            .collect();

        // Build widths based on visibility
        let all_widths = [
            (Constraint::Length(3), vis.show_icon),
            (Constraint::Percentage(25), vis.show_name),
            (Constraint::Percentage(25), vis.show_type),
            (Constraint::Length(5), vis.show_nullable),
            (Constraint::Percentage(25), vis.show_default),
            (Constraint::Length(5), vis.show_key),
        ];

        let widths: Vec<Constraint> = all_widths
            .into_iter()
            .filter(|(_, visible)| *visible)
            .map(|(w, _)| w)
            .collect();

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

        let vis = &app.column_visibility.indexes;

        // Build visible header cells
        let all_headers = ["Name", "Type", "Method", "Columns"];
        let visibility_flags = [
            vis.show_name,
            vis.show_type,
            vis.show_method,
            vis.show_columns,
        ];

        let header_cells: Vec<Cell> = all_headers
            .iter()
            .zip(visibility_flags.iter())
            .filter(|(_, &visible)| visible)
            .map(|(h, _)| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect();
        let header = Row::new(header_cells).height(1);

        // Create rows with visibility filtering
        let rows: Vec<Row> = table
            .indexes
            .iter()
            .map(|idx| {
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

                let all_cells = [
                    (
                        Cell::from(idx.name.clone()).style(Style::default().fg(Color::Cyan)),
                        vis.show_name,
                    ),
                    (
                        Cell::from(idx.index_type.to_string()).style(type_style),
                        vis.show_type,
                    ),
                    (
                        Cell::from(idx.method.to_string())
                            .style(Style::default().fg(Color::DarkGray)),
                        vis.show_method,
                    ),
                    (Cell::from(columns_str), vis.show_columns),
                ];

                let visible_cells: Vec<Cell> = all_cells
                    .into_iter()
                    .filter(|(_, visible)| *visible)
                    .map(|(cell, _)| cell)
                    .collect();

                Row::new(visible_cells).height(1)
            })
            .collect();

        // Build widths based on visibility
        let all_widths = [
            (Constraint::Percentage(30), vis.show_name),
            (Constraint::Percentage(15), vis.show_type),
            (Constraint::Percentage(15), vis.show_method),
            (Constraint::Percentage(40), vis.show_columns),
        ];

        let widths: Vec<Constraint> = all_widths
            .into_iter()
            .filter(|(_, visible)| *visible)
            .map(|(w, _)| w)
            .collect();

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

        let vis = &app.column_visibility.foreign_keys;

        // Build visible header cells
        let all_headers = ["Name", "Column", "References", "ON DELETE", "ON UPDATE"];
        let visibility_flags = [
            vis.show_name,
            vis.show_column,
            vis.show_references,
            vis.show_on_delete,
            vis.show_on_update,
        ];

        let header_cells: Vec<Cell> = all_headers
            .iter()
            .zip(visibility_flags.iter())
            .filter(|(_, &visible)| visible)
            .map(|(h, _)| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect();
        let header = Row::new(header_cells).height(1);

        // Create rows with visibility filtering
        let rows: Vec<Row> = table
            .foreign_keys
            .iter()
            .map(|fk| {
                let columns_str = fk.columns.join(", ");
                let ref_str = format!(
                    "{}.{}",
                    fk.referenced_table,
                    fk.referenced_columns.join(", ")
                );

                let all_cells = [
                    (
                        Cell::from(fk.name.clone()).style(Style::default().fg(Color::Cyan)),
                        vis.show_name,
                    ),
                    (Cell::from(columns_str), vis.show_column),
                    (
                        Cell::from(ref_str).style(Style::default().fg(Color::Green)),
                        vis.show_references,
                    ),
                    (
                        Cell::from(fk.on_delete.to_string()).style(Style::default().fg(Color::Red)),
                        vis.show_on_delete,
                    ),
                    (
                        Cell::from(fk.on_update.to_string())
                            .style(Style::default().fg(Color::Magenta)),
                        vis.show_on_update,
                    ),
                ];

                let visible_cells: Vec<Cell> = all_cells
                    .into_iter()
                    .filter(|(_, visible)| *visible)
                    .map(|(cell, _)| cell)
                    .collect();

                Row::new(visible_cells).height(1)
            })
            .collect();

        // Build widths based on visibility
        let all_widths = [
            (Constraint::Percentage(25), vis.show_name),
            (Constraint::Percentage(15), vis.show_column),
            (Constraint::Percentage(25), vis.show_references),
            (Constraint::Percentage(17), vis.show_on_delete),
            (Constraint::Percentage(18), vis.show_on_update),
        ];

        let widths: Vec<Constraint> = all_widths
            .into_iter()
            .filter(|(_, visible)| *visible)
            .map(|(w, _)| w)
            .collect();

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

        let vis = &app.column_visibility.constraints;

        // Build visible header cells
        let all_headers = ["Name", "Type", "Columns", "Definition"];
        let visibility_flags = [
            vis.show_name,
            vis.show_type,
            vis.show_columns,
            vis.show_definition,
        ];

        let header_cells: Vec<Cell> = all_headers
            .iter()
            .zip(visibility_flags.iter())
            .filter(|(_, &visible)| visible)
            .map(|(h, _)| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            })
            .collect();
        let header = Row::new(header_cells).height(1);

        // Create rows with visibility filtering
        let rows: Vec<Row> = table
            .constraints
            .iter()
            .map(|c| {
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
                    crate::model::ConstraintType::Default => {
                        Style::default().fg(Color::LightYellow)
                    }
                    crate::model::ConstraintType::Exclusion => {
                        Style::default().fg(Color::LightMagenta)
                    }
                };

                let all_cells = [
                    (
                        Cell::from(c.name.clone()).style(Style::default().fg(Color::Cyan)),
                        vis.show_name,
                    ),
                    (
                        Cell::from(c.constraint_type.to_string()).style(type_style),
                        vis.show_type,
                    ),
                    (Cell::from(columns_str), vis.show_columns),
                    (
                        Cell::from(def_str).style(Style::default().fg(Color::DarkGray)),
                        vis.show_definition,
                    ),
                ];

                let visible_cells: Vec<Cell> = all_cells
                    .into_iter()
                    .filter(|(_, visible)| *visible)
                    .map(|(cell, _)| cell)
                    .collect();

                Row::new(visible_cells).height(1)
            })
            .collect();

        // Build widths based on visibility
        let all_widths = [
            (Constraint::Percentage(25), vis.show_name),
            (Constraint::Percentage(15), vis.show_type),
            (Constraint::Percentage(20), vis.show_columns),
            (Constraint::Percentage(40), vis.show_definition),
        ];

        let widths: Vec<Constraint> = all_widths
            .into_iter()
            .filter(|(_, visible)| *visible)
            .map(|(w, _)| w)
            .collect();

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

fn draw_definition_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if !table.table_type.is_view() {
            let msg =
                Paragraph::new("Definition is only available for Views and Materialized Views")
                    .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(msg, area);
            return;
        }

        if let Some(definition) = &table.view_definition {
            // Display the view definition (SELECT statement) with syntax-like coloring
            let lines: Vec<Line> = definition
                .lines()
                .map(|line| {
                    // Simple SQL keyword highlighting
                    let styled_line = highlight_sql_line(line);
                    Line::from(styled_line)
                })
                .collect();

            let paragraph = Paragraph::new(lines)
                .style(Style::default().fg(Color::White))
                .wrap(ratatui::widgets::Wrap { trim: false });
            frame.render_widget(paragraph, area);
        } else {
            let empty = Paragraph::new("View definition not loaded")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
        }
    } else {
        let empty = Paragraph::new("Select a view to see its definition")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

/// Simple SQL keyword highlighting for view definitions
/// Uses string slices to avoid unnecessary allocations
fn highlight_sql_line(line: &str) -> Vec<Span<'static>> {
    const KEYWORDS: &[&str] = &[
        "SELECT",
        "FROM",
        "WHERE",
        "AND",
        "OR",
        "NOT",
        "IN",
        "IS",
        "NULL",
        "JOIN",
        "LEFT",
        "RIGHT",
        "INNER",
        "OUTER",
        "ON",
        "AS",
        "ORDER",
        "BY",
        "GROUP",
        "HAVING",
        "LIMIT",
        "OFFSET",
        "UNION",
        "ALL",
        "DISTINCT",
        "CREATE",
        "VIEW",
        "MATERIALIZED",
        "WITH",
        "CASE",
        "WHEN",
        "THEN",
        "ELSE",
        "END",
        "TRUE",
        "FALSE",
        "LIKE",
        "ILIKE",
        "BETWEEN",
        "EXISTS",
        "CAST",
        "COALESCE",
    ];

    let mut spans = Vec::new();
    let mut pos = 0;
    let bytes = line.as_bytes();

    while pos < line.len() {
        let remaining = &line[pos..];

        // Check for string literals (single or double quotes)
        if remaining.starts_with('\'') || remaining.starts_with('"') {
            let quote_char = remaining.chars().next().unwrap();
            let mut end_pos = 1;
            while end_pos < remaining.len() {
                if remaining[end_pos..].starts_with(quote_char) {
                    end_pos += 1;
                    break;
                }
                end_pos += 1;
            }
            spans.push(Span::styled(
                remaining[..end_pos].to_string(),
                Style::default().fg(Color::Green),
            ));
            pos += end_pos;
            continue;
        }

        // Check for keywords (case-insensitive)
        let mut found = false;
        for keyword in KEYWORDS {
            if remaining.len() >= keyword.len()
                && remaining[..keyword.len()].eq_ignore_ascii_case(keyword)
            {
                // Verify it's a complete word
                let next_pos = pos + keyword.len();
                let is_word_boundary = next_pos >= line.len()
                    || !bytes[next_pos].is_ascii_alphanumeric() && bytes[next_pos] != b'_';

                if is_word_boundary {
                    spans.push(Span::styled(
                        remaining[..keyword.len()].to_string(),
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ));
                    pos += keyword.len();
                    found = true;
                    break;
                }
            }
        }

        if !found {
            // Handle single character
            let ch = remaining.chars().next().unwrap();
            let style = if ch.is_ascii_digit() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };
            spans.push(Span::styled(ch.to_string(), style));
            pos += ch.len_utf8();
        }
    }

    spans
}

#[cfg(test)]
mod highlight_tests {
    use super::*;

    fn get_span_content(spans: &[Span]) -> Vec<String> {
        spans.iter().map(|s| s.content.to_string()).collect()
    }

    #[test]
    fn test_highlight_sql_keyword() {
        let spans = highlight_sql_line("SELECT");
        assert_eq!(get_span_content(&spans), vec!["SELECT"]);
    }

    #[test]
    fn test_highlight_sql_keyword_lowercase() {
        let spans = highlight_sql_line("select");
        assert_eq!(get_span_content(&spans), vec!["select"]);
    }

    #[test]
    fn test_highlight_sql_keyword_mixed_case() {
        let spans = highlight_sql_line("Select");
        assert_eq!(get_span_content(&spans), vec!["Select"]);
    }

    #[test]
    fn test_highlight_partial_keyword_not_matched() {
        // "SELECTING" should not highlight "SELECT" as a keyword
        let spans = highlight_sql_line("SELECTING");
        let content = get_span_content(&spans);
        // Should be individual characters since SELECTING is not a keyword
        assert_eq!(content.join(""), "SELECTING");
        assert!(content.len() > 1); // Not a single span
    }

    #[test]
    fn test_highlight_string_literal_single_quote() {
        let spans = highlight_sql_line("'hello world'");
        assert_eq!(get_span_content(&spans), vec!["'hello world'"]);
    }

    #[test]
    fn test_highlight_string_literal_double_quote() {
        let spans = highlight_sql_line("\"column_name\"");
        assert_eq!(get_span_content(&spans), vec!["\"column_name\""]);
    }

    #[test]
    fn test_highlight_numeric_literal() {
        let spans = highlight_sql_line("123");
        assert_eq!(get_span_content(&spans), vec!["1", "2", "3"]);
    }

    #[test]
    fn test_highlight_mixed_sql() {
        let spans = highlight_sql_line("SELECT * FROM users");
        let content = get_span_content(&spans);
        assert!(content.contains(&"SELECT".to_string()));
        assert!(content.contains(&"FROM".to_string()));
    }

    #[test]
    fn test_highlight_keyword_with_underscore_suffix() {
        // "SELECT_QUERY" should not match SELECT as keyword
        let spans = highlight_sql_line("SELECT_QUERY");
        let content = get_span_content(&spans);
        assert_eq!(content.join(""), "SELECT_QUERY");
        assert!(content.len() > 1);
    }
}
