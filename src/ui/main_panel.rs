use crate::app::{App, Focus, MainPanelTab};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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
    if let Some(table) = app.selected_table_info() {
        // Separator style for vertical lines
        let separator = "│";
        let separator_style = Style::default().fg(Color::DarkGray);

        // Create header row
        let header_cells = vec![
            Cell::from("#").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(separator).style(separator_style),
            Cell::from("PK").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(separator).style(separator_style),
            Cell::from("Column").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(separator).style(separator_style),
            Cell::from("Type").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(separator).style(separator_style),
            Cell::from("Comment").style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ];
        let header = Row::new(header_cells)
            .height(1)
            .style(Style::default().bg(Color::Rgb(40, 40, 50)))
            .bottom_margin(1);

        // Create data rows with zebra striping
        let row_count = table.columns.len();
        let row_num_width = format!("{}", row_count).len().max(2);

        let rows: Vec<Row> = table
            .columns
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                // Zebra stripe: alternate background colors
                let row_bg = if idx % 2 == 0 {
                    Color::Reset
                } else {
                    Color::Rgb(45, 45, 55)
                };

                let pk_marker = if col.is_primary_key { "🔑" } else { "" };
                let name_style = if col.is_primary_key {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::White)
                };

                let comment = col.comment.as_deref().unwrap_or("");

                let cells = vec![
                    Cell::from(format!("{:>width$}", idx + 1, width = row_num_width))
                        .style(Style::default().fg(Color::DarkGray)),
                    Cell::from(separator).style(separator_style),
                    Cell::from(pk_marker).style(Style::default().fg(Color::Yellow)),
                    Cell::from(separator).style(separator_style),
                    Cell::from(col.name.clone()).style(name_style),
                    Cell::from(separator).style(separator_style),
                    Cell::from(col.data_type.clone()).style(Style::default().fg(Color::Cyan)),
                    Cell::from(separator).style(separator_style),
                    Cell::from(comment).style(Style::default().fg(Color::DarkGray)),
                ];

                Row::new(cells).height(1).style(Style::default().bg(row_bg))
            })
            .collect();

        // Column widths
        let widths = vec![
            Constraint::Length(3),      // Row number
            Constraint::Length(1),      // Separator
            Constraint::Length(2),      // PK marker
            Constraint::Length(1),      // Separator
            Constraint::Percentage(25), // Column name
            Constraint::Length(1),      // Separator
            Constraint::Percentage(20), // Type
            Constraint::Length(1),      // Separator
            Constraint::Percentage(45), // Comment
        ];

        let schema_table = Table::new(rows, widths)
            .header(header)
            .column_spacing(1)
            .row_highlight_style(
                Style::default()
                    .bg(Color::Rgb(70, 70, 100))
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(schema_table, area);
    } else {
        let empty = Paragraph::new("Select a table to view schema")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

fn draw_data_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(result) = &app.result {
        // Row number column width (calculate based on total rows)
        let row_count = result.rows.len();
        let row_num_width = format!("{}", row_count).len().max(2); // minimum 2 digits

        // Separator style for vertical lines
        let separator = "│";
        let separator_style = Style::default().fg(Color::DarkGray);

        // Create header row with row number column and separators
        let mut header_cells: Vec<Cell> = vec![
            Cell::from("#").style(
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            ),
            Cell::from(separator).style(separator_style),
        ];
        for (i, col) in result.columns.iter().enumerate() {
            header_cells.push(
                Cell::from(col.clone()).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
            );
            // Add separator after each column except the last
            if i < result.columns.len() - 1 {
                header_cells.push(Cell::from(separator).style(separator_style));
            }
        }
        let header = Row::new(header_cells)
            .height(1)
            .style(Style::default().bg(Color::Rgb(40, 40, 50)))
            .bottom_margin(1); // Add margin for visual separation (acts as horizontal line)

        // Create data rows with row numbers, separators, and zebra striping
        let rows: Vec<Row> = result
            .rows
            .iter()
            .enumerate()
            .map(|(idx, row_data)| {
                // Zebra stripe: alternate background colors (more visible contrast)
                let row_bg = if idx % 2 == 0 {
                    Color::Reset // Default background
                } else {
                    Color::Rgb(45, 45, 55) // More visible stripe for odd rows
                };

                // Row number cell with separator
                let mut cells: Vec<Cell> = vec![
                    Cell::from(format!("{:>width$}", idx + 1, width = row_num_width))
                        .style(Style::default().fg(Color::DarkGray)),
                    Cell::from(separator).style(separator_style),
                ];

                // Data cells with separators
                for (i, cell_data) in row_data.iter().enumerate() {
                    cells.push(
                        Cell::from(cell_data.clone()).style(Style::default().fg(Color::White)),
                    );
                    // Add separator after each column except the last
                    if i < row_data.len() - 1 {
                        cells.push(Cell::from(separator).style(separator_style));
                    }
                }

                Row::new(cells).height(1).style(Style::default().bg(row_bg))
            })
            .collect();

        // Calculate column widths (row number + separator + data columns with separators)
        let data_col_count = result.columns.len();
        let mut widths: Vec<Constraint> = vec![
            Constraint::Length(row_num_width as u16), // Row number
            Constraint::Length(1),                    // Separator
        ];

        // Calculate remaining width for data columns
        if data_col_count > 0 {
            let per_col_percentage = 100u16.saturating_div(data_col_count as u16).max(5);
            for i in 0..data_col_count {
                widths.push(Constraint::Percentage(per_col_percentage));
                // Add separator width except after the last column
                if i < data_col_count - 1 {
                    widths.push(Constraint::Length(1));
                }
            }
        }

        let table = Table::new(rows, widths)
            .header(header)
            .column_spacing(1) // Add spacing between columns for better readability
            .row_highlight_style(
                Style::default()
                    .bg(Color::Rgb(70, 70, 100))
                    .add_modifier(Modifier::BOLD),
            );

        frame.render_widget(table, area);
    } else {
        let empty =
            Paragraph::new("No data to display").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}
