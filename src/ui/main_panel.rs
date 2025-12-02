use crate::app::{App, Focus, MainPanelTab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};

pub fn draw_query_editor(frame: &mut Frame, app: &App, area: Rect) {
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

pub fn draw_main_panel(frame: &mut Frame, app: &App, area: Rect) {
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
                    Span::styled(format!("{:<20}", &col.name), name_style),
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
