//! Constraints sub-tab rendering

use crate::app::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Paragraph, Row, Table as RatatuiTable},
    Frame,
};

pub fn draw_constraints_content(frame: &mut Frame, app: &App, area: Rect) {
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
