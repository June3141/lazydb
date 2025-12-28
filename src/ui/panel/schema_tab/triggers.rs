//! Triggers sub-tab rendering

use crate::app::App;
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    widgets::{Cell, Paragraph, Row, Table as RatatuiTable},
    Frame,
};

pub fn draw_triggers_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.triggers.is_empty() {
            let empty =
                Paragraph::new("No triggers defined").style(Style::default().fg(Color::DarkGray));
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
            .triggers
            .iter()
            .map(|trigger| {
                let timing_style = match trigger.timing {
                    crate::model::TriggerTiming::Before => Style::default().fg(Color::Cyan),
                    crate::model::TriggerTiming::After => Style::default().fg(Color::Green),
                    crate::model::TriggerTiming::InsteadOf => Style::default().fg(Color::Magenta),
                };

                let enabled_str = if trigger.enabled { "YES" } else { "NO" };
                let enabled_style = if trigger.enabled {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::Red)
                };

                let all_cells = [
                    (
                        Cell::from(trigger.name.clone()).style(Style::default().fg(Color::Yellow)),
                        vis.show_name,
                    ),
                    (
                        Cell::from(trigger.timing.to_string()).style(timing_style),
                        vis.show_timing,
                    ),
                    (
                        Cell::from(trigger.events_display())
                            .style(Style::default().fg(Color::White)),
                        vis.show_events,
                    ),
                    (
                        Cell::from(trigger.orientation.to_string())
                            .style(Style::default().fg(Color::DarkGray)),
                        vis.show_level,
                    ),
                    (
                        Cell::from(trigger.function_name.clone())
                            .style(Style::default().fg(Color::Cyan)),
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
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view triggers")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}
