//! Indexes sub-tab rendering

use crate::app::App;
use crate::ui::theme;
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Cell, Paragraph, Row, Table as RatatuiTable},
    Frame,
};

pub fn draw_indexes_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.indexes.is_empty() {
            let empty = Paragraph::new("No indexes defined").style(theme::muted());
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
            .map(|(h, _)| Cell::from(*h).style(theme::header()))
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

                // Primary/Unique use header style, others use text
                let type_style = match idx.index_type {
                    crate::model::IndexType::Primary => theme::header(),
                    crate::model::IndexType::Unique => theme::header(),
                    _ => theme::text(),
                };

                let all_cells = [
                    (
                        Cell::from(idx.name.clone()).style(theme::selected()),
                        vis.show_name,
                    ),
                    (
                        Cell::from(idx.index_type.to_string()).style(type_style),
                        vis.show_type,
                    ),
                    (
                        Cell::from(idx.method.to_string()).style(theme::muted()),
                        vis.show_method,
                    ),
                    (
                        Cell::from(columns_str).style(theme::text()),
                        vis.show_columns,
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
            .row_highlight_style(theme::row_highlight());

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view indexes").style(theme::muted());
        frame.render_widget(empty, area);
    }
}
