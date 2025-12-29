use crate::app::{App, Focus, SidebarMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::theme;
use super::utils::{format_number, format_size};

pub fn draw_sidebar(frame: &mut Frame, app: &App, area: Rect) {
    // Split area: mode indicator (top) + content (bottom)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Mode indicator
            Constraint::Min(1),    // Content
        ])
        .split(area);

    draw_mode_indicator(frame, app, chunks[0]);

    match app.sidebar_mode {
        SidebarMode::Projects => draw_projects_view(frame, app, chunks[1]),
        SidebarMode::Connections(proj_idx) => {
            draw_connections_view(frame, app, chunks[1], proj_idx)
        }
    }
}

/// Draw the mode indicator at the top of sidebar
fn draw_mode_indicator(frame: &mut Frame, app: &App, area: Rect) {
    let is_projects_mode = matches!(app.sidebar_mode, SidebarMode::Projects);

    let projects_style = if is_projects_mode {
        theme::focused()
    } else {
        theme::muted()
    };

    let connections_style = if !is_projects_mode {
        theme::focused()
    } else {
        theme::muted()
    };

    let indicator = Line::from(vec![
        Span::styled(" Projects ", projects_style),
        Span::styled("│", theme::muted()),
        Span::styled(" Connections ", connections_style),
    ]);

    let paragraph = Paragraph::new(indicator);
    frame.render_widget(paragraph, area);
}

/// Draw the Projects list view
fn draw_projects_view(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Sidebar;
    let border_style = if is_focused {
        theme::border_focused()
    } else {
        theme::border_inactive()
    };

    let block = Block::default()
        .title(" Projects ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // Help hints for project operations
    lines.push(Line::from(Span::styled(
        "a: add  e: edit  d: delete",
        theme::muted(),
    )));
    lines.push(Line::from(""));

    for (idx, project) in app.projects.iter().enumerate() {
        let is_selected = idx == app.selected_project_idx;

        let style = if is_selected && is_focused {
            theme::focused()
        } else if is_selected {
            theme::selected()
        } else {
            theme::text()
        };

        let conn_count = project.connections.len();
        let suffix = if conn_count == 1 {
            " (1 connection)".to_string()
        } else {
            format!(" ({} connections)", conn_count)
        };

        lines.push(Line::from(vec![
            Span::styled("▶ ", style),
            Span::styled(&project.name, style),
            Span::styled(suffix, theme::muted()),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled("No projects", theme::muted())));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Draw the Connections list view for a specific project
fn draw_connections_view(frame: &mut Frame, app: &App, area: Rect, proj_idx: usize) {
    let is_focused = app.focus == Focus::Sidebar;
    let border_style = if is_focused {
        theme::border_focused()
    } else {
        theme::border_inactive()
    };

    let project_name = app
        .projects
        .get(proj_idx)
        .map(|p| p.name.as_str())
        .unwrap_or("Unknown");

    let block = Block::default()
        .title(format!(" {} ", project_name))
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    // Navigation and operation hints
    lines.push(Line::from(Span::styled(
        "← Back  a: add connection",
        theme::muted(),
    )));
    lines.push(Line::from(""));

    let connections = app
        .projects
        .get(proj_idx)
        .map(|p| &p.connections[..])
        .unwrap_or(&[]);

    for (conn_idx, conn) in connections.iter().enumerate() {
        let is_selected_conn = conn_idx == app.selected_connection_idx
            && app.selected_table_idx.is_none()
            && app.selected_routine_idx.is_none();
        let expand_icon = if conn.expanded { "▼" } else { "▶" };

        let conn_style = if is_selected_conn && is_focused {
            theme::focused()
        } else if conn_idx == app.selected_connection_idx {
            theme::selected()
        } else {
            theme::text()
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{} ", expand_icon), conn_style),
            Span::styled(&conn.name, conn_style),
        ]));

        if conn.expanded {
            let total_items = conn.tables.len() + conn.routines.len();
            let mut item_idx = 0;

            // Draw tables
            for (table_idx, table) in conn.tables.iter().enumerate() {
                let is_selected_table = conn_idx == app.selected_connection_idx
                    && app.selected_table_idx == Some(table_idx);

                let table_style = if is_selected_table && is_focused {
                    theme::focused()
                } else if is_selected_table {
                    // Selected table highlighted in white+bold when not focused
                    theme::text().add_modifier(Modifier::BOLD)
                } else {
                    theme::muted()
                };

                // Icon style matches table style for consistency
                let icon_style = table_style;

                item_idx += 1;
                let prefix = if item_idx == total_items {
                    "  └─ "
                } else {
                    "  ├─ "
                };

                let icon = table.table_type.icon();

                // Apply style to entire line for proper background highlighting
                let line = Line::from(vec![
                    Span::styled(prefix, theme::muted()),
                    Span::styled(format!("{} ", icon), icon_style),
                    Span::styled(&table.name, table_style),
                ]);

                // If table is selected and focused, apply background to entire line
                if is_selected_table && is_focused {
                    lines.push(line.style(theme::focused()));
                } else {
                    lines.push(line);
                }
            }

            // Draw routines (stored procedures and functions)
            for (routine_idx, routine) in conn.routines.iter().enumerate() {
                let is_selected_routine = conn_idx == app.selected_connection_idx
                    && app.selected_routine_idx == Some(routine_idx);

                let routine_style = if is_selected_routine && is_focused {
                    theme::focused()
                } else if is_selected_routine {
                    theme::text().add_modifier(Modifier::BOLD)
                } else {
                    theme::muted()
                };

                let icon_style = routine_style;

                item_idx += 1;
                let prefix = if item_idx == total_items {
                    "  └─ "
                } else {
                    "  ├─ "
                };

                // Icon: ƒ for functions, ⚙ for procedures
                let icon = if routine.is_function() { "ƒ" } else { "⚙" };

                let line = Line::from(vec![
                    Span::styled(prefix, theme::muted()),
                    Span::styled(format!("{} ", icon), icon_style),
                    Span::styled(&routine.name, routine_style),
                ]);

                if is_selected_routine && is_focused {
                    lines.push(line.style(theme::focused()));
                } else {
                    lines.push(line);
                }
            }
        }
    }

    if connections.is_empty() {
        lines.push(Line::from(Span::styled("No connections", theme::muted())));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

pub fn draw_table_summary(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .border_style(theme::border_inactive());

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let lines = if let Some(routine) = app.selected_routine_info() {
        // Routine selected - show routine info
        let routine_type = if routine.is_function() {
            "Function"
        } else {
            "Procedure"
        };
        let icon = if routine.is_function() { "ƒ" } else { "⚙" };

        let mut info_lines = vec![
            Line::from(vec![
                Span::styled(format!("{} ", icon), theme::header()),
                Span::styled(
                    &routine.name,
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![Span::styled(routine_type, theme::muted())]),
        ];

        // Show schema
        if !routine.schema.is_empty() {
            info_lines.push(Line::from(vec![
                Span::styled("Schema: ", theme::muted()),
                Span::styled(&routine.schema, theme::text()),
            ]));
        }

        // Show return type for functions
        if routine.is_function() {
            if let Some(return_type) = &routine.return_type {
                info_lines.push(Line::from(vec![
                    Span::styled("Returns: ", theme::muted()),
                    Span::styled(return_type, theme::selected()),
                ]));
            }
        }

        // Show parameter count
        let param_count = routine.parameters.len();
        if param_count > 0 {
            info_lines.push(Line::from(vec![Span::styled(
                format!("{} parameter(s)", param_count),
                theme::muted(),
            )]));
        } else {
            info_lines.push(Line::from(vec![Span::styled(
                "No parameters",
                theme::muted(),
            )]));
        }

        // Show language
        if !routine.language.is_empty() {
            info_lines.push(Line::from(vec![
                Span::styled("Language: ", theme::muted()),
                Span::styled(&routine.language, theme::text()),
            ]));
        }

        info_lines
    } else if let Some(table) = app.selected_table_info() {
        let pk_name = table
            .columns
            .iter()
            .find(|c| c.is_primary_key)
            .map(|c| c.name.as_str())
            .unwrap_or("-");

        let size_str = format_size(table.size_bytes);

        let mut info_lines = vec![
            Line::from(vec![
                Span::styled(format!("{} ", table.table_type.icon()), theme::header()),
                Span::styled(
                    &table.name,
                    Style::default()
                        .fg(theme::ACCENT)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![Span::styled(
                table.table_type.to_string(),
                theme::muted(),
            )]),
            Line::from(vec![Span::styled(
                format!("{} columns", table.columns.len()),
                theme::muted(),
            )]),
        ];

        // Only show row count and PK for non-view tables
        if !table.table_type.is_view() {
            info_lines.push(Line::from(vec![Span::styled(
                format!("{} rows", format_number(table.row_count)),
                theme::muted(),
            )]));
            info_lines.push(Line::from(vec![
                Span::styled("PK: ", theme::muted()),
                Span::styled(pk_name, theme::selected()),
            ]));
        }

        info_lines.push(Line::from(vec![Span::styled(size_str, theme::muted())]));

        info_lines
    } else if let Some(conn) = app.selected_connection_info() {
        vec![
            Line::from(vec![Span::styled(
                &conn.name,
                Style::default()
                    .fg(theme::PRIMARY)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(&conn.database, theme::muted())]),
            Line::from(vec![Span::styled(
                format!("{}:{}", conn.host, conn.port),
                theme::muted(),
            )]),
            Line::from(vec![Span::styled(
                format!("{} tables", conn.tables.len()),
                theme::muted(),
            )]),
        ]
    } else if let Some(project) = app.selected_project_info() {
        // Projects mode - show project info
        if matches!(app.sidebar_mode, SidebarMode::Projects) {
            vec![
                Line::from(vec![Span::styled(
                    &project.name,
                    Style::default()
                        .fg(theme::PRIMARY)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![Span::styled(
                    format!("{} connections", project.connections.len()),
                    theme::muted(),
                )]),
                Line::from(vec![Span::styled("Press Enter to view", theme::muted())]),
            ]
        } else {
            vec![Line::from(Span::styled(
                "Select a connection",
                theme::muted(),
            ))]
        }
    } else {
        vec![Line::from(Span::styled("No selection", theme::muted()))]
    };

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}
