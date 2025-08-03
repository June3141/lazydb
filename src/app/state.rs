use crate::config::Config;

#[derive(Debug)]
pub struct App {
    pub config: Config,
    pub should_quit: bool,
    pub current_view: ViewState,
    pub connection_list_state: ConnectionListState,
    pub database_explorer_state: DatabaseExplorerState,
    pub query_editor_state: QueryEditorState,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ViewState {
    ConnectionList,
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
            database_explorer_state: DatabaseExplorerState {
                focused_pane: DatabaseExplorerPane::Structure,
                structure_list_index: 0,
            },
            query_editor_state: QueryEditorState {
                focused_pane: QueryEditorPane::Editor,
            },
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
            ViewState::DatabaseExplorer => {
                if self.database_explorer_state.focused_pane == DatabaseExplorerPane::Structure {
                    match direction {
                        Direction::Up => {
                            if self.database_explorer_state.structure_list_index > 0 {
                                self.database_explorer_state.structure_list_index -= 1;
                            }
                        }
                        Direction::Down => {
                            // Use actual structure list length
                            if self.database_explorer_state.structure_list_index < self.database_explorer_state.structure_list.len().saturating_sub(1) {
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}