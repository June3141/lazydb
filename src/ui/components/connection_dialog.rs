use crate::app::{App, ConnectionFormFields, NewConnectionStep};
use crate::config::DatabaseType;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    if app.connection_dialog_state.is_some() {
        let area = frame.area();

        // Create a larger centered popup area for the connection dialog
        let popup_area = centered_rect(80, 70, area);

        // Clear the background
        let clear = Clear;
        frame.render_widget(clear, popup_area);

        match app.new_connection_state.step {
            NewConnectionStep::SelectProject => {
                render_project_selection_dialog(frame, popup_area, app);
            }
            NewConnectionStep::SelectDatabaseType => {
                render_database_type_selection_dialog(frame, popup_area, app);
            }
            NewConnectionStep::FillConnectionDetails => {
                render_connection_form_dialog(frame, popup_area, app);
            }
        }
    }
}

fn render_project_selection_dialog(frame: &mut Frame, area: Rect, app: &App) {
    let projects = &app.config.projects;

    let items: Vec<ListItem> = projects
        .iter()
        .map(|project| ListItem::new(project.name.clone()))
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("接続を追加するプロジェクトを選択")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    if !projects.is_empty() {
        state.select(Some(app.new_connection_state.project_index));
    }

    // Split area for list and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // List
            Constraint::Length(1), // Help
        ])
        .split(area);

    frame.render_stateful_widget(list, chunks[0], &mut state);

    // Help text
    let help_text = "↑↓: 選択 | Enter: 決定 | Esc: キャンセル";
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(help_paragraph, chunks[1]);
}

fn render_database_type_selection_dialog(frame: &mut Frame, area: Rect, app: &App) {
    let database_types = vec![
        DatabaseType::PostgreSQL,
        DatabaseType::MySQL,
        DatabaseType::SQLite,
        DatabaseType::MongoDB,
    ];

    let items: Vec<ListItem> = database_types
        .iter()
        .map(|db_type| {
            let name = format!("{:?}", db_type);
            ListItem::new(name)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("データベースの種類を選択")
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.new_connection_state.database_type_index));

    // Split area for list and help
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // List
            Constraint::Length(1), // Help
        ])
        .split(area);

    frame.render_stateful_widget(list, chunks[0], &mut state);

    // Help text
    let help_text = "↑↓: 選択 | Enter: 決定 | Esc: キャンセル";
    let help_paragraph = Paragraph::new(help_text)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center);

    frame.render_widget(help_paragraph, chunks[1]);
}

fn render_connection_form_dialog(frame: &mut Frame, area: Rect, app: &App) {
    let fields = get_form_fields_for_database_type(
        app.new_connection_state
            .selected_database_type
            .as_ref()
            .unwrap(),
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            std::iter::repeat(Constraint::Length(3))
                .take(fields.len() + 1) // +1 for help text
                .collect::<Vec<_>>(),
        )
        .split(area);

    // Title with selected DB type
    let title = format!(
        "{:?} 接続設定",
        app.new_connection_state
            .selected_database_type
            .as_ref()
            .unwrap()
    );

    let title_block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Yellow));

    frame.render_widget(title_block, chunks[0]);

    // Form fields
    for (i, field) in fields.iter().enumerate() {
        if i + 1 < chunks.len() - 1 {
            let is_current = i == app.new_connection_state.current_field;
            let value = get_field_value(&app.new_connection_state.form_fields, field);

            // Add cursor to current field
            let mut display_value = value.to_string();
            if is_current {
                display_value.push('|');
            }

            let input = Paragraph::new(display_value)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(field.display_name())
                        .border_style(if is_current {
                            Style::default().fg(Color::Yellow)
                        } else {
                            Style::default().fg(Color::White)
                        }),
                )
                .style(Style::default().fg(Color::White));

            frame.render_widget(input, chunks[i + 1]);
        }
    }

    // Help text
    if let Some(help_chunk) = chunks.last() {
        let help_text = "Tab: 次のフィールド | Enter: 保存 | Esc: キャンセル";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);

        frame.render_widget(help_paragraph, *help_chunk);
    }
}

#[derive(Debug, Clone)]
enum FormField {
    Name,
    Host,
    Port,
    Username,
    Password,
    DatabaseName,
}

impl FormField {
    fn display_name(&self) -> &str {
        match self {
            FormField::Name => "接続名",
            FormField::Host => "ホスト",
            FormField::Port => "ポート",
            FormField::Username => "ユーザー名",
            FormField::Password => "パスワード",
            FormField::DatabaseName => "データベース名",
        }
    }
}

fn get_form_fields_for_database_type(db_type: &DatabaseType) -> Vec<FormField> {
    match db_type {
        DatabaseType::PostgreSQL => vec![
            FormField::Name,
            FormField::Host,
            FormField::Port,
            FormField::Username,
            FormField::Password,
            FormField::DatabaseName,
        ],
        DatabaseType::MySQL => vec![
            FormField::Name,
            FormField::Host,
            FormField::Port,
            FormField::Username,
            FormField::Password,
            FormField::DatabaseName,
        ],
        DatabaseType::SQLite => vec![
            FormField::Name,
            FormField::DatabaseName, // SQLiteではファイルパス
        ],
        DatabaseType::MongoDB => vec![
            FormField::Name,
            FormField::Host,
            FormField::Port,
            FormField::Username,
            FormField::Password,
            FormField::DatabaseName,
        ],
    }
}

fn get_field_value<'a>(form_fields: &'a ConnectionFormFields, field: &FormField) -> &'a str {
    match field {
        FormField::Name => &form_fields.name,
        FormField::Host => &form_fields.host,
        FormField::Port => &form_fields.port,
        FormField::Username => &form_fields.username,
        FormField::Password => &form_fields.password,
        FormField::DatabaseName => &form_fields.database_name,
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
