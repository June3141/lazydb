//! Modal input and action handlers

use crate::app::enums::{MainPanelTab, SchemaSubTab, SidebarMode};
use crate::app::modal_fields::{ConfirmModalField, ConnectionModalField, ProjectModalField};
use crate::app::modals::{
    AddConnectionModal, ColumnVisibilityModal, DeleteProjectModal, HistoryModal, ModalState,
    ProjectModal, SearchConnectionModal, SearchProjectModal, SearchTableModal, UnifiedSearchModal,
    UnifiedSearchSection,
};
use crate::app::App;
use crate::model::{Connection, Project};

impl App {
    /// Handle character input for modals
    pub(crate) fn handle_modal_input_char(&mut self, c: char) {
        match &mut self.modal_state {
            ModalState::AddConnection(modal) => match modal.focused_field {
                ConnectionModalField::Name => modal.name.push(c),
                ConnectionModalField::Host => modal.host.push(c),
                ConnectionModalField::Port => {
                    if c.is_ascii_digit() && modal.port.len() < 5 {
                        modal.port.push(c);
                    }
                }
                ConnectionModalField::User => modal.user.push(c),
                ConnectionModalField::Password => modal.password.push(c),
                ConnectionModalField::Database => modal.database.push(c),
                ConnectionModalField::ButtonOk | ConnectionModalField::ButtonCancel => {}
            },
            ModalState::AddProject(modal) | ModalState::EditProject(_, modal) => {
                if modal.focused_field == ProjectModalField::Name {
                    modal.name.push(c);
                }
            }
            ModalState::SearchProject(modal) => {
                modal.query.push(c);
                modal.update_filter(&self.projects);
            }
            ModalState::SearchConnection(modal) => {
                modal.query.push(c);
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        modal.update_filter(&project.connections);
                    }
                }
            }
            ModalState::SearchTable(modal) => {
                modal.query.push(c);
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        if let Some(conn) = project.connections.get(self.selected_connection_idx) {
                            modal.update_filter(&conn.tables);
                        }
                    }
                }
            }
            ModalState::UnifiedSearch(modal) => {
                modal.query.push(c);
                // Get the filter data
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        let tables = if let Some(conn) =
                            project.connections.get(self.selected_connection_idx)
                        {
                            if conn.expanded {
                                &conn.tables[..]
                            } else {
                                &[]
                            }
                        } else {
                            &[]
                        };
                        modal.update_filter(&project.connections, tables);
                    }
                }
            }
            _ => {}
        }
    }

    /// Handle backspace for modals
    pub(crate) fn handle_modal_backspace(&mut self) {
        match &mut self.modal_state {
            ModalState::AddConnection(modal) => match modal.focused_field {
                ConnectionModalField::Name => {
                    modal.name.pop();
                }
                ConnectionModalField::Host => {
                    modal.host.pop();
                }
                ConnectionModalField::Port => {
                    modal.port.pop();
                }
                ConnectionModalField::User => {
                    modal.user.pop();
                }
                ConnectionModalField::Password => {
                    modal.password.pop();
                }
                ConnectionModalField::Database => {
                    modal.database.pop();
                }
                ConnectionModalField::ButtonOk | ConnectionModalField::ButtonCancel => {}
            },
            ModalState::AddProject(modal) | ModalState::EditProject(_, modal) => {
                if modal.focused_field == ProjectModalField::Name {
                    modal.name.pop();
                }
            }
            ModalState::SearchProject(modal) => {
                modal.query.pop();
                modal.update_filter(&self.projects);
            }
            ModalState::SearchConnection(modal) => {
                modal.query.pop();
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        modal.update_filter(&project.connections);
                    }
                }
            }
            ModalState::SearchTable(modal) => {
                modal.query.pop();
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        if let Some(conn) = project.connections.get(self.selected_connection_idx) {
                            modal.update_filter(&conn.tables);
                        }
                    }
                }
            }
            ModalState::UnifiedSearch(modal) => {
                modal.query.pop();
                // Get the filter data
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        let tables = if let Some(conn) =
                            project.connections.get(self.selected_connection_idx)
                        {
                            if conn.expanded {
                                &conn.tables[..]
                            } else {
                                &[]
                            }
                        } else {
                            &[]
                        };
                        modal.update_filter(&project.connections, tables);
                    }
                }
            }
            _ => {}
        }
    }

    /// Handle modal next field navigation
    pub(crate) fn handle_modal_next_field(&mut self) {
        match &mut self.modal_state {
            ModalState::AddConnection(modal) => {
                modal.focused_field = modal.focused_field.next();
            }
            ModalState::AddProject(modal) | ModalState::EditProject(_, modal) => {
                modal.focused_field = modal.focused_field.next();
            }
            ModalState::DeleteProject(modal) => {
                modal.focused_field = modal.focused_field.next();
            }
            ModalState::SearchProject(modal) => {
                modal.navigate_down();
            }
            ModalState::SearchConnection(modal) => {
                modal.navigate_down();
            }
            ModalState::SearchTable(modal) => {
                modal.navigate_down();
            }
            ModalState::UnifiedSearch(modal) => {
                modal.navigate_down();
            }
            ModalState::ColumnVisibility(modal) => {
                modal.navigate_down();
            }
            ModalState::None | ModalState::History(_) => {}
        }
    }

    /// Handle modal prev field navigation
    pub(crate) fn handle_modal_prev_field(&mut self) {
        match &mut self.modal_state {
            ModalState::AddConnection(modal) => {
                modal.focused_field = modal.focused_field.prev();
            }
            ModalState::AddProject(modal) | ModalState::EditProject(_, modal) => {
                modal.focused_field = modal.focused_field.prev();
            }
            ModalState::DeleteProject(modal) => {
                modal.focused_field = modal.focused_field.prev();
            }
            ModalState::SearchProject(modal) => {
                modal.navigate_up();
            }
            ModalState::SearchConnection(modal) => {
                modal.navigate_up();
            }
            ModalState::SearchTable(modal) => {
                modal.navigate_up();
            }
            ModalState::UnifiedSearch(modal) => {
                modal.navigate_up();
            }
            ModalState::ColumnVisibility(modal) => {
                modal.navigate_up();
            }
            ModalState::None | ModalState::History(_) => {}
        }
    }

    /// Handle modal confirm action
    pub(crate) fn handle_modal_confirm(&mut self) {
        match &self.modal_state {
            ModalState::AddConnection(modal) => {
                if let Some(conn) = self.create_connection_from_modal(modal) {
                    // Add connection to current project if in Connections mode
                    if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                        if let Some(project) = self.projects.get_mut(proj_idx) {
                            project.connections.push(conn);
                            self.status_message = "Connection added".to_string();
                        }
                    }
                    self.modal_state = ModalState::None;
                } else {
                    self.status_message =
                        "Invalid: fill name, host, user, database and valid port (1-65535)"
                            .to_string();
                    // Keep modal open for user to correct input
                }
            }
            ModalState::AddProject(modal) => {
                if modal.name.trim().is_empty() {
                    self.status_message = "Project name cannot be empty".to_string();
                } else {
                    let new_project = Project::new(modal.name.trim());
                    self.projects.push(new_project);
                    self.selected_project_idx = self.projects.len() - 1;
                    self.status_message = "Project added".to_string();
                    self.modal_state = ModalState::None;
                }
            }
            ModalState::EditProject(proj_idx, modal) => {
                if modal.name.trim().is_empty() {
                    self.status_message = "Project name cannot be empty".to_string();
                } else {
                    let proj_idx = *proj_idx;
                    let new_name = modal.name.trim().to_string();
                    if let Some(project) = self.projects.get_mut(proj_idx) {
                        project.name = new_name;
                        self.status_message = "Project updated".to_string();
                    }
                    self.modal_state = ModalState::None;
                }
            }
            ModalState::DeleteProject(modal) => {
                let proj_idx = modal.project_idx;
                if proj_idx < self.projects.len() {
                    self.projects.remove(proj_idx);
                    // Adjust selection if needed
                    if self.selected_project_idx >= self.projects.len() && !self.projects.is_empty()
                    {
                        self.selected_project_idx = self.projects.len() - 1;
                    }
                    self.status_message = "Project deleted".to_string();
                }
                self.modal_state = ModalState::None;
            }
            ModalState::SearchProject(_) => {
                // SearchProject uses SearchConfirm instead of ModalConfirm
            }
            ModalState::SearchConnection(_) => {
                // SearchConnection uses SearchConnectionConfirm instead of ModalConfirm
            }
            ModalState::SearchTable(_) => {
                // SearchTable uses TableSearchConfirm instead of ModalConfirm
            }
            ModalState::UnifiedSearch(_) => {
                // UnifiedSearch uses UnifiedSearchConfirm instead of ModalConfirm
            }
            ModalState::ColumnVisibility(_) => {
                // ColumnVisibility uses ToggleColumnVisibility, just close on confirm
                self.modal_state = ModalState::None;
            }
            ModalState::None | ModalState::History(_) => {}
        }
    }

    /// Create connection from modal data
    pub(crate) fn create_connection_from_modal(
        &self,
        modal: &AddConnectionModal,
    ) -> Option<Connection> {
        // Parse port as u16, which constrains the upper bound to 65535
        let port: u16 = modal.port.parse().ok()?;
        // Validate port range: reject port 0 (u16 already ensures <= 65535)
        if port == 0 {
            return None;
        }
        // Validate required fields: name, host, database, and user
        if modal.name.is_empty()
            || modal.host.is_empty()
            || modal.database.is_empty()
            || modal.user.is_empty()
        {
            return None;
        }
        Some(Connection {
            name: modal.name.clone(),
            host: modal.host.clone(),
            port,
            database: modal.database.clone(),
            username: modal.user.clone(),
            password: modal.password.clone(),
            expanded: false,
            tables: vec![],
            routines: vec![],
            routines_loaded: false,
        })
    }

    // ========================================================================
    // Modal Opening Handlers
    // ========================================================================
    // Note: These handlers are reserved for future refactoring of the update method.

    #[allow(dead_code)]
    pub(crate) fn open_add_connection_modal(&mut self) {
        self.modal_state = ModalState::AddConnection(AddConnectionModal::default());
    }

    #[allow(dead_code)]
    pub(crate) fn open_add_project_modal(&mut self) {
        self.modal_state = ModalState::AddProject(ProjectModal::default());
    }

    #[allow(dead_code)]
    pub(crate) fn open_edit_project_modal(&mut self) {
        if let SidebarMode::Projects = self.sidebar_mode {
            if let Some(project) = self.projects.get(self.selected_project_idx) {
                self.modal_state = ModalState::EditProject(
                    self.selected_project_idx,
                    ProjectModal::with_name(&project.name),
                );
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn open_delete_project_modal(&mut self) {
        if let SidebarMode::Projects = self.sidebar_mode {
            if let Some(project) = self.projects.get(self.selected_project_idx) {
                self.modal_state = ModalState::DeleteProject(DeleteProjectModal {
                    project_idx: self.selected_project_idx,
                    project_name: project.name.clone(),
                    focused_field: ConfirmModalField::ButtonCancel,
                });
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn open_search_project_modal(&mut self) {
        if let SidebarMode::Projects = self.sidebar_mode {
            self.modal_state = ModalState::SearchProject(SearchProjectModal::with_all_projects(
                self.projects.len(),
            ));
        }
    }

    #[allow(dead_code)]
    pub(crate) fn open_search_connection_modal(&mut self) {
        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
            if let Some(project) = self.projects.get(proj_idx) {
                self.modal_state = ModalState::SearchConnection(
                    SearchConnectionModal::with_all_connections(project.connections.len()),
                );
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn open_search_table_modal(&mut self) {
        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
            if let Some(project) = self.projects.get(proj_idx) {
                if let Some(conn) = project.connections.get(self.selected_connection_idx) {
                    if conn.expanded && !conn.tables.is_empty() {
                        self.modal_state = ModalState::SearchTable(
                            SearchTableModal::with_all_tables(conn.tables.len()),
                        );
                    }
                }
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn open_unified_search_modal(&mut self) {
        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
            if let Some(project) = self.projects.get(proj_idx) {
                let connection_count = project.connections.len();
                let tables_first = self.is_connection_expanded();
                let table_count =
                    if let Some(conn) = project.connections.get(self.selected_connection_idx) {
                        if conn.expanded {
                            conn.tables.len()
                        } else {
                            0
                        }
                    } else {
                        0
                    };
                self.modal_state = ModalState::UnifiedSearch(UnifiedSearchModal::new(
                    connection_count,
                    table_count,
                    tables_first,
                ));
            }
        }
    }

    #[allow(dead_code)]
    pub(crate) fn open_column_visibility_modal(&mut self) {
        // Only open in Schema tab (not Data or Relations)
        if self.panel_tab == MainPanelTab::Schema {
            self.modal_state =
                ModalState::ColumnVisibility(ColumnVisibilityModal::new(self.schema_sub_tab));
        }
    }

    #[allow(dead_code)]
    pub(crate) fn open_history_modal(&mut self) {
        if !self.query_history.is_empty() {
            self.modal_state = ModalState::History(HistoryModal::default());
        } else {
            self.status_message = "No query history".to_string();
        }
    }

    // ========================================================================
    // Search Confirm Handlers
    // ========================================================================

    pub(crate) fn handle_search_confirm(&mut self) {
        if let ModalState::SearchProject(modal) = &self.modal_state {
            if let Some(proj_idx) = modal.selected_project_idx() {
                self.selected_project_idx = proj_idx;
                self.status_message = format!("Selected: {}", self.projects[proj_idx].name);
            }
        }
        self.modal_state = ModalState::None;
    }

    pub(crate) fn handle_search_connection_confirm(&mut self) {
        if let ModalState::SearchConnection(modal) = &self.modal_state {
            if let Some(conn_idx) = modal.selected_connection_idx() {
                self.selected_connection_idx = conn_idx;
                self.selected_table_idx = None;
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        if let Some(conn) = project.connections.get(conn_idx) {
                            self.status_message = format!("Selected: {}", conn.name);
                        }
                    }
                }
            }
        }
        self.modal_state = ModalState::None;
    }

    pub(crate) fn handle_table_search_confirm(&mut self) {
        if let ModalState::SearchTable(modal) = &self.modal_state {
            if let Some(table_idx) = modal.selected_table_idx() {
                self.selected_table_idx = Some(table_idx);
                if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                    if let Some(project) = self.projects.get(proj_idx) {
                        if let Some(conn) = project.connections.get(self.selected_connection_idx) {
                            if let Some(table) = conn.tables.get(table_idx) {
                                self.status_message = format!("Selected: {}", table.name);
                            }
                        }
                    }
                }
            }
        }
        self.modal_state = ModalState::None;
    }

    pub(crate) fn handle_unified_search_confirm(&mut self) {
        if let ModalState::UnifiedSearch(modal) = &self.modal_state {
            match modal.active_section {
                UnifiedSearchSection::Connections => {
                    if let Some(conn_idx) = modal.selected_connection() {
                        self.selected_connection_idx = conn_idx;
                        self.selected_table_idx = None;
                        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                            if let Some(project) = self.projects.get(proj_idx) {
                                if let Some(conn) = project.connections.get(conn_idx) {
                                    self.status_message = format!("Selected: {}", conn.name);
                                }
                            }
                        }
                    }
                }
                UnifiedSearchSection::Tables => {
                    if let Some(table_idx) = modal.selected_table() {
                        self.selected_table_idx = Some(table_idx);
                        if let SidebarMode::Connections(proj_idx) = self.sidebar_mode {
                            if let Some(project) = self.projects.get(proj_idx) {
                                if let Some(conn) =
                                    project.connections.get(self.selected_connection_idx)
                                {
                                    if let Some(table) = conn.tables.get(table_idx) {
                                        self.status_message = format!("Selected: {}", table.name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        self.modal_state = ModalState::None;
    }

    // ========================================================================
    // Column Visibility Handler
    // ========================================================================

    pub(crate) fn toggle_column_visibility(&mut self) {
        if let ModalState::ColumnVisibility(modal) = &self.modal_state {
            let idx = modal.selected_idx;
            let tab = modal.tab;
            match tab {
                SchemaSubTab::Columns => {
                    self.column_visibility.columns.toggle(idx);
                }
                SchemaSubTab::Indexes => {
                    self.column_visibility.indexes.toggle(idx);
                }
                SchemaSubTab::ForeignKeys => {
                    self.column_visibility.foreign_keys.toggle(idx);
                }
                SchemaSubTab::Constraints => {
                    self.column_visibility.constraints.toggle(idx);
                }
                SchemaSubTab::Triggers => {
                    self.column_visibility.triggers.toggle(idx);
                }
                SchemaSubTab::Definition => {
                    // No visibility settings for Definition tab
                }
            }
        }
    }

    // ========================================================================
    // History Handlers
    // ========================================================================

    pub(crate) fn handle_history_navigate_up(&mut self) {
        if let ModalState::History(modal) = &mut self.modal_state {
            if modal.selected_idx > 0 {
                modal.selected_idx -= 1;
            } else if !self.query_history.is_empty() {
                modal.selected_idx = self.query_history.len() - 1;
            }
        }
    }

    pub(crate) fn handle_history_navigate_down(&mut self) {
        if let ModalState::History(modal) = &mut self.modal_state {
            if modal.selected_idx + 1 < self.query_history.len() {
                modal.selected_idx += 1;
            } else {
                modal.selected_idx = 0;
            }
        }
    }

    pub(crate) fn handle_history_select_entry(&mut self) {
        if let ModalState::History(modal) = &self.modal_state {
            if let Some(entry) = self.query_history.get(modal.selected_idx) {
                self.query = entry.query.clone();
                self.status_message =
                    format!("Loaded query from history ({})", entry.connection_name);
            }
            self.modal_state = ModalState::None;
        }
    }

    pub(crate) fn handle_clear_history(&mut self) {
        self.query_history.clear();
        self.history_dirty = true;
        // Reset selected_idx before closing modal
        if let ModalState::History(modal) = &mut self.modal_state {
            modal.selected_idx = 0;
        }
        self.modal_state = ModalState::None;
        self.status_message = "Query history cleared".to_string();
    }
}
