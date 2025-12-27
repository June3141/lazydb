mod app;
mod config;
mod db;
mod message;
mod model;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use app::{
    App, ColumnVisibilityModal, ConfirmModalField, ConnectionModalField, Focus, HistoryModal,
    MainPanelTab, ModalState, ProjectModalField, SearchConnectionModal, SearchProjectModal,
    SearchTableModal, UnifiedSearchModal,
};
use clap::Parser;
use config::ConfigLoader;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use db::spawn_db_worker;
use message::Message;
use model::Project;
use ratatui::{backend::CrosstermBackend, Terminal};

/// Poll timeout for checking DB responses while waiting for input.
/// Lower values = more responsive to async results but higher CPU usage.
const POLL_TIMEOUT: Duration = Duration::from_millis(50);

/// A simple terminal UI for database management
#[derive(Parser, Debug)]
#[command(name = "lazydb")]
#[command(version, about, long_about = None)]
struct Args {}

fn main() -> Result<()> {
    let _args = Args::parse();

    // Initialize config directory and load projects
    let config_loader = ConfigLoader::new()?;
    config_loader.init_config_dir()?;
    let config = config_loader.load_config()?;
    let (project_files, _warnings) = config_loader.load_all_projects(&config);
    let projects: Vec<Project> = project_files.into_iter().map(Project::from).collect();

    // Load query history (ignore errors - start with empty history if load fails)
    let history = config_loader.load_history().unwrap_or_default();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app with loaded projects and history
    let mut app = App::with_history(projects, history);

    // Spawn background DB worker thread
    let db_worker = spawn_db_worker();
    app.set_db_worker(db_worker);

    // Main loop
    let res = run_app(&mut terminal, &mut app, &config_loader);

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
    config_loader: &ConfigLoader,
) -> Result<()> {
    loop {
        // View: render UI
        terminal.draw(|frame| ui::draw(frame, app))?;

        // Process any pending DB responses (non-blocking)
        app.process_db_responses();

        // Poll for input events with timeout (allows checking DB responses regularly)
        if !event::poll(POLL_TIMEOUT)? {
            // No input event - continue loop to check for DB responses
            continue;
        }

        // Handle input events
        if let Event::Key(key) = event::read()? {
            let message = if app.is_modal_open() {
                // Modal is open - handle modal-specific keys
                handle_modal_input(app, key.code)
            } else {
                // Check if we're in data table navigation mode
                let in_data_table = app.focus == Focus::MainPanel
                    && app.main_panel_tab == MainPanelTab::Data
                    && app.result.is_some();

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
                    // Data table navigation (when in MainPanel with Data tab)
                    (KeyCode::Up | KeyCode::Char('k'), _) if in_data_table => {
                        Some(Message::DataTableUp)
                    }
                    (KeyCode::Down | KeyCode::Char('j'), _) if in_data_table => {
                        Some(Message::DataTableDown)
                    }
                    (KeyCode::PageUp, _) if in_data_table => Some(Message::DataTablePageUp),
                    (KeyCode::PageDown, _) if in_data_table => Some(Message::DataTablePageDown),
                    (KeyCode::Char('g'), _) if in_data_table => Some(Message::DataTableFirst),
                    (KeyCode::Char('G'), KeyModifiers::SHIFT) if in_data_table => {
                        Some(Message::DataTableLast)
                    }
                    // Regular navigation within current pane (Sidebar)
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
                    // Query history: Ctrl+r to open history modal (like shell reverse-search)
                    (KeyCode::Char('r'), KeyModifiers::CONTROL) => Some(Message::OpenHistoryModal),
                    (KeyCode::Char('r'), _) => Some(Message::SwitchToRelations),
                    // Schema sub-tab shortcuts (1-5)
                    (KeyCode::Char('1'), _) => Some(Message::SwitchToColumns),
                    (KeyCode::Char('2'), _) => Some(Message::SwitchToIndexes),
                    (KeyCode::Char('3'), _) => Some(Message::SwitchToForeignKeys),
                    (KeyCode::Char('4'), _) => Some(Message::SwitchToConstraints),
                    (KeyCode::Char('5'), _) => Some(Message::SwitchToDefinition),
                    // Pagination shortcuts (Data tab)
                    (KeyCode::Char('n'), _) if app.main_panel_tab == app::MainPanelTab::Data => {
                        Some(Message::PageNext)
                    }
                    (KeyCode::Char('p'), _) if app.main_panel_tab == app::MainPanelTab::Data => {
                        Some(Message::PagePrev)
                    }
                    (KeyCode::Char('g'), _) if app.main_panel_tab == app::MainPanelTab::Data => {
                        Some(Message::PageFirst)
                    }
                    (KeyCode::Char('G'), KeyModifiers::SHIFT)
                        if app.main_panel_tab == app::MainPanelTab::Data =>
                    {
                        Some(Message::PageLast)
                    }
                    (KeyCode::Char('z'), _) if app.main_panel_tab == app::MainPanelTab::Data => {
                        Some(Message::PageSizeCycle)
                    }
                    // Add operation: 'a' key in sidebar (Project or Connection depending on mode)
                    (KeyCode::Char('a'), _) if app.focus == Focus::Sidebar => {
                        match app.sidebar_mode {
                            app::SidebarMode::Projects => Some(Message::OpenAddProjectModal),
                            app::SidebarMode::Connections(_) => {
                                Some(Message::OpenAddConnectionModal)
                            }
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
                    // Project search: '/' key in Projects view
                    (KeyCode::Char('/'), _)
                        if app.focus == Focus::Sidebar
                            && matches!(app.sidebar_mode, app::SidebarMode::Projects) =>
                    {
                        Some(Message::OpenSearchProjectModal)
                    }
                    // Unified search: '/' key in Connections view
                    // Searches connections, and tables if a connection is expanded
                    (KeyCode::Char('/'), _)
                        if app.focus == Focus::Sidebar
                            && matches!(app.sidebar_mode, app::SidebarMode::Connections(_)) =>
                    {
                        Some(Message::OpenUnifiedSearchModal)
                    }
                    // Column visibility: 'c' key in Schema tab when main panel is focused
                    (KeyCode::Char('c'), _)
                        if app.focus == Focus::MainPanel
                            && app.main_panel_tab == MainPanelTab::Schema =>
                    {
                        Some(Message::OpenColumnVisibilityModal)
                    }
                    _ => None,
                }
            };

            if let Some(msg) = message {
                // Update: process message
                let should_quit = app.update(msg);

                // Save history if dirty
                if app.history_dirty {
                    if let Err(e) = config_loader.save_history(&app.query_history) {
                        app.status_message = format!("Failed to save history: {}", e);
                    }
                    app.history_dirty = false;
                }

                if should_quit {
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
        ModalState::SearchProject(modal) => handle_search_project_modal_input(key_code, modal),
        ModalState::SearchConnection(modal) => {
            handle_search_connection_modal_input(key_code, modal)
        }
        ModalState::SearchTable(modal) => handle_search_table_modal_input(key_code, modal),
        ModalState::UnifiedSearch(modal) => handle_unified_search_modal_input(key_code, modal),
        ModalState::History(modal) => handle_history_modal_input(key_code, modal),
        ModalState::ColumnVisibility(modal) => {
            handle_column_visibility_modal_input(key_code, modal)
        }
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

fn handle_project_modal_input(key_code: KeyCode, modal: &app::ProjectModal) -> Option<Message> {
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

fn handle_search_project_modal_input(
    key_code: KeyCode,
    _modal: &SearchProjectModal,
) -> Option<Message> {
    match key_code {
        KeyCode::Esc => Some(Message::CloseModal),
        KeyCode::Enter => Some(Message::SearchConfirm),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::ModalPrevField),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::ModalNextField),
        KeyCode::Tab => Some(Message::ModalNextField),
        KeyCode::BackTab => Some(Message::ModalPrevField),
        KeyCode::Backspace => Some(Message::ModalInputBackspace),
        KeyCode::Char(c) => Some(Message::ModalInputChar(c)),
        _ => None,
    }
}

fn handle_search_connection_modal_input(
    key_code: KeyCode,
    _modal: &SearchConnectionModal,
) -> Option<Message> {
    match key_code {
        KeyCode::Esc => Some(Message::CloseModal),
        KeyCode::Enter => Some(Message::SearchConnectionConfirm),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::ModalPrevField),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::ModalNextField),
        KeyCode::Tab => Some(Message::ModalNextField),
        KeyCode::BackTab => Some(Message::ModalPrevField),
        KeyCode::Backspace => Some(Message::ModalInputBackspace),
        KeyCode::Char(c) => Some(Message::ModalInputChar(c)),
        _ => None,
    }
}

fn handle_search_table_modal_input(
    key_code: KeyCode,
    _modal: &SearchTableModal,
) -> Option<Message> {
    match key_code {
        KeyCode::Esc => Some(Message::CloseModal),
        KeyCode::Enter => Some(Message::TableSearchConfirm),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::ModalPrevField),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::ModalNextField),
        KeyCode::Tab => Some(Message::ModalNextField),
        KeyCode::BackTab => Some(Message::ModalPrevField),
        KeyCode::Backspace => Some(Message::ModalInputBackspace),
        KeyCode::Char(c) => Some(Message::ModalInputChar(c)),
        _ => None,
    }
}

fn handle_unified_search_modal_input(
    key_code: KeyCode,
    _modal: &UnifiedSearchModal,
) -> Option<Message> {
    match key_code {
        KeyCode::Esc => Some(Message::CloseModal),
        KeyCode::Enter => Some(Message::UnifiedSearchConfirm),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::ModalPrevField),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::ModalNextField),
        KeyCode::Tab => Some(Message::UnifiedSearchSwitchSection),
        KeyCode::BackTab => Some(Message::UnifiedSearchSwitchSection),
        KeyCode::Backspace => Some(Message::ModalInputBackspace),
        KeyCode::Char(c) => Some(Message::ModalInputChar(c)),
        _ => None,
    }
}

fn handle_history_modal_input(key_code: KeyCode, _modal: &HistoryModal) -> Option<Message> {
    match key_code {
        KeyCode::Esc | KeyCode::Char('q') => Some(Message::CloseModal),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::HistoryNavigateUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::HistoryNavigateDown),
        KeyCode::Enter => Some(Message::HistorySelectEntry),
        // 'c' to clear history
        KeyCode::Char('c') => Some(Message::ClearHistory),
        _ => None,
    }
}

fn handle_column_visibility_modal_input(
    key_code: KeyCode,
    _modal: &ColumnVisibilityModal,
) -> Option<Message> {
    match key_code {
        KeyCode::Esc | KeyCode::Char('q') => Some(Message::CloseModal),
        KeyCode::Up | KeyCode::Char('k') => Some(Message::ModalPrevField),
        KeyCode::Down | KeyCode::Char('j') => Some(Message::ModalNextField),
        KeyCode::Enter | KeyCode::Char(' ') => Some(Message::ToggleColumnVisibility),
        _ => None,
    }
}
