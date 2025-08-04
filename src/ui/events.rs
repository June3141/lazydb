use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crate::app::{App, ViewState, Direction, ConnectionListPane};

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
                            let next_view = match app.current_view {
                                ViewState::ConnectionList => ViewState::DatabaseExplorer,
                                ViewState::DatabaseExplorer => ViewState::QueryEditor,
                                ViewState::QueryEditor => ViewState::ConnectionList,
                            };
                            app.switch_view(next_view);
                        }
                        KeyCode::Esc => {
                            app.switch_view(ViewState::ConnectionList);
                        }
                        // Vim-style navigation within panes
                        KeyCode::Char('h') | KeyCode::Left => {
                            app.move_within_pane(Direction::Left);
                        }
                        KeyCode::Char('j') | KeyCode::Down => {
                            app.move_within_pane(Direction::Down);
                        }
                        KeyCode::Char('k') | KeyCode::Up => {
                            app.move_within_pane(Direction::Up);
                        }
                        KeyCode::Char('l') | KeyCode::Right => {
                            app.move_within_pane(Direction::Right);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
    Ok(())
}