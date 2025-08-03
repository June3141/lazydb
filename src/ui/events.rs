use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crate::app::{App, ViewState};

pub fn handle_events(app: &mut App) -> anyhow::Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press {
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
                    _ => {}
                }
            }
        }
    }
    Ok(())
}