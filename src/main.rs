mod app;
mod config;
mod message;
mod model;
mod ui;

use anyhow::Result;
use app::{
    App, ConfirmModalField, ConnectionModalField, Focus, ModalState, ProjectModalField,
};
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
                    (KeyCode::Backspace, _) if app.focus == Focus::Sidebar => Some(Message::GoBack),
                    (KeyCode::Char('s'), _) => Some(Message::SwitchToSchema),
                    (KeyCode::Char('d'), _) if app.focus != Focus::Sidebar => {
                        Some(Message::SwitchToData)
                    }
                    // Connection/Project add: 'a' key in sidebar
                    (KeyCode::Char('a'), _) if app.focus == Focus::Sidebar => {
                        match app.sidebar_mode {
                            app::SidebarMode::Projects => Some(Message::OpenAddProjectModal),
                            app::SidebarMode::Connections(_) => Some(Message::OpenAddConnectionModal),
                        }
                    }
                    // Project edit: 'e' key in Projects view
                    (KeyCode::Char('e'), _)
                        if app.focus == Focus::Sidebar
                            && matches!(app.sidebar_mode, app::SidebarMode::Projects) =>
                    {
                        Some(Message::OpenEditProjectModal)
                    }
                    // Project delete: 'd' key in Projects view
                    (KeyCode::Char('d'), _)
                        if app.focus == Focus::Sidebar
                            && matches!(app.sidebar_mode, app::SidebarMode::Projects) =>
                    {
                        Some(Message::DeleteProject)
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
        ModalState::AddConnection(modal) => handle_connection_modal_input(key_code, modal),
        ModalState::AddProject(modal) | ModalState::EditProject(_, modal) => {
            handle_project_modal_input(key_code, modal)
        }
        ModalState::DeleteProject(modal) => handle_delete_modal_input(key_code, modal),
    }
}

fn handle_connection_modal_input(
    key_code: KeyCode,
    modal: &app::AddConnectionModal,
) -> Option<Message> {
    match key_code {
        KeyCode::Esc => Some(Message::CloseModal),
        KeyCode::Tab => Some(Message::ModalNextField),
        KeyCode::BackTab => Some(Message::ModalPrevField),
        KeyCode::Down | KeyCode::Char('j')
            if matches!(
                modal.focused_field,
                ConnectionModalField::ButtonOk | ConnectionModalField::ButtonCancel
            ) =>
        {
            Some(Message::ModalNextField)
        }
        KeyCode::Up | KeyCode::Char('k')
            if matches!(
                modal.focused_field,
                ConnectionModalField::ButtonOk | ConnectionModalField::ButtonCancel
            ) =>
        {
            Some(Message::ModalPrevField)
        }
        KeyCode::Left | KeyCode::Char('h')
            if matches!(
                modal.focused_field,
                ConnectionModalField::ButtonOk | ConnectionModalField::ButtonCancel
            ) =>
        {
            Some(Message::ModalPrevField)
        }
        KeyCode::Right | KeyCode::Char('l')
            if matches!(
                modal.focused_field,
                ConnectionModalField::ButtonOk | ConnectionModalField::ButtonCancel
            ) =>
        {
            Some(Message::ModalNextField)
        }
        KeyCode::Enter => match modal.focused_field {
            ConnectionModalField::ButtonOk => Some(Message::ModalConfirm),
            ConnectionModalField::ButtonCancel => Some(Message::CloseModal),
            _ => Some(Message::ModalNextField),
        },
        KeyCode::Backspace => Some(Message::ModalInputBackspace),
        KeyCode::Char(c) => Some(Message::ModalInputChar(c)),
        _ => None,
    }
}

fn handle_project_modal_input(
    key_code: KeyCode,
    modal: &app::ProjectModal,
) -> Option<Message> {
    match key_code {
        KeyCode::Esc => Some(Message::CloseModal),
        KeyCode::Tab => Some(Message::ModalNextField),
        KeyCode::BackTab => Some(Message::ModalPrevField),
        KeyCode::Down | KeyCode::Char('j')
            if matches!(
                modal.focused_field,
                ProjectModalField::ButtonOk | ProjectModalField::ButtonCancel
            ) =>
        {
            Some(Message::ModalNextField)
        }
        KeyCode::Up | KeyCode::Char('k')
            if matches!(
                modal.focused_field,
                ProjectModalField::ButtonOk | ProjectModalField::ButtonCancel
            ) =>
        {
            Some(Message::ModalPrevField)
        }
        KeyCode::Left | KeyCode::Char('h')
            if matches!(
                modal.focused_field,
                ProjectModalField::ButtonOk | ProjectModalField::ButtonCancel
            ) =>
        {
            Some(Message::ModalPrevField)
        }
        KeyCode::Right | KeyCode::Char('l')
            if matches!(
                modal.focused_field,
                ProjectModalField::ButtonOk | ProjectModalField::ButtonCancel
            ) =>
        {
            Some(Message::ModalNextField)
        }
        KeyCode::Enter => match modal.focused_field {
            ProjectModalField::ButtonOk => Some(Message::ModalConfirm),
            ProjectModalField::ButtonCancel => Some(Message::CloseModal),
            ProjectModalField::Name => Some(Message::ModalNextField),
        },
        KeyCode::Backspace => Some(Message::ModalInputBackspace),
        KeyCode::Char(c) => Some(Message::ModalInputChar(c)),
        _ => None,
    }
}

fn handle_delete_modal_input(
    key_code: KeyCode,
    modal: &app::DeleteProjectModal,
) -> Option<Message> {
    match key_code {
        KeyCode::Esc => Some(Message::CloseModal),
        KeyCode::Tab | KeyCode::Left | KeyCode::Right | KeyCode::Char('h') | KeyCode::Char('l') => {
            Some(Message::ModalNextField)
        }
        KeyCode::BackTab => Some(Message::ModalPrevField),
        KeyCode::Enter => match modal.focused_field {
            ConfirmModalField::ButtonOk => Some(Message::ModalConfirm),
            ConfirmModalField::ButtonCancel => Some(Message::CloseModal),
        },
        _ => None,
    }
}
