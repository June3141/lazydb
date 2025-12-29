//! Columns sub-tab rendering

use crate::app::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Paragraph, Row, Table as RatatuiTable},
    Frame,
};

pub fn draw_columns_content(frame: &mut Frame, app: &App, area: Rect) {
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
