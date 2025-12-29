//! Routine detail view for stored procedures and functions
//!
//! Displays routine definition, parameters, and return type information.

use crate::app::App;
use crate::model::schema::Routine;
use crate::ui::theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};

/// Draw routine details when a routine is selected
pub fn draw_routine_content(frame: &mut Frame, app: &App, area: Rect) {
    let Some(routine) = app.selected_routine_info() else {
        let msg = Paragraph::new("No routine selected")
            .style(theme::muted())
            .block(Block::default().borders(Borders::NONE));
        frame.render_widget(msg, area);
        return;
    };

    // Calculate header height dynamically based on content
    // Base: 2 lines (name + type)
    let mut header_height: u16 = 2;
    if routine.is_function() {
        header_height += 1; // Returns line
    }
    if !routine.language.is_empty() {
        header_height += 1; // Language line
    }

    // Split area: header + parameters + definition
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(header_height),
            Constraint::Length(8),  // Parameters table
            Constraint::Min(5),     // Definition
        ])
        .split(area);

    draw_routine_header(frame, routine, chunks[0]);
    draw_parameters_section(frame, routine, chunks[1]);
    draw_definition_section(frame, routine, chunks[2]);
}

/// Draw routine header with name, type, and return type
fn draw_routine_header(frame: &mut Frame, routine: &Routine, area: Rect) {
    let routine_type = if routine.is_function() {
        "Function"
    } else {
        "Procedure"
    };
    let icon = if routine.is_function() { "ƒ" } else { "⚙" };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(format!("{} ", icon), theme::header()),
            Span::styled(
                routine.qualified_name(),
                theme::text().add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("Type: ", theme::muted()),
            Span::styled(routine_type, theme::text()),
        ]),
    ];

    // Return type (for functions)
    if routine.is_function() {
        let return_type = routine
            .return_type
            .as_deref()
            .unwrap_or("unknown");
        lines.push(Line::from(vec![
            Span::styled("Returns: ", theme::muted()),
            Span::styled(return_type, theme::selected()),
        ]));
    }

    // Language
    if !routine.language.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Language: ", theme::muted()),
            Span::styled(&routine.language, theme::text()),
        ]));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

/// Draw parameters table
fn draw_parameters_section(frame: &mut Frame, routine: &Routine, area: Rect) {
    let block = Block::default()
        .title(" Parameters ")
        .borders(Borders::ALL)
        .border_style(theme::border_inactive());

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    if routine.parameters.is_empty() {
        let msg = Paragraph::new("No parameters").style(theme::muted());
        frame.render_widget(msg, inner_area);
        return;
    }

    // Build table rows
    let rows: Vec<Row> = routine
        .parameters
        .iter()
        .map(|param| {
            let mode_str = param.mode.to_string();
            let default_str = param
                .default_value
                .as_ref()
                .map(|d| format!(" = {}", d))
                .unwrap_or_default();

            Row::new(vec![
                param.name.clone(),
                param.data_type.clone(),
                mode_str,
                default_str,
            ])
            .style(theme::text())
        })
        .collect();

    let header = Row::new(vec!["Name", "Type", "Mode", "Default"])
        .style(theme::header())
        .bottom_margin(1);

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(30),
        Constraint::Percentage(15),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(theme::focused());

    frame.render_widget(table, inner_area);
}

/// Draw routine definition (source code)
fn draw_definition_section(frame: &mut Frame, routine: &Routine, area: Rect) {
    let block = Block::default()
        .title(" Definition ")
        .borders(Borders::ALL)
        .border_style(theme::border_inactive());

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let definition = routine
        .definition
        .as_deref()
        .unwrap_or("-- Definition not available");

    // Split definition into lines for proper display
    let lines: Vec<Line> = definition
        .lines()
        .map(|line| Line::from(Span::styled(line, theme::text())))
        .collect();

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}
