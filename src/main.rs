mod app;
mod message;
mod model;
mod ui;

use anyhow::Result;
use app::{App, Focus, ModalField, ModalState};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use message::Message;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

/// A simple terminal UI for database management
#[derive(Parser, Debug)]
#[command(name = "lazydb")]
#[command(version, about, long_about = None)]
struct Args {}

fn main() -> Result<()> {
    let _args = Args::parse();

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

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<()> {
    loop {
        // View: render UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Handle input events
        if let Event::Key(key) = event::read()? {
            let message = if app.is_modal_open() {
                // Modal is open - handle modal-specific keys
                handle_modal_input(app, key.code)
            } else {
                // Normal mode
                match (key.code, key.modifiers) {
                    (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                        Some(Message::Quit)
                    }
                    // Shift + movement keys: directional pane navigation
                    (KeyCode::Left, KeyModifiers::SHIFT)
                    | (KeyCode::Char('H'), KeyModifiers::SHIFT) => Some(Message::FocusLeft),
                    (KeyCode::Right, KeyModifiers::SHIFT)
                    | (KeyCode::Char('L'), KeyModifiers::SHIFT) => Some(Message::FocusRight),
                    (KeyCode::Up, KeyModifiers::SHIFT)
                    | (KeyCode::Char('K'), KeyModifiers::SHIFT) => Some(Message::FocusUp),
                    (KeyCode::Down, KeyModifiers::SHIFT)
                    | (KeyCode::Char('J'), KeyModifiers::SHIFT) => Some(Message::FocusDown),
                    // Regular navigation within current pane
                    (KeyCode::Up | KeyCode::Char('k'), _) => Some(Message::NavigateUp),
                    (KeyCode::Down | KeyCode::Char('j'), _) => Some(Message::NavigateDown),
                    (KeyCode::Tab, _) => Some(Message::NextFocus),
                    (KeyCode::BackTab, _) => Some(Message::PrevFocus),
                    (KeyCode::Enter, _) => Some(Message::Activate),
                    (KeyCode::Char('e'), _) => Some(Message::ToggleExpandCollapse),
                    (KeyCode::Char('s'), _) => Some(Message::SwitchToSchema),
                    (KeyCode::Char('d'), _) => Some(Message::SwitchToData),
                    (KeyCode::Char('a'), _) if app.focus == Focus::Sidebar => {
                        Some(Message::OpenAddConnectionModal)
                    }
                    _ => None,
                }
            };

            if let Some(msg) = message {
                // Update: process message
                if app.update(msg) {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn handle_modal_input(app: &App, key_code: KeyCode) -> Option<Message> {
    match &app.modal_state {
        ModalState::None => None,
        ModalState::AddConnection(modal) => match key_code {
            KeyCode::Esc => Some(Message::CloseModal),
            KeyCode::Tab => Some(Message::ModalNextField),
            KeyCode::BackTab => Some(Message::ModalPrevField),
            KeyCode::Down | KeyCode::Char('j')
                if matches!(
                    modal.focused_field,
                    ModalField::ButtonOk | ModalField::ButtonCancel
                ) =>
            {
                Some(Message::ModalNextField)
            }
            KeyCode::Up | KeyCode::Char('k')
                if matches!(
                    modal.focused_field,
                    ModalField::ButtonOk | ModalField::ButtonCancel
                ) =>
            {
                Some(Message::ModalPrevField)
            }
            KeyCode::Left | KeyCode::Char('h')
                if matches!(
                    modal.focused_field,
                    ModalField::ButtonOk | ModalField::ButtonCancel
                ) =>
            {
                Some(Message::ModalPrevField)
            }
            KeyCode::Right | KeyCode::Char('l')
                if matches!(
                    modal.focused_field,
                    ModalField::ButtonOk | ModalField::ButtonCancel
                ) =>
            {
                Some(Message::ModalNextField)
            }
            KeyCode::Enter => match modal.focused_field {
                ModalField::ButtonOk => Some(Message::ModalConfirm),
                ModalField::ButtonCancel => Some(Message::CloseModal),
                _ => Some(Message::ModalNextField),
            },
            KeyCode::Backspace => Some(Message::ModalInputBackspace),
            KeyCode::Char(c) => Some(Message::ModalInputChar(c)),
            _ => None,
        },
    }
}
