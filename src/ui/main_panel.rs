use crate::app::{App, Focus, MainPanelTab, SchemaSubTab};
use crate::model::{ForeignKey, Table};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table as RatatuiTable, Tabs, Wrap},
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
    let tab_titles = vec!["Schema [s]", "Data [d]", "Relations [r]"];
    let selected_tab = match app.main_panel_tab {
        MainPanelTab::Schema => 0,
        MainPanelTab::Data => 1,
        MainPanelTab::Relations => 2,
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
        MainPanelTab::Relations => draw_relations_content(frame, app, inner_area),
    }
}

fn draw_schema_content(frame: &mut Frame, app: &App, area: Rect) {
    // Split for sub-tabs and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Sub-tabs
            Constraint::Min(1),    // Content
        ])
        .split(area);

    // Draw sub-tabs
    let sub_tab_titles = vec![
        "Columns [1]",
        "Indexes [2]",
        "Foreign Keys [3]",
        "Constraints [4]",
    ];
    let selected_sub_tab = match app.schema_sub_tab {
        SchemaSubTab::Columns => 0,
        SchemaSubTab::Indexes => 1,
        SchemaSubTab::ForeignKeys => 2,
        SchemaSubTab::Constraints => 3,
    };

    let sub_tabs = Tabs::new(sub_tab_titles)
        .select(selected_sub_tab)
        .style(Style::default().fg(Color::DarkGray))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .divider(" | ");

    frame.render_widget(sub_tabs, chunks[0]);

    // Draw content based on selected sub-tab
    match app.schema_sub_tab {
        SchemaSubTab::Columns => draw_columns_content(frame, app, chunks[1]),
        SchemaSubTab::Indexes => draw_indexes_content(frame, app, chunks[1]),
        SchemaSubTab::ForeignKeys => draw_foreign_keys_content(frame, app, chunks[1]),
        SchemaSubTab::Constraints => draw_constraints_content(frame, app, chunks[1]),
    }
}

fn draw_columns_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        // Create header
        let header_cells = ["", "Name", "Type", "Null", "Default", "Key"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            });
        let header = Row::new(header_cells).height(1);

        // Create rows
        let rows = table.columns.iter().map(|col| {
            let pk_marker = if col.is_primary_key {
                "🔑"
            } else if col.is_unique {
                "⚡"
            } else {
                ""
            };

            let key_info = if col.is_primary_key {
                "PK".to_string()
            } else if col.is_unique {
                "UQ".to_string()
            } else {
                "".to_string()
            };

            let null_str = if col.is_nullable { "YES" } else { "NO" };
            let default_str = col.default_value.as_deref().unwrap_or("-");

            let name_style = if col.is_primary_key {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                Cell::from(pk_marker),
                Cell::from(col.name.clone()).style(name_style),
                Cell::from(col.data_type.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(null_str).style(if col.is_nullable {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default().fg(Color::Red)
                }),
                Cell::from(default_str).style(Style::default().fg(Color::Green)),
                Cell::from(key_info).style(Style::default().fg(Color::Magenta)),
            ])
            .height(1)
        });

        let widths = [
            Constraint::Length(3),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Length(5),
            Constraint::Percentage(25),
            Constraint::Length(5),
        ];

        let table_widget = RatatuiTable::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view columns")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

fn draw_indexes_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.indexes.is_empty() {
            let empty =
                Paragraph::new("No indexes defined").style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
            return;
        }

        // Create header
        let header_cells = ["Name", "Type", "Method", "Columns"].iter().map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells).height(1);

        // Create rows
        let rows = table.indexes.iter().map(|idx| {
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

            let type_style = match idx.index_type {
                crate::model::IndexType::Primary => Style::default().fg(Color::Yellow),
                crate::model::IndexType::Unique => Style::default().fg(Color::Magenta),
                _ => Style::default().fg(Color::White),
            };

            Row::new(vec![
                Cell::from(idx.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(idx.index_type.to_string()).style(type_style),
                Cell::from(idx.method.to_string()).style(Style::default().fg(Color::DarkGray)),
                Cell::from(columns_str),
            ])
            .height(1)
        });

        let widths = [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(40),
        ];

        let table_widget = RatatuiTable::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view indexes")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

fn draw_foreign_keys_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.foreign_keys.is_empty() {
            let empty = Paragraph::new("No foreign keys defined")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
            return;
        }

        // Create header
        let header_cells = ["Name", "Column", "References", "ON DELETE", "ON UPDATE"]
            .iter()
            .map(|h| {
                Cell::from(*h).style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            });
        let header = Row::new(header_cells).height(1);

        // Create rows
        let rows = table.foreign_keys.iter().map(|fk| {
            let columns_str = fk.columns.join(", ");
            let ref_str = format!(
                "{}.{}",
                fk.referenced_table,
                fk.referenced_columns.join(", ")
            );

            Row::new(vec![
                Cell::from(fk.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(columns_str),
                Cell::from(ref_str).style(Style::default().fg(Color::Green)),
                Cell::from(fk.on_delete.to_string()).style(Style::default().fg(Color::Red)),
                Cell::from(fk.on_update.to_string()).style(Style::default().fg(Color::Magenta)),
            ])
            .height(1)
        });

        let widths = [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(25),
            Constraint::Percentage(17),
            Constraint::Percentage(18),
        ];

        let table_widget = RatatuiTable::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view foreign keys")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

fn draw_constraints_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if table.constraints.is_empty() {
            let empty = Paragraph::new("No constraints defined")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
            return;
        }

        // Create header
        let header_cells = ["Name", "Type", "Columns", "Definition"].iter().map(|h| {
            Cell::from(*h).style(
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )
        });
        let header = Row::new(header_cells).height(1);

        // Create rows
        let rows = table.constraints.iter().map(|c| {
            let columns_str = if c.columns.is_empty() {
                "-".to_string()
            } else {
                c.columns.join(", ")
            };
            let def_str = c.definition.as_deref().unwrap_or("-");

            let type_style = match c.constraint_type {
                crate::model::ConstraintType::PrimaryKey => Style::default().fg(Color::Yellow),
                crate::model::ConstraintType::Unique => Style::default().fg(Color::Magenta),
                crate::model::ConstraintType::ForeignKey => Style::default().fg(Color::Cyan),
                crate::model::ConstraintType::Check => Style::default().fg(Color::Green),
                _ => Style::default().fg(Color::White),
            };

            Row::new(vec![
                Cell::from(c.name.clone()).style(Style::default().fg(Color::Cyan)),
                Cell::from(c.constraint_type.to_string()).style(type_style),
                Cell::from(columns_str),
                Cell::from(def_str).style(Style::default().fg(Color::DarkGray)),
            ])
            .height(1)
        });

        let widths = [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(40),
        ];

        let table_widget = RatatuiTable::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table_widget, area);
    } else {
        let empty = Paragraph::new("Select a table to view constraints")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
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
            let cells = row_data
                .iter()
                .map(|cell| Cell::from(cell.clone()).style(Style::default().fg(Color::White)));
            Row::new(cells).height(1)
        });

        // Calculate column widths
        let widths: Vec<Constraint> = result
            .columns
            .iter()
            .map(|_| Constraint::Percentage(100 / result.columns.len() as u16))
            .collect();

        let table = RatatuiTable::new(rows, widths)
            .header(header)
            .row_highlight_style(Style::default().bg(Color::DarkGray));

        frame.render_widget(table, area);
    } else {
        let empty =
            Paragraph::new("No data to display").style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

fn draw_relations_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(tables) = app.current_connection_tables() {
        if tables.is_empty() {
            let empty = Paragraph::new("No tables in this connection")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty, area);
            return;
        }

        // Build ER diagram visualization
        let lines = build_er_diagram(tables);
        let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
        frame.render_widget(paragraph, area);
    } else {
        let empty = Paragraph::new("Select a connection to view relations")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, area);
    }
}

/// Build a text-based ER diagram showing table relationships
fn build_er_diagram(tables: &[Table]) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();

    // Title
    lines.push(Line::from(vec![Span::styled(
        "  Entity Relationship Diagram",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));
    lines.push(Line::from(""));

    // Collect all foreign key relationships
    let relationships: Vec<(&Table, &ForeignKey)> = tables
        .iter()
        .flat_map(|t| t.foreign_keys.iter().map(move |fk| (t, fk)))
        .collect();

    // Draw each table as a box
    for table in tables {
        // Table header
        let pk_cols: Vec<&str> = table
            .columns
            .iter()
            .filter(|c| c.is_primary_key)
            .map(|c| c.name.as_str())
            .collect();

        let fk_cols: Vec<&str> = table
            .foreign_keys
            .iter()
            .flat_map(|fk| fk.columns.iter().map(|s| s.as_str()))
            .collect();

        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("┌─ {} ", table.name),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "─".repeat(30_usize.saturating_sub(table.name.len())),
                Style::default().fg(Color::Cyan),
            ),
            Span::styled("┐", Style::default().fg(Color::Cyan)),
        ]));

        // Columns (limited to first 5 for brevity)
        for col in table.columns.iter().take(5) {
            let is_pk = pk_cols.contains(&col.name.as_str());
            let is_fk = fk_cols.contains(&col.name.as_str());

            let marker = if is_pk && is_fk {
                "🔑🔗"
            } else if is_pk {
                "🔑  "
            } else if is_fk {
                "  🔗"
            } else {
                "    "
            };

            let col_style = if is_pk {
                Style::default().fg(Color::Yellow)
            } else if is_fk {
                Style::default().fg(Color::Magenta)
            } else {
                Style::default().fg(Color::White)
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("│ ", Style::default().fg(Color::Cyan)),
                Span::raw(marker),
                Span::styled(format!("{:<18}", col.name), col_style),
                Span::styled(
                    format!("{:<10}", col.data_type),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled("│", Style::default().fg(Color::Cyan)),
            ]));
        }

        if table.columns.len() > 5 {
            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::styled("│ ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("    ... and {} more columns", table.columns.len() - 5),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("    "),
                Span::styled("│", Style::default().fg(Color::Cyan)),
            ]));
        }

        // Table footer
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(
                format!("└{}┘", "─".repeat(34)),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        // Draw relationships from this table
        for fk in &table.foreign_keys {
            lines.push(Line::from(vec![
                Span::raw("        "),
                Span::styled("│", Style::default().fg(Color::Green)),
            ]));
            lines.push(Line::from(vec![
                Span::raw("        "),
                Span::styled(
                    format!(
                        "└──▶ {}.{} ",
                        fk.referenced_table,
                        fk.referenced_columns.join(", ")
                    ),
                    Style::default().fg(Color::Green),
                ),
                Span::styled(
                    format!("({})", fk.name),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::raw("             "),
                Span::styled(
                    format!("ON DELETE: {} | ON UPDATE: {}", fk.on_delete, fk.on_update),
                    Style::default().fg(Color::DarkGray),
                ),
            ]));
        }

        lines.push(Line::from(""));
    }

    // Summary section
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Summary: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            format!(
                "{} tables, {} relationships",
                tables.len(),
                relationships.len()
            ),
            Style::default().fg(Color::Cyan),
        ),
    ]));

    // Legend
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  Legend: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("🔑 Primary Key  ", Style::default().fg(Color::Yellow)),
        Span::styled("🔗 Foreign Key", Style::default().fg(Color::Magenta)),
    ]));

    lines
}
