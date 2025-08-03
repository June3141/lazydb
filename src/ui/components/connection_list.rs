use crate::app::App;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Projects list
            Constraint::Percentage(70), // Connections list
        ])
        .split(area);

    render_projects_list(frame, chunks[0], app);
    render_connections_list(frame, chunks[1], app);
}

fn render_projects_list(frame: &mut Frame, area: Rect, app: &App) {
    let projects: Vec<ListItem> = app
        .config
        .projects
        .iter()
        .map(|project| {
            ListItem::new(project.name.clone())
        })
        .collect();

    let projects_list = List::new(projects)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Projects")
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    frame.render_widget(projects_list, area);
}

fn render_connections_list(frame: &mut Frame, area: Rect, app: &App) {
    let connections: Vec<ListItem> = app
        .config
        .connections
        .iter()
        .map(|connection| {
            let database_type = format!("{:?}", connection.database_type);
            let connection_info = format!("{} ({}:{})", 
                connection.name, 
                database_type,
                connection.host
            );
            ListItem::new(connection_info)
        })
        .collect();

    let connections_list = List::new(connections)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Connections")
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    frame.render_widget(connections_list, area);
}