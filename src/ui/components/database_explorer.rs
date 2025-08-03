use crate::app::{App, DatabaseExplorerPane};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25), // Database structure
            Constraint::Percentage(75), // Table content
        ])
        .split(area);

    render_database_structure(frame, chunks[0], app);
    render_table_content(frame, chunks[1], app);
}

fn render_database_structure(frame: &mut Frame, area: Rect, app: &App) {
    let database_structure = List::new(vec![
        ListItem::new("📁 Tables"),
        ListItem::new("  📋 users"),
        ListItem::new("  📋 posts"),
        ListItem::new("  📋 comments"),
        ListItem::new("📁 Views"),
        ListItem::new("📁 Procedures"),
    ]);

    let is_focused = app.database_explorer_state.focused_pane == DatabaseExplorerPane::Structure;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let title = "Database Structure";

    let database_structure = database_structure
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style)
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    if is_focused {
        state.select(Some(app.database_explorer_state.structure_list_index));
    }

    frame.render_stateful_widget(database_structure, area, &mut state);
}

fn render_table_content(frame: &mut Frame, area: Rect, app: &App) {
    let is_focused = app.database_explorer_state.focused_pane == DatabaseExplorerPane::Content;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let title = "Table Content";

    let table_content = Paragraph::new("Select a table to view its content")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style)
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(table_content, area);
}