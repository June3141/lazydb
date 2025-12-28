//! Search modal rendering (projects, connections, tables, unified)

mod project;
mod connection;
mod table;
mod unified;

pub use project::draw_search_project_modal;
pub use connection::draw_search_connection_modal;
pub use table::draw_search_table_modal;
pub use unified::draw_unified_search_modal;

use ratatui::{
    layout::{Alignment, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

/// Draw help text for search modals
pub(super) fn draw_search_help(frame: &mut Frame, area: Rect) {
    let help_text = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(": select  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(": cancel  "),
        Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
        Span::raw(": navigate"),
    ]);
    let help = Paragraph::new(help_text).alignment(Alignment::Center);
    frame.render_widget(help, area);
}
