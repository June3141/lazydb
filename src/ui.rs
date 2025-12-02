use crate::app::{App, Focus, MainPanelTab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};

pub fn draw(frame: &mut Frame, app: &App) {
    let size = frame.area();

    // Main layout: Sidebar | Main area
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(28), // Sidebar width
            Constraint::Min(40),    // Main area
        ])
        .split(size);

    // Sidebar layout: Connections tree | Table info summary
    let sidebar_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(8),     // Connections tree
            Constraint::Length(7),  // Table info summary (compact)
        ])
        .split(main_chunks[0]);

    // Main area layout: Query editor | Main panel | Status
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),  // Query editor
            Constraint::Min(10),    // Main panel (Schema/Data)
            Constraint::Length(3),  // Status bar
        ])
        .split(main_chunks[1]);

    // Bottom layout: Help bar
    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1), // Help bar
        ])
        .split(size);

    draw_connections_tree(frame, app, sidebar_chunks[0]);
    draw_table_summary(frame, app, sidebar_chunks[1]);
    draw_query_editor(frame, app, right_chunks[0]);
    draw_main_panel(frame, app, right_chunks[1]);
    draw_status_bar(frame, app, right_chunks[2]);
    draw_help_bar(frame, bottom_chunks[1]);
}

fn draw_connections_tree(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::Sidebar;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" Connections ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_area = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    for (conn_idx, conn) in app.connections.iter().enumerate() {
        let is_selected_conn = conn_idx == app.selected_connection && app.selected_table.is_none();
        let expand_icon = if conn.expanded { "▼" } else { "▶" };

        let conn_style = if is_selected_conn && is_focused {
            Style::default()
                .bg(Color::Cyan)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD)
        } else if conn_idx == app.selected_connection {
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
                let is_selected_table = conn_idx == app.selected_connection
                    && app.selected_table == Some(table_idx);

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

                let prefix = if table_idx == conn.tables.len() - 1 {
                    "  └─ "
                } else {
                    "  ├─ "
                };

                lines.push(Line::from(vec![
                    Span::styled(prefix, Style::default().fg(Color::DarkGray)),
                    Span::styled(&table.name, table_style),
                ]));
            }
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

fn draw_table_summary(frame: &mut Frame, app: &App, area: Rect) {
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

        vec![
            Line::from(vec![
                Span::styled(&table.name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(format!("{} columns", table.columns.len()), Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled(format!("{} rows", format_number(table.row_count)), Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("PK: ", Style::default().fg(Color::DarkGray)),
                Span::styled(pk_name, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled(size_str, Style::default().fg(Color::DarkGray)),
            ]),
        ]
    } else if let Some(conn) = app.selected_connection_info() {
        vec![
            Line::from(vec![
                Span::styled(&conn.name, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::styled(&conn.database, Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled(format!("{}:{}", conn.host, conn.port), Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(vec![
                Span::styled(format!("{} tables", conn.tables.len()), Style::default().fg(Color::Gray)),
            ]),
        ]
    } else {
        vec![Line::from(Span::styled(
            "No selection",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner_area);
}

fn draw_query_editor(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::QueryEditor;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let block = Block::default()
        .title(" SQL Query ")
        .borders(Borders::ALL)
        .border_style(border_style);

    let query_text = Paragraph::new(app.query.as_str())
        .block(block)
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: false });

    frame.render_widget(query_text, area);
}

fn draw_main_panel(frame: &mut Frame, app: &App, area: Rect) {
    let is_focused = app.focus == Focus::MainPanel;
    let border_style = if is_focused {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    // Split area for tabs and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Tabs
            Constraint::Min(1),    // Content
        ])
        .split(area);

    // Draw tabs
    let tab_titles = vec!["Schema [s]", "Data [d]"];
    let selected_tab = match app.main_panel_tab {
        MainPanelTab::Schema => 0,
        MainPanelTab::Data => 1,
    };

    let tabs = Tabs::new(tab_titles)
        .select(selected_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");

    frame.render_widget(tabs, chunks[0]);

    // Draw content based on selected tab
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner_area = content_block.inner(chunks[1]);
    frame.render_widget(content_block, chunks[1]);

    match app.main_panel_tab {
        MainPanelTab::Schema => draw_schema_content(frame, app, inner_area),
        MainPanelTab::Data => draw_data_content(frame, app, inner_area),
    }
}

fn draw_schema_content(frame: &mut Frame, app: &App, area: Rect) {
    let lines = if let Some(table) = app.selected_table_info() {
        table
            .columns
            .iter()
            .map(|col| {
                let pk_marker = if col.is_primary_key { "🔑 " } else { "   " };
                let name_style = if col.is_primary_key {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };

                Line::from(vec![
                    Span::styled(pk_marker, Style::default()),
                    Span::styled(
                        format!("{:<20}", &col.name),
                        name_style,
                    ),
                    Span::styled(&col.data_type, Style::default().fg(Color::DarkGray)),
                ])
            })
            .collect()
    } else {
        vec![Line::from(Span::styled(
            "Select a table to view schema",
            Style::default().fg(Color::DarkGray),
        ))]
    };

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn draw_data_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(result) = &app.result {
        // Create header row
        let header_cells = result.columns.iter().map(|col| {
            Cell::from(col.clone()).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells).height(1);

        // Create data rows
        let rows = result.rows.iter().map(|row_data| {
            let cells = row_data.iter().map(|cell| {
                Cell::from(cell.clone()).style(Style::default().fg(Color::White))
            });
            Row::new(cells).height(1)
        });

        // Calculate column widths
        let widths: Vec<Constraint> = result
            .columns
            .iter()
            .map(|_| Constraint::Percentage(100 / result.columns.len() as u16))
            .collect();

        let table = Table::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table, area);
    } else {
        let empty = Paragraph::new("No data to display")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .title(" Status ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let status_parts = if let Some(result) = &app.result {
        vec![
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::styled(
                format!("{} rows", result.rows.len()),
                Style::default().fg(Color::White),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}ms", result.execution_time_ms),
                Style::default().fg(Color::Gray),
            ),
            Span::styled(" │ ", Style::default().fg(Color::DarkGray)),
            Span::styled(&app.status_message, Style::default().fg(Color::Cyan)),
        ]
    } else {
        vec![Span::styled(
            &app.status_message,
            Style::default().fg(Color::Gray),
        )]
    };

    let status = Paragraph::new(Line::from(status_parts)).block(block);
    frame.render_widget(status, area);
}

fn draw_help_bar(frame: &mut Frame, area: Rect) {
    let help_items = vec![
        ("q", "Quit"),
        ("↑/k", "Up"),
        ("↓/j", "Down"),
        ("Tab", "Focus"),
        ("Enter", "Select"),
        ("s", "Schema"),
        ("d", "Data"),
    ];

    let spans: Vec<Span> = help_items
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Gray),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(Color::Gray)),
            ]
        })
        .collect();

    let help = Paragraph::new(Line::from(spans))
        .style(Style::default().bg(Color::Black));

    frame.render_widget(help, area);
}

/// Format bytes to human readable size
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Format number with thousand separators
fn format_number(n: usize) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
