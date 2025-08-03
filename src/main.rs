mod app;
mod config;
mod db;
mod ui;

use app::App;
use ui::{events, terminal};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut app = App::new()?;
    let mut terminal = terminal::init()?;

    let result = run_app(&mut terminal, &mut app).await;

    terminal::restore()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

async fn run_app(terminal: &mut terminal::Tui, app: &mut App) -> anyhow::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, app))?;
        events::handle_events(app)?;
    }
    Ok(())
}
