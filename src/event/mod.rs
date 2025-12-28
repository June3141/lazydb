//! Event handling for keyboard input
//!
//! This module contains handlers for converting keyboard events into messages.

mod modal;
mod normal;

use crossterm::event::KeyCode;

use crate::app::App;
use crate::message::Message;

pub use modal::handle_modal_input;
pub use normal::handle_normal_input;

/// Convert a key event into a message based on current app state
pub fn key_to_message(app: &App, key_code: KeyCode, modifiers: crossterm::event::KeyModifiers) -> Option<Message> {
    if app.is_modal_open() {
        handle_modal_input(app, key_code)
    } else {
        handle_normal_input(app, key_code, modifiers)
    }
}
