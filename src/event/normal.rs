//! Normal mode key handling (when no modal is open)

use crossterm::event::{KeyCode, KeyModifiers};

use crate::app::{App, Focus, MainPanelTab, SidebarMode};
use crate::message::Message;

/// Handle keyboard input in normal mode (no modal open)
pub fn handle_normal_input(
    app: &App,
    key_code: KeyCode,
    modifiers: KeyModifiers,
) -> Option<Message> {
    // Check if we're in data table navigation mode
    let in_data_table = app.focus == Focus::MainPanel
        && app.panel_tab == MainPanelTab::Data
        && app.result.is_some();

    match (key_code, modifiers) {
        // Quit
        (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
            Some(Message::Quit)
        }

        // Shift + movement keys: directional pane navigation
        (KeyCode::Left, KeyModifiers::SHIFT) | (KeyCode::Char('H'), KeyModifiers::SHIFT) => {
            Some(Message::FocusLeft)
        }
        (KeyCode::Right, KeyModifiers::SHIFT) | (KeyCode::Char('L'), KeyModifiers::SHIFT) => {
            Some(Message::FocusRight)
        }
        (KeyCode::Up, KeyModifiers::SHIFT) | (KeyCode::Char('K'), KeyModifiers::SHIFT) => {
            Some(Message::FocusUp)
        }
        (KeyCode::Down, KeyModifiers::SHIFT) | (KeyCode::Char('J'), KeyModifiers::SHIFT) => {
            Some(Message::FocusDown)
        }

        // Data table navigation (when in MainPanel with Data tab)
        (KeyCode::Up | KeyCode::Char('k'), _) if in_data_table => Some(Message::DataTableUp),
        (KeyCode::Down | KeyCode::Char('j'), _) if in_data_table => Some(Message::DataTableDown),
        (KeyCode::PageUp, _) if in_data_table => Some(Message::DataTablePageUp),
        (KeyCode::PageDown, _) if in_data_table => Some(Message::DataTablePageDown),
        (KeyCode::Char('g'), _) if in_data_table => Some(Message::DataTableFirst),
        (KeyCode::Char('G'), KeyModifiers::SHIFT) if in_data_table => Some(Message::DataTableLast),

        // Regular navigation within current pane (Sidebar)
        (KeyCode::Up | KeyCode::Char('k'), _) => Some(Message::NavigateUp),
        (KeyCode::Down | KeyCode::Char('j'), _) => Some(Message::NavigateDown),
        (KeyCode::Tab, _) => Some(Message::NextFocus),
        (KeyCode::BackTab, _) => Some(Message::PrevFocus),
        (KeyCode::Enter, _) => Some(Message::Activate),
        (KeyCode::Backspace, _) if app.focus == Focus::Sidebar => Some(Message::GoBack),

        // Tab switching
        (KeyCode::Char('s'), _) => Some(Message::SwitchToSchema),
        (KeyCode::Char('d'), _) if app.focus != Focus::Sidebar => Some(Message::SwitchToData),

        // Query history: Ctrl+r to open history modal (like shell reverse-search)
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => Some(Message::OpenHistoryModal),
        (KeyCode::Char('r'), _) => Some(Message::SwitchToRelations),

        // Schema sub-tab shortcuts (1-6)
        (KeyCode::Char('1'), _) => Some(Message::SwitchToColumns),
        (KeyCode::Char('2'), _) => Some(Message::SwitchToIndexes),
        (KeyCode::Char('3'), _) => Some(Message::SwitchToForeignKeys),
        (KeyCode::Char('4'), _) => Some(Message::SwitchToConstraints),
        (KeyCode::Char('5'), _) => Some(Message::SwitchToTriggers),
        (KeyCode::Char('6'), _) => Some(Message::SwitchToDefinition),

        // Pagination shortcuts (Data tab)
        (KeyCode::Char('n'), _) if app.panel_tab == MainPanelTab::Data => Some(Message::PageNext),
        (KeyCode::Char('p'), _) if app.panel_tab == MainPanelTab::Data => Some(Message::PagePrev),
        (KeyCode::Char('g'), _) if app.panel_tab == MainPanelTab::Data => Some(Message::PageFirst),
        (KeyCode::Char('G'), KeyModifiers::SHIFT) if app.panel_tab == MainPanelTab::Data => {
            Some(Message::PageLast)
        }
        (KeyCode::Char('z'), _) if app.panel_tab == MainPanelTab::Data => {
            Some(Message::PageSizeCycle)
        }

        // Add operation: 'a' key in sidebar (Project or Connection depending on mode)
        (KeyCode::Char('a'), _) if app.focus == Focus::Sidebar => match app.sidebar_mode {
            SidebarMode::Projects => Some(Message::OpenAddProjectModal),
            SidebarMode::Connections(_) => Some(Message::OpenAddConnectionModal),
        },

        // Project edit: 'e' key in Projects view
        (KeyCode::Char('e'), _)
            if app.focus == Focus::Sidebar && matches!(app.sidebar_mode, SidebarMode::Projects) =>
        {
            Some(Message::OpenEditProjectModal)
        }

        // Project delete: 'd' key in Projects view
        (KeyCode::Char('d'), _)
            if app.focus == Focus::Sidebar && matches!(app.sidebar_mode, SidebarMode::Projects) =>
        {
            Some(Message::DeleteProject)
        }

        // Project search: '/' key in Projects view
        (KeyCode::Char('/'), _)
            if app.focus == Focus::Sidebar && matches!(app.sidebar_mode, SidebarMode::Projects) =>
        {
            Some(Message::OpenSearchProjectModal)
        }

        // Unified search: '/' key in Connections view
        (KeyCode::Char('/'), _)
            if app.focus == Focus::Sidebar
                && matches!(app.sidebar_mode, SidebarMode::Connections(_)) =>
        {
            Some(Message::OpenUnifiedSearchModal)
        }

        // Column visibility: 'c' key in Schema tab when main panel is focused
        (KeyCode::Char('c'), _)
            if app.focus == Focus::MainPanel && app.panel_tab == MainPanelTab::Schema =>
        {
            Some(Message::OpenColumnVisibilityModal)
        }

        _ => None,
    }
}
