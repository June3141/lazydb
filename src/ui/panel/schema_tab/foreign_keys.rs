//! Foreign Keys sub-tab rendering

use crate::app::App;
use crate::ui::theme;
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Cell, Paragraph, Row, Table as RatatuiTable},
    Frame,
};

pub fn draw_foreign_keys_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.foreign_keys.is_empty() {
            let empty = Paragraph::new("No foreign keys defined").style(theme::muted());
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
            .map(|(h, _)| Cell::from(*h).style(theme::header()))
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
                        Cell::from(fk.name.clone()).style(theme::selected()),
                        vis.show_name,
                    ),
                    (Cell::from(columns_str).style(theme::text()), vis.show_column),
                    (
                        Cell::from(ref_str).style(theme::header()),
                        vis.show_references,
                    ),
                    (
                        Cell::from(fk.on_delete.to_string()).style(theme::muted()),
                        vis.show_on_delete,
                    ),
                    (
                        Cell::from(fk.on_update.to_string()).style(theme::muted()),
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
            .row_highlight_style(theme::row_highlight());

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view foreign keys").style(theme::muted());
        frame.render_widget(empty, area);
    }
}
