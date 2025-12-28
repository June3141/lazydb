//! Connection modal rendering

use crate::app::{AddConnectionModal, ConnectionModalField};
use crate::ui::theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::helpers::{centered_rect, draw_input_field};

pub fn draw_add_connection_modal(frame: &mut Frame, modal: &AddConnectionModal) {
    let area = centered_rect(50, 70, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal container
    let block = Block::default()
        .title(" Add Connection ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(theme::border_focused());

    frame.render_widget(block, area);

    // Inner area for content
    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Layout for form fields
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Host
            Constraint::Length(3), // Port
            Constraint::Length(3), // User
            Constraint::Length(3), // Password
            Constraint::Length(3), // Database
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Buttons
        ])
        .split(inner);

    // Draw fields
    draw_input_field(
        frame,
        chunks[0],
        "Name",
        &modal.name,
        modal.focused_field == ConnectionModalField::Name,
        false,
    );
    draw_input_field(
        frame,
        chunks[1],
        "Host",
        &modal.host,
        modal.focused_field == ConnectionModalField::Host,
        false,
    );
    draw_input_field(
        frame,
        chunks[2],
        "Port",
        &modal.port,
        modal.focused_field == ConnectionModalField::Port,
        false,
    );
    draw_input_field(
        frame,
        chunks[3],
        "User",
        &modal.user,
        modal.focused_field == ConnectionModalField::User,
        false,
    );
    draw_input_field(
        frame,
        chunks[4],
        "Password",
        &modal.password,
        modal.focused_field == ConnectionModalField::Password,
        true,
    );
    draw_input_field(
        frame,
        chunks[5],
        "Database",
        &modal.database,
        modal.focused_field == ConnectionModalField::Database,
        false,
    );

    // Draw buttons
    draw_connection_buttons(frame, chunks[7], modal.focused_field);
}

fn draw_connection_buttons(frame: &mut Frame, area: Rect, focused_field: ConnectionModalField) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // OK button
    let ok_style = if focused_field == ConnectionModalField::ButtonOk {
        theme::focused()
    } else {
        theme::selected()
    };

    let ok_button = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("[ OK ]", ok_style),
        Span::raw(" "),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));

    // Cancel button
    let cancel_style = if focused_field == ConnectionModalField::ButtonCancel {
        theme::button_cancel_focused()
    } else {
        theme::muted()
    };

    let cancel_button = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("[ Cancel ]", cancel_style),
        Span::raw(" "),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));

    frame.render_widget(ok_button, button_chunks[0]);
    frame.render_widget(cancel_button, button_chunks[1]);
}
