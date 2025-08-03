use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use crate::app::{App, ViewState, Direction};

pub fn handle_events(app: &mut App) -> anyhow::Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
                // Check for Ctrl modifier first for pane switching
                if key_event.modifiers.contains(KeyModifiers::CONTROL) {
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