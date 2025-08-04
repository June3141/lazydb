use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crate::app::{App, ViewState, Direction, ConnectionListPane, NewConnectionStep};
use crate::config::{DatabaseType, Connection};

pub fn handle_events(app: &mut App) -> anyhow::Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                // Handle input dialog events first
                if app.input_dialog_state.is_some() {
                    match key_event.code {
                        KeyCode::Enter => {
                            app.submit_input_dialog()?;
                        }
                        KeyCode::Esc => {
                            app.close_input_dialog();
                        }
                        KeyCode::Backspace => {
                            app.handle_input_dialog_backspace();
                        }
                        KeyCode::Left => {
                            app.handle_input_dialog_left();
                        }
                        KeyCode::Right => {
                            app.handle_input_dialog_right();
                        }
                        KeyCode::Char(ch) => {
                            app.handle_input_dialog_input(ch);
                        }
                        _ => {}
                    }
                    return Ok(());
                }

                // Check for Shift modifier for special actions
                if key_event.modifiers.contains(KeyModifiers::SHIFT) {
                    match key_event.code {
                        KeyCode::Char('A') => {
                            // Only show new project dialog if we're in ConnectionList view and Projects pane is focused
                            if app.current_view == ViewState::ConnectionList 
                                && app.connection_list_state.focused_pane == ConnectionListPane::Projects {
                                app.show_new_project_dialog();
                            }
                        }
                        _ => {}
                    }
                // Check for Ctrl modifier for pane switching
                } else if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    match key_event.code {
                        KeyCode::Char('h') | KeyCode::Left => {
                            app.move_focus_next_pane();
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.move_focus_next_pane();
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.move_focus_next_pane();
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            app.move_focus_next_pane();
                        }
                        _ => {}
                    }
                } else {
                    // Regular key handling
                    match key_event.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Tab => {
                            if app.current_view == ViewState::NewConnection {
                                handle_new_connection_tab(app);
                            } else {
                                let next_view = match app.current_view {
                                    ViewState::ConnectionList => ViewState::NewConnection,
                                    ViewState::NewConnection => ViewState::DatabaseExplorer,
                                    ViewState::DatabaseExplorer => ViewState::QueryEditor,
                                    ViewState::QueryEditor => ViewState::ConnectionList,
                                };
                                app.switch_view(next_view);
                            }
                        }
                        KeyCode::Esc => {
                            app.switch_view(ViewState::ConnectionList);
                        }
                        KeyCode::Enter => {
                            if app.current_view == ViewState::NewConnection {
                                handle_new_connection_enter(app)?;
                            }
                        }
                        // Vim-style navigation within panes
                        KeyCode::Char('h') | KeyCode::Left => {
                            if app.current_view == ViewState::NewConnection {
                                // Handle left navigation in new connection view
                            } else {
                                app.move_within_pane(Direction::Left);
                            }
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            if app.current_view == ViewState::NewConnection {
                                handle_new_connection_down(app);
                            } else {
                                app.move_within_pane(Direction::Down);
                            }
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            if app.current_view == ViewState::NewConnection {
                                handle_new_connection_up(app);
                            } else {
                                app.move_within_pane(Direction::Up);
                            }
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            if app.current_view == ViewState::NewConnection {
                                // Handle right navigation in new connection view
                            } else {
                                app.move_within_pane(Direction::Right);
                            }
                        }
                        KeyCode::Char(c) => {
                            if app.current_view == ViewState::NewConnection {
                                handle_new_connection_char_input(app, c);
                            }
                        }
                        KeyCode::Backspace => {
                            if app.current_view == ViewState::NewConnection {
                                handle_new_connection_backspace(app);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}

fn handle_new_connection_up(app: &mut App) {
    match app.new_connection_state.step {
        NewConnectionStep::SelectDatabaseType => {
            if app.new_connection_state.database_type_index > 0 {
                app.new_connection_state.database_type_index -= 1;
            }
        }
        NewConnectionStep::FillConnectionDetails => {
            if app.new_connection_state.current_field > 0 {
                app.new_connection_state.current_field -= 1;
            }
        }
    }
}

fn handle_new_connection_down(app: &mut App) {
    match app.new_connection_state.step {
        NewConnectionStep::SelectDatabaseType => {
            let max_index = 3; // PostgreSQL, MySQL, SQLite, MongoDB (0-3)
            if app.new_connection_state.database_type_index < max_index {
                app.new_connection_state.database_type_index += 1;
            }
        }
        NewConnectionStep::FillConnectionDetails => {
            let max_fields = get_form_field_count(app.new_connection_state.selected_database_type.as_ref().unwrap());
            if app.new_connection_state.current_field < max_fields - 1 {
                app.new_connection_state.current_field += 1;
            }
        }
    }
}

fn handle_new_connection_enter(app: &mut App) -> anyhow::Result<()> {
    match app.new_connection_state.step {
        NewConnectionStep::SelectDatabaseType => {
            let database_types = vec![
                DatabaseType::PostgreSQL,
                DatabaseType::MySQL,
                DatabaseType::SQLite,
                DatabaseType::MongoDB,
            ];
            
            let selected_type = database_types[app.new_connection_state.database_type_index].clone();
            app.new_connection_state.selected_database_type = Some(selected_type.clone());
            app.new_connection_state.step = NewConnectionStep::FillConnectionDetails;
            app.new_connection_state.current_field = 0;
            
            // Set default values based on database type
            match selected_type {
                DatabaseType::PostgreSQL => {
                    app.new_connection_state.form_fields.port = "5432".to_string();
                }
                DatabaseType::MySQL => {
                    app.new_connection_state.form_fields.port = "3306".to_string();
                }
                DatabaseType::MongoDB => {
                    app.new_connection_state.form_fields.port = "27017".to_string();
                }
                DatabaseType::SQLite => {
                    app.new_connection_state.form_fields.host = "localhost".to_string();
                    app.new_connection_state.form_fields.port = "0".to_string();
                }
            }
        }
        NewConnectionStep::FillConnectionDetails => {
            // Save the connection
            save_new_connection(app)?;
            app.switch_view(ViewState::ConnectionList);
        }
    }
    Ok(())
}

fn handle_new_connection_tab(app: &mut App) {
    if app.new_connection_state.step == NewConnectionStep::FillConnectionDetails {
        let max_fields = get_form_field_count(app.new_connection_state.selected_database_type.as_ref().unwrap());
        app.new_connection_state.current_field = (app.new_connection_state.current_field + 1) % max_fields;
    }
}

fn handle_new_connection_char_input(app: &mut App, c: char) {
    if app.new_connection_state.step == NewConnectionStep::FillConnectionDetails {
        let field_index = app.new_connection_state.current_field;
        let db_type = app.new_connection_state.selected_database_type.as_ref().unwrap();
        
        match get_field_at_index(db_type, field_index) {
            0 => app.new_connection_state.form_fields.name.push(c), // Name
            1 => {
                if matches!(db_type, DatabaseType::SQLite) {
                    app.new_connection_state.form_fields.database_name.push(c); // SQLite file path
                } else {
                    app.new_connection_state.form_fields.host.push(c); // Host
                }
            }
            2 => app.new_connection_state.form_fields.port.push(c), // Port
            3 => app.new_connection_state.form_fields.username.push(c), // Username
            4 => app.new_connection_state.form_fields.password.push(c), // Password
            5 => app.new_connection_state.form_fields.database_name.push(c), // Database name
            _ => {}
        }
    }
}

fn handle_new_connection_backspace(app: &mut App) {
    if app.new_connection_state.step == NewConnectionStep::FillConnectionDetails {
        let field_index = app.new_connection_state.current_field;
        let db_type = app.new_connection_state.selected_database_type.as_ref().unwrap();
        
        match get_field_at_index(db_type, field_index) {
            0 => { app.new_connection_state.form_fields.name.pop(); }
            1 => {
                if matches!(db_type, DatabaseType::SQLite) {
                    app.new_connection_state.form_fields.database_name.pop();
                } else {
                    app.new_connection_state.form_fields.host.pop();
                }
            }
            2 => { app.new_connection_state.form_fields.port.pop(); }
            3 => { app.new_connection_state.form_fields.username.pop(); }
            4 => { app.new_connection_state.form_fields.password.pop(); }
            5 => { app.new_connection_state.form_fields.database_name.pop(); }
            _ => {}
        }
    }
}

fn get_form_field_count(db_type: &DatabaseType) -> usize {
    match db_type {
        DatabaseType::SQLite => 2, // Name, Database file path
        _ => 6, // Name, Host, Port, Username, Password, Database name
    }
}

fn get_field_at_index(db_type: &DatabaseType, index: usize) -> usize {
    match db_type {
        DatabaseType::SQLite => {
            match index {
                0 => 0, // Name
                1 => 5, // Database name (file path for SQLite)
                _ => 0,
            }
        }
        _ => index, // For other databases, index maps directly
    }
}

fn save_new_connection(app: &mut App) -> anyhow::Result<()> {
    let form_fields = &app.new_connection_state.form_fields;
    let db_type = app.new_connection_state.selected_database_type.as_ref().unwrap().clone();
    
    // Generate a unique ID
    let id = format!("{}_{}_{}", 
        format!("{:?}", db_type).to_lowercase(),
        form_fields.name.replace(" ", "_").to_lowercase(),
        chrono::Utc::now().timestamp_millis()
    );
    
    // Parse port
    let port = form_fields.port.parse::<u16>().unwrap_or(match db_type {
        DatabaseType::PostgreSQL => 5432,
        DatabaseType::MySQL => 3306,
        DatabaseType::MongoDB => 27017,
        DatabaseType::SQLite => 0,
    });
    
    let mut connection = Connection::new(
        id,
        form_fields.name.clone(),
        db_type,
        if matches!(app.new_connection_state.selected_database_type.as_ref().unwrap(), DatabaseType::SQLite) {
            "localhost".to_string()
        } else {
            form_fields.host.clone()
        },
        port,
        if matches!(app.new_connection_state.selected_database_type.as_ref().unwrap(), DatabaseType::SQLite) {
            "".to_string()
        } else {
            form_fields.username.clone()
        },
        if form_fields.database_name.is_empty() {
            None
        } else {
            Some(form_fields.database_name.clone())
        },
    );
    
    // Set password if provided
    if !form_fields.password.is_empty() && !matches!(app.new_connection_state.selected_database_type.as_ref().unwrap(), DatabaseType::SQLite) {
        connection.set_password(&form_fields.password)?;
    }
    
    // Add to config and save
    app.config.add_connection(connection);
    app.config.save()?;
    
    Ok(())
}