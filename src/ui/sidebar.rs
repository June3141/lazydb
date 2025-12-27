use crate::app::{App, Focus, SidebarMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

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
        Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let connections_style = if !is_projects_mode {
        Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let indicator = Line::from(vec![
        Span::styled(" Projects ", projects_style),
        Span::styled("│", Style::default().fg(Color::DarkGray)),
        Span::styled(" Connections ", connections_style),
    ]);

    let paragraph = Paragraph::new(indicator);
    frame.render_widget(paragraph, area);
}

/// Draw the Projects list view
fn draw_projects_view(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Sidebar;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
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
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    for (idx, project) in app.projects.iter().enumerate() {
        let is_selected = idx == app.selected_project_idx;

        let style = if is_selected && is_focused {
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
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
            Span::styled(suffix, Style::default().fg(Color::DarkGray)),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No projects",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

/// Draw the Connections list view for a specific project
fn draw_connections_view(frame: &mut Frame, app: &App, area: Rect, proj_idx: usize) {
    let is_focused = app.focus == Focus::Sidebar;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
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
        Style::default().fg(Color::DarkGray),
    )));
    lines.push(Line::from(""));

    let connections = app
        .projects
        .get(proj_idx)
        .map(|p| &p.connections[..])
        .unwrap_or(&[]);

    for (conn_idx, conn) in connections.iter().enumerate() {
        let is_selected_conn =
            conn_idx == app.selected_connection_idx && app.selected_table_idx.is_none();
        let expand_icon = if conn.expanded { "▼" } else { "▶" };

        let conn_style = if is_selected_conn && is_focused {
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else if conn_idx == app.selected_connection_idx {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::White)
        };

        lines.push(Line::from(vec![
            Span::styled(format!("{} ", expand_icon), conn_style),
            Span::styled(&conn.name, conn_style),
        ]));

        if conn.expanded {
            for (table_idx, table) in conn.tables.iter().enumerate() {
                let is_selected_table = conn_idx == app.selected_connection_idx
                    && app.selected_table_idx == Some(table_idx);

                let table_style = if is_selected_table && is_focused {
                    Style::default()
                        .bg(Color::Cyan)
                        .fg(Color::Black)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected_table {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Gray)
                };

                // Icon style: Views are shown in a different color
                let icon_style = if table.table_type.is_view() {
                    if is_selected_table && is_focused {
                        Style::default().bg(Color::Cyan).fg(Color::Black)
                    } else {
                        Style::default().fg(Color::Magenta)
                    }
                } else {
                    table_style
                };

                let prefix = if table_idx == conn.tables.len() - 1 {
                    "  └─ "
                } else {
                    "  ├─ "
                };

                let icon = table.table_type.icon();

                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                    Span::styled(format!("{} ", icon), icon_style),
                    Span::styled(&table.name, table_style),
                ]));
            }
        }
    }

    if connections.is_empty() {
        lines.push(Line::from(Span::styled(
            "No connections",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

pub fn draw_table_summary(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Info ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let lines = if let Some(table) = app.selected_table_info() {
        let pk_name = table
            .columns
            .iter()
            .find(|c| c.is_primary_key)
            .map(|c| c.name.as_str())
            .unwrap_or("-");

        let size_str = format_size(table.size_bytes);

        // Name color varies by table type
        let name_color = if table.table_type.is_view() {
            Color::Magenta
        } else {
            Color::Yellow
        };

        let mut info_lines = vec![
            Line::from(vec![
                Span::styled(
                    format!("{} ", table.table_type.icon()),
                    Style::default().fg(name_color),
                ),
                Span::styled(
                    &table.name,
                    Style::default()
                        .fg(name_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![Span::styled(
                table.table_type.to_string(),
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                format!("{} columns", table.columns.len()),
                Style::default().fg(Color::Gray),
            )]),
        ];

        // Only show row count and PK for non-view tables
        if !table.table_type.is_view() {
            info_lines.push(Line::from(vec![Span::styled(
                format!("{} rows", format_number(table.row_count)),
                Style::default().fg(Color::Gray),
            )]));
            info_lines.push(Line::from(vec![
                Span::styled("PK: ", Style::default().fg(Color::DarkGray)),
                Span::styled(pk_name, Style::default().fg(Color::Cyan)),
            ]));
        }

        info_lines.push(Line::from(vec![Span::styled(
            size_str,
            Style::default().fg(Color::DarkGray),
        )]));

        info_lines
    } else if let Some(conn) = app.selected_connection_info() {
        vec![
            Line::from(vec![Span::styled(
                &conn.name,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(vec![Span::styled(
                &conn.database,
                Style::default().fg(Color::Gray),
            )]),
            Line::from(vec![Span::styled(
                format!("{}:{}", conn.host, conn.port),
                Style::default().fg(Color::DarkGray),
            )]),
            Line::from(vec![Span::styled(
                format!("{} tables", conn.tables.len()),
                Style::default().fg(Color::Gray),
            )]),
        ]
    } else if let Some(project) = app.selected_project_info() {
        // Projects mode - show project info
        if matches!(app.sidebar_mode, SidebarMode::Projects) {
            vec![
                Line::from(vec![Span::styled(
                    &project.name,
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                )]),
                Line::from(vec![Span::styled(
                    format!("{} connections", project.connections.len()),
                    Style::default().fg(Color::Gray),
                )]),
                Line::from(vec![Span::styled(
                    "Press Enter to view",
                    Style::default().fg(Color::DarkGray),
                )]),
            ]
        } else {
            vec![Line::from(Span::styled(
                "Select a connection",
                Style::default().fg(Color::DarkGray),
            ))]
        }
    } else {
        vec![Line::from(Span::styled(
            "No selection",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}
