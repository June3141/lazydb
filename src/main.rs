//! lazydb - A simple terminal UI for database management
//!
//! lazydb is a TUI (Terminal User Interface) tool that allows developers to interact
//! with databases directly from the terminal. It aims to provide a lightweight
//! alternative to GUI database clients like DBeaver, focusing on keyboard-driven workflows.
//!
//! # Architecture
//!
//! lazydb follows **The Elm Architecture (TEA)** pattern:
//!
//! 1. **Model** - Application state ([`app::App`])
//! 2. **View** - UI rendering ([`ui`] module)
//! 3. **Update** - State updates via messages ([`message::Message`])
//!
//! # Modules
//!
//! - [`app`] - Application state and update logic
//! - [`config`] - Configuration file management
//! - [`db`] - Database provider abstraction layer
//! - [`event`] - Keyboard event handling
//! - [`export`] - Data export (CSV, JSON)
//! - [`message`] - Message types for TEA pattern
//! - [`model`] - Data models
//! - [`ui`] - User interface components

mod app;
mod config;
mod db;
mod event;
mod export;
mod message;
mod model;
mod ui;

use std::io;
use std::time::Duration;

use anyhow::Result;
use app::App;
use clap::Parser;
use config::ConfigLoader;
use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use db::spawn_db_worker;
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
        if !poll(POLL_TIMEOUT)? {
            // No input event - continue loop to check for DB responses
            continue;
        }

        // Handle input events
        if let Event::Key(key) = read()? {
            let message = event::key_to_message(app, key.code, key.modifiers);

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
