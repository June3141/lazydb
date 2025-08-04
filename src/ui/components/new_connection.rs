use crate::app::{App, NewConnectionStep, ConnectionFormFields};
use crate::config::DatabaseType;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    match app.new_connection_state.step {
        NewConnectionStep::SelectDatabaseType => {
            render_database_type_selection(frame, area, app);
        }
        NewConnectionStep::FillConnectionDetails => {
            render_connection_form(frame, area, app);
        }
    }
}

fn render_database_type_selection(frame: &mut Frame, area: Rect, app: &App) {
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
                .title("データベースの種類を選択してください")
                .border_style(Style::default().fg(Color::Yellow))
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let mut state = ListState::default();
    state.select(Some(app.new_connection_state.database_type_index));

    // Instructions
    let instructions = Paragraph::new("矢印キー: 選択  Enter: 決定  Esc: キャンセル")
        .block(Block::default().borders(Borders::ALL).title("操作方法"))
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),      // List
            Constraint::Length(3),   // Instructions
        ])
        .split(area);

    frame.render_stateful_widget(list, chunks[0], &mut state);
    frame.render_widget(instructions, chunks[1]);
}

fn render_connection_form(frame: &mut Frame, area: Rect, app: &App) {
    let fields = get_form_fields_for_database_type(
        app.new_connection_state.selected_database_type.as_ref().unwrap()
    );

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            std::iter::repeat(Constraint::Length(3))
                .take(fields.len() + 2) // +2 for title and instructions
                .collect::<Vec<_>>()
        )
        .split(area);

    // Title
    let title = Paragraph::new(format!(
        "{:?} 接続設定",
        app.new_connection_state.selected_database_type.as_ref().unwrap()
    ))
    .block(Block::default().borders(Borders::ALL))
    .style(Style::default().fg(Color::Yellow));

    frame.render_widget(title, chunks[0]);

    // Form fields
    for (i, field) in fields.iter().enumerate() {
        let is_current = i == app.new_connection_state.current_field;
        let value = get_field_value(&app.new_connection_state.form_fields, field);
        
        let input = Paragraph::new(value)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(field.display_name())
                    .border_style(if is_current {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default().fg(Color::White)
                    })
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(input, chunks[i + 1]);
    }

    // Instructions
    let instructions = Paragraph::new("Tab: 次のフィールド  Enter: 保存  Esc: キャンセル")
        .block(Block::default().borders(Borders::ALL).title("操作方法"))
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });

    frame.render_widget(instructions, chunks[chunks.len() - 1]);
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