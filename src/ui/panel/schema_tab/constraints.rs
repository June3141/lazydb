//! Constraints sub-tab rendering

use crate::app::App;
use crate::ui::theme;
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Cell, Paragraph, Row, Table as RatatuiTable},
    Frame,
};

pub fn draw_constraints_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.constraints.is_empty() {
            let empty = Paragraph::new("No constraints defined").style(theme::muted());
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
            .map(|(h, _)| Cell::from(*h).style(theme::header()))
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

                // Simplified: PrimaryKey/Unique use header style, others use selected/muted
                let type_style = match c.constraint_type {
                    crate::model::ConstraintType::PrimaryKey => theme::header(),
                    crate::model::ConstraintType::Unique => theme::header(),
                    crate::model::ConstraintType::ForeignKey => theme::selected(),
                    _ => theme::text(),
                };

                let all_cells = [
                    (
                        Cell::from(c.name.clone()).style(theme::selected()),
                        vis.show_name,
                    ),
                    (
                        Cell::from(c.constraint_type.to_string()).style(type_style),
                        vis.show_type,
                    ),
                    (
                        Cell::from(columns_str).style(theme::text()),
                        vis.show_columns,
                    ),
                    (
                        Cell::from(def_str).style(theme::muted()),
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
            .row_highlight_style(theme::row_highlight());

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view constraints").style(theme::muted());
        frame.render_widget(empty, area);
    }
}
