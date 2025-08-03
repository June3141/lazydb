use crate::app::App;
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

fn render_database_structure(frame: &mut Frame, area: Rect, _app: &App) {
    let database_structure = List::new(vec![
        ListItem::new("📁 Tables"),
        ListItem::new("  📋 users"),
        ListItem::new("  📋 posts"),
        ListItem::new("  📋 comments"),
        ListItem::new("📁 Views"),
        ListItem::new("📁 Procedures"),
    ])
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Database Structure")
    )
    .style(Style::default().fg(Color::White))
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
    .highlight_symbol(">> ");

    frame.render_widget(database_structure, area);
}

fn render_table_content(frame: &mut Frame, area: Rect, _app: &App) {
    let table_content = Paragraph::new("Select a table to view its content")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Table Content")
        )
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(table_content, area);
}