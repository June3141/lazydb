//! Modal key handling (when a modal is open)

use crossterm::event::KeyCode;

use crossterm::event::KeyModifiers;

use crate::app::{
    AddConnectionModal, App, ColumnVisibilityModal, ConfirmModalField, ConnectionModalField,
    DeleteProjectModal, HistoryModal, ModalState, ProjectModal, ProjectModalField, QueryInputModal,
    SearchConnectionModal, SearchProjectModal, SearchTableModal, UnifiedSearchModal,
};
use crate::message::Message;

/// Handle keyboard input when a modal is open
pub fn handle_modal_input(
    app: &App,
    key_code: KeyCode,
    modifiers: KeyModifiers,
) -> Option<Message> {
    match &app.modal_state {
        ModalState::None => None,
        ModalState::AddConnection(modal) => handle_connection_modal(key_code, modal),
        ModalState::AddProject(modal) | ModalState::EditProject(_, modal) => {
            handle_project_modal(key_code, modal)
        }
        ModalState::DeleteProject(modal) => handle_delete_modal(key_code, modal),
        ModalState::SearchProject(modal) => handle_search_project_modal(key_code, modal),
        ModalState::SearchConnection(modal) => handle_search_connection_modal(key_code, modal),
        ModalState::SearchTable(modal) => handle_search_table_modal(key_code, modal),
        ModalState::UnifiedSearch(modal) => handle_unified_search_modal(key_code, modal),
        ModalState::History(modal) => handle_history_modal(key_code, modal),
        ModalState::ColumnVisibility(modal) => handle_column_visibility_modal(key_code, modal),
        ModalState::QueryInput(modal) => handle_query_input_modal(key_code, modifiers, modal),
    }
}

fn handle_connection_modal(key_code: KeyCode, modal: &AddConnectionModal) -> Option<Message> {
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

fn handle_project_modal(key_code: KeyCode, modal: &ProjectModal) -> Option<Message> {
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

fn handle_delete_modal(key_code: KeyCode, modal: &DeleteProjectModal) -> Option<Message> {
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

fn handle_search_project_modal(key_code: KeyCode, _modal: &SearchProjectModal) -> Option<Message> {
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

fn handle_search_connection_modal(
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

fn handle_search_table_modal(key_code: KeyCode, _modal: &SearchTableModal) -> Option<Message> {
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

fn handle_unified_search_modal(key_code: KeyCode, _modal: &UnifiedSearchModal) -> Option<Message> {
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

fn handle_history_modal(key_code: KeyCode, _modal: &HistoryModal) -> Option<Message> {
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

fn handle_column_visibility_modal(
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

fn handle_query_input_modal(
    key_code: KeyCode,
    modifiers: KeyModifiers,
    _modal: &QueryInputModal,
) -> Option<Message> {
    // Check for Ctrl modifier
    let ctrl = modifiers.contains(KeyModifiers::CONTROL);

    match key_code {
        // Esc to cancel and close modal
        KeyCode::Esc => Some(Message::CloseModal),
        // Ctrl+Enter to execute query
        KeyCode::Enter if ctrl => Some(Message::QueryInputExecute),
        // Regular Enter for newline
        KeyCode::Enter => Some(Message::QueryInputNewline),
        // Ctrl+r to open history (switches to history modal)
        KeyCode::Char('r') if ctrl => Some(Message::OpenHistoryModal),
        // Ctrl+u to clear input
        KeyCode::Char('u') if ctrl => Some(Message::QueryInputClear),
        // Backspace to delete character before cursor
        KeyCode::Backspace => Some(Message::QueryInputBackspace),
        // Delete to delete character at cursor
        KeyCode::Delete => Some(Message::QueryInputDelete),
        // Cursor movement
        KeyCode::Left => Some(Message::QueryInputCursorLeft),
        KeyCode::Right => Some(Message::QueryInputCursorRight),
        KeyCode::Up => Some(Message::QueryInputCursorUp),
        KeyCode::Down => Some(Message::QueryInputCursorDown),
        KeyCode::Home => Some(Message::QueryInputCursorHome),
        KeyCode::End => Some(Message::QueryInputCursorEnd),
        // Character input
        KeyCode::Char(c) if !ctrl => Some(Message::QueryInputChar(c)),
        _ => None,
    }
}
