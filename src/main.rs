mod app;
mod message;
mod model;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use message::Message;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with sample data
    let mut app = App::new_with_sample_data();

    // Main loop
    let res = run_app(&mut terminal, &mut app);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {err:?}");
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        // View: render UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle input events
        if let Event::Key(key) = event::read()? {
            let message = match (key.code, key.modifiers) {
                (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                    Message::Quit
                }
                (KeyCode::Up | KeyCode::Char('k'), _) => Message::NavigateUp,
                (KeyCode::Down | KeyCode::Char('j'), _) => Message::NavigateDown,
                (KeyCode::Tab, _) => Message::NextFocus,
                (KeyCode::BackTab, _) => Message::PrevFocus,
                (KeyCode::Enter, _) => Message::Activate,
                (KeyCode::Char('e'), _) => Message::ToggleExpandCollapse,
                (KeyCode::Char('s'), _) => Message::SwitchToSchema,
                (KeyCode::Char('d'), _) => Message::SwitchToData,
                _ => continue,
            };

            // Update: process message
            if app.update(message) {
                break;
            }
        }
    }

    Ok(())
}
