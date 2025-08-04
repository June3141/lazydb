use crate::config::{Config, DatabaseType};

#[derive(Debug)]
pub struct App {
    pub config: Config,
    pub should_quit: bool,
    pub current_view: ViewState,
    pub connection_list_state: ConnectionListState,
    pub new_connection_state: NewConnectionState,
    pub database_explorer_state: DatabaseExplorerState,
    pub query_editor_state: QueryEditorState,
    pub input_dialog_state: Option<InputDialogState>,
}

#[derive(Debug, Clone)]
pub struct InputDialogState {
    pub input_type: InputDialogType,
    pub input_text: String,
    pub cursor_position: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputDialogType {
    NewProject,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewState {
    ConnectionList,
    NewConnection,
    DatabaseExplorer,
    QueryEditor,
}

#[derive(Debug, Clone)]
pub struct ConnectionListState {
    pub focused_pane: ConnectionListPane,
    pub projects_list_index: usize,
    pub connections_list_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionListPane {
    Projects,
    Connections,
}

#[derive(Debug, Clone)]
pub struct DatabaseExplorerState {
    pub focused_pane: DatabaseExplorerPane,
    pub structure_list_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DatabaseExplorerPane {
    Structure,
    Content,
}

#[derive(Debug, Clone)]
pub struct QueryEditorState {
    pub focused_pane: QueryEditorPane,
}

#[derive(Debug, Clone, PartialEq)]
pub enum QueryEditorPane {
    Editor,
    Results,
}

#[derive(Debug, Clone)]
pub struct NewConnectionState {
    pub step: NewConnectionStep,
    pub selected_database_type: Option<DatabaseType>,
    pub database_type_index: usize,
    pub form_fields: ConnectionFormFields,
    pub current_field: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NewConnectionStep {
    SelectDatabaseType,
    FillConnectionDetails,
}

#[derive(Debug, Clone)]
pub struct ConnectionFormFields {
    pub name: String,
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub database_name: String,
}

#[derive(Debug, Clone)]
pub struct InputDialogState {
    pub input_type: InputDialogType,
    pub input_text: String,
    pub cursor_position: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputDialogType {
    NewProject,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load()?;
        
        Ok(Self {
            config,
            should_quit: false,
            current_view: ViewState::ConnectionList,
            connection_list_state: ConnectionListState {
                focused_pane: ConnectionListPane::Projects,
                projects_list_index: 0,
                connections_list_index: 0,
            },
            new_connection_state: NewConnectionState {
                step: NewConnectionStep::SelectDatabaseType,
                selected_database_type: None,
                database_type_index: 0,
                form_fields: ConnectionFormFields::default(),
                current_field: 0,
            },
            database_explorer_state: DatabaseExplorerState {
                focused_pane: DatabaseExplorerPane::Structure,
                structure_list_index: 0,
            },
            query_editor_state: QueryEditorState {
                focused_pane: QueryEditorPane::Editor,
            },
            input_dialog_state: None,
        })
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn switch_view(&mut self, view: ViewState) {
        self.current_view = view;
        // Reset focus to first pane when switching view
        match view {
            ViewState::ConnectionList => {
                self.connection_list_state.focused_pane = ConnectionListPane::Projects;
            }
            ViewState::NewConnection => {
                self.new_connection_state.step = NewConnectionStep::SelectDatabaseType;
                self.new_connection_state.selected_database_type = None;
                self.new_connection_state.database_type_index = 0;
                self.new_connection_state.form_fields = ConnectionFormFields::default();
                self.new_connection_state.current_field = 0;
            }
            ViewState::DatabaseExplorer => {
                self.database_explorer_state.focused_pane = DatabaseExplorerPane::Structure;
            }
            ViewState::QueryEditor => {
                self.query_editor_state.focused_pane = QueryEditorPane::Editor;
            }
        }
    }

    pub fn move_focus_next_pane(&mut self) {
        match self.current_view {
            ViewState::ConnectionList => {
                self.connection_list_state.focused_pane = match self.connection_list_state.focused_pane {
                    ConnectionListPane::Projects => ConnectionListPane::Connections,
                    ConnectionListPane::Connections => ConnectionListPane::Projects,
                };
            }
            ViewState::NewConnection => {
                // No panes to switch in new connection view
            }
            ViewState::DatabaseExplorer => {
                self.database_explorer_state.focused_pane = match self.database_explorer_state.focused_pane {
                    DatabaseExplorerPane::Structure => DatabaseExplorerPane::Content,
                    DatabaseExplorerPane::Content => DatabaseExplorerPane::Structure,
                };
            }
            ViewState::QueryEditor => {
                self.query_editor_state.focused_pane = match self.query_editor_state.focused_pane {
                    QueryEditorPane::Editor => QueryEditorPane::Results,
                    QueryEditorPane::Results => QueryEditorPane::Editor,
                };
            }
        }
    }

    pub fn move_within_pane(&mut self, direction: Direction) {
        match self.current_view {
            ViewState::ConnectionList => {
                match self.connection_list_state.focused_pane {
                    ConnectionListPane::Projects => {
                        match direction {
                            Direction::Up => {
                                if self.connection_list_state.projects_list_index > 0 {
                                    self.connection_list_state.projects_list_index -= 1;
                                }
                            }
                            Direction::Down => {
                                if self.connection_list_state.projects_list_index < self.config.projects.len().saturating_sub(1) {
                                    self.connection_list_state.projects_list_index += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    ConnectionListPane::Connections => {
                        match direction {
                            Direction::Up => {
                                if self.connection_list_state.connections_list_index > 0 {
                                    self.connection_list_state.connections_list_index -= 1;
                                }
                            }
                            Direction::Down => {
                                if self.connection_list_state.connections_list_index < self.config.connections.len().saturating_sub(1) {
                                    self.connection_list_state.connections_list_index += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            ViewState::NewConnection => {
                // Movement within new connection view is handled in events.rs
            }
            ViewState::DatabaseExplorer => {
                if self.database_explorer_state.focused_pane == DatabaseExplorerPane::Structure {
                    match direction {
                        Direction::Up => {
                            if self.database_explorer_state.structure_list_index > 0 {
                                self.database_explorer_state.structure_list_index -= 1;
                            }
                        }
                        Direction::Down => {
                            // TODO: Use actual structure list length
                            if self.database_explorer_state.structure_list_index < 5 {
                                self.database_explorer_state.structure_list_index += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
            ViewState::QueryEditor => {
                // Query editor movement will be handled separately
            }
        }
    }

    pub fn show_new_project_dialog(&mut self) {
        self.input_dialog_state = Some(InputDialogState {
            input_type: InputDialogType::NewProject,
            input_text: String::new(),
            cursor_position: 0,
        });
    }

    pub fn close_input_dialog(&mut self) {
        self.input_dialog_state = None;
    }

    pub fn handle_input_dialog_input(&mut self, ch: char) {
        if let Some(ref mut dialog_state) = self.input_dialog_state {
            dialog_state.input_text.insert(dialog_state.cursor_position, ch);
            dialog_state.cursor_position += 1;
        }
    }

    pub fn handle_input_dialog_backspace(&mut self) {
        if let Some(ref mut dialog_state) = self.input_dialog_state {
            if dialog_state.cursor_position > 0 {
                dialog_state.cursor_position -= 1;
                dialog_state.input_text.remove(dialog_state.cursor_position);
            }
        }
    }

    pub fn handle_input_dialog_left(&mut self) {
        if let Some(ref mut dialog_state) = self.input_dialog_state {
            if dialog_state.cursor_position > 0 {
                dialog_state.cursor_position -= 1;
            }
        }
    }

    pub fn handle_input_dialog_right(&mut self) {
        if let Some(ref mut dialog_state) = self.input_dialog_state {
            if dialog_state.cursor_position < dialog_state.input_text.len() {
                dialog_state.cursor_position += 1;
            }
        }
    }

    pub fn submit_input_dialog(&mut self) -> anyhow::Result<()> {
        if let Some(dialog_state) = &self.input_dialog_state {
            match dialog_state.input_type {
                InputDialogType::NewProject => {
                    if !dialog_state.input_text.trim().is_empty() {
                        let project = crate::config::Project {
                            id: uuid::Uuid::new_v4().to_string(),
                            name: dialog_state.input_text.trim().to_string(),
                            connection_ids: Vec::new(),
                        };
                        self.config.add_project(project);
                        self.config.save()?;
                    }
                }
            }
        }
        self.close_input_dialog();
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Default for ConnectionFormFields {
    fn default() -> Self {
        Self {
            name: String::new(),
            host: String::new(),
            port: String::new(),
            username: String::new(),
            password: String::new(),
            database_name: String::new(),
        }
    }
}