use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;

pub type Tui = Terminal<CrosstermBackend<io::Stdout>>;

pub fn init() -> anyhow::Result<Tui> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;
    terminal.clear()?;

    Ok(terminal)
}

pub fn restore() -> anyhow::Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    disable_raw_mode()?;
    Ok(())
}
