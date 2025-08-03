use crate::config::Config;

#[derive(Debug)]
pub struct App {
    pub config: Config,
    pub should_quit: bool,
    pub current_view: ViewState,
}

#[derive(Debug, PartialEq)]
pub enum ViewState {
    ConnectionList,
    DatabaseExplorer,
    QueryEditor,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        let config = Config::load()?;
        
        Ok(Self {
            config,
            should_quit: false,
            current_view: ViewState::ConnectionList,
        })
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    pub fn switch_view(&mut self, view: ViewState) {
        self.current_view = view;
    }
}