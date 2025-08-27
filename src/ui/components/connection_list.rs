use crate::app::{App, ConnectionListPane};
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
        .map(|project| ListItem::new(format!("📁 {}", project.name)))
        .collect();

    let is_focused = app.connection_list_state.focused_pane == ConnectionListPane::Projects;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let title = "Projects";

    let projects_list = List::new(projects)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    if is_focused {
        state.select(Some(app.connection_list_state.projects_list_index));
    }

    frame.render_stateful_widget(projects_list, area, &mut state);
}

fn render_connections_list(frame: &mut Frame, area: Rect, app: &App) {
    let connections: Vec<ListItem> = app
        .config
        .connections
        .iter()
        .map(|connection| {
            let database_type = format!("{:?}", connection.database_type);
            let connection_info = format!(
                "{} ({}:{})",
                connection.name, database_type, connection.host
            );
            ListItem::new(connection_info)
        })
        .collect();

    let is_focused = app.connection_list_state.focused_pane == ConnectionListPane::Connections;
    let border_style = if is_focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };

    let title = "Connections";

    let connections_list = List::new(connections)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    if is_focused {
        state.select(Some(app.connection_list_state.connections_list_index));
    }

    frame.render_stateful_widget(connections_list, area, &mut state);
}
