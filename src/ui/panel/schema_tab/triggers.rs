//! Triggers sub-tab rendering

use crate::app::App;
use crate::ui::theme;
use ratatui::{
    layout::{Constraint, Rect},
    widgets::{Cell, Paragraph, Row, Table as RatatuiTable},
    Frame,
};

pub fn draw_triggers_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.triggers.is_empty() {
            let empty = Paragraph::new("No triggers defined").style(theme::muted());
            frame.render_widget(empty, area);
            return;
        }

        let vis = &app.column_visibility.triggers;

        // Build visible header cells
        let all_headers = ["Name", "Timing", "Events", "Level", "Function", "Enabled"];
        let visibility_flags = [
            vis.show_name,
            vis.show_timing,
            vis.show_events,
            vis.show_level,
            vis.show_function,
            vis.show_enabled,
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
            .triggers
            .iter()
            .map(|trigger| {
                // Use selected for all timing types (simplified)
                let timing_style = theme::selected();

                let enabled_str = if trigger.enabled { "YES" } else { "NO" };
                // Enabled: header (accent), Disabled: muted
                let enabled_style = if trigger.enabled {
                    theme::header()
                } else {
                    theme::muted()
                };

                let all_cells = [
                    (
                        Cell::from(trigger.name.clone()).style(theme::header()),
                        vis.show_name,
                    ),
                    (
                        Cell::from(trigger.timing.to_string()).style(timing_style),
                        vis.show_timing,
                    ),
                    (
                        Cell::from(trigger.events_display()).style(theme::text()),
                        vis.show_events,
                    ),
                    (
                        Cell::from(trigger.orientation.to_string()).style(theme::muted()),
                        vis.show_level,
                    ),
                    (
                        Cell::from(trigger.function_name.clone()).style(theme::selected()),
                        vis.show_function,
                    ),
                    (
                        Cell::from(enabled_str).style(enabled_style),
                        vis.show_enabled,
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
            (Constraint::Percentage(12), vis.show_timing),
            (Constraint::Percentage(20), vis.show_events),
            (Constraint::Percentage(10), vis.show_level),
            (Constraint::Percentage(25), vis.show_function),
            (Constraint::Percentage(8), vis.show_enabled),
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
        let empty = Paragraph::new("Select a table to view triggers").style(theme::muted());
        frame.render_widget(empty, area);
    }
}
