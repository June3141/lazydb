//! Centralized color theme for the UI
//!
//! This module defines a minimal 5-color palette for consistent styling:
//! - Primary (Cyan): Focus/selection, active borders, links
//! - Accent (Yellow): Headers, emphasis, important info (PK, etc.)
//! - Muted (DarkGray): Inactive elements, borders, secondary info
//! - Text (White): Normal text
//! - Background (Black): Background color

use ratatui::style::{Color, Modifier, Style};

/// Primary color for focus/selection states and active elements
pub const PRIMARY: Color = Color::Cyan;

/// Accent color for headers and emphasis
pub const ACCENT: Color = Color::Yellow;

/// Muted color for inactive/secondary elements
pub const MUTED: Color = Color::DarkGray;

/// Default text color
pub const TEXT: Color = Color::White;

/// Background color
pub const BG: Color = Color::Black;

// =============================================================================
// Semantic Styles
// =============================================================================

/// Style for focused/selected elements (inverted: primary bg, black text)
pub fn focused() -> Style {
    Style::default()
        .bg(PRIMARY)
        .fg(BG)
        .add_modifier(Modifier::BOLD)
}

/// Style for selected but not focused elements
pub fn selected() -> Style {
    Style::default().fg(PRIMARY)
}

/// Style for active/focused borders
pub fn border_focused() -> Style {
    Style::default().fg(PRIMARY)
}

/// Style for inactive borders
pub fn border_inactive() -> Style {
    Style::default().fg(MUTED)
}

/// Style for headers and emphasis
pub fn header() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

/// Style for normal text
pub fn text() -> Style {
    Style::default().fg(TEXT)
}

/// Style for muted/secondary text
pub fn muted() -> Style {
    Style::default().fg(MUTED)
}

/// Style for highlighted row in tables
pub fn row_highlight() -> Style {
    Style::default().bg(MUTED).add_modifier(Modifier::BOLD)
}

/// Style for input field when focused
pub fn input_focused() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

/// Style for input field border when focused
pub fn input_border_focused() -> Style {
    Style::default().fg(ACCENT)
}

/// Style for input field border when inactive
pub fn input_border_inactive() -> Style {
    Style::default().fg(MUTED)
}

/// Style for highlighted match in search results
pub fn highlight_match() -> Style {
    Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)
}

/// Style for highlighted match in selected search result
pub fn highlight_match_selected() -> Style {
    Style::default()
        .fg(ACCENT)
        .bg(PRIMARY)
        .add_modifier(Modifier::BOLD)
}
