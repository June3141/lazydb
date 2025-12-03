use crate::app::{AddConnectionModal, ModalField, ModalState};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw_modal(frame: &mut Frame, modal_state: &ModalState) {
    match modal_state {
        ModalState::None => {}
        ModalState::AddConnection(modal) => {
            draw_add_connection_modal(frame, modal);
        }
    }
}

fn draw_add_connection_modal(frame: &mut Frame, modal: &AddConnectionModal) {
    let area = centered_rect(50, 70, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal container
    let block = Block::default()
        .title(" Add Connection ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

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
        modal.focused_field == ModalField::Name,
        false,
    );
    draw_input_field(
        frame,
        chunks[1],
        "Host",
        &modal.host,
        modal.focused_field == ModalField::Host,
        false,
    );
    draw_input_field(
        frame,
        chunks[2],
        "Port",
        &modal.port,
        modal.focused_field == ModalField::Port,
        false,
    );
    draw_input_field(
        frame,
        chunks[3],
        "User",
        &modal.user,
        modal.focused_field == ModalField::User,
        false,
    );
    draw_input_field(
        frame,
        chunks[4],
        "Password",
        &modal.password,
        modal.focused_field == ModalField::Password,
        true,
    );
    draw_input_field(
        frame,
        chunks[5],
        "Database",
        &modal.database,
        modal.focused_field == ModalField::Database,
        false,
    );

    // Draw buttons
    draw_buttons(frame, chunks[7], modal.focused_field);
}

fn draw_input_field(
    frame: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    focused: bool,
    is_password: bool,
) {
    let style = if focused {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let border_style = if focused {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    // Mask password field
    let masked_value = if is_password {
        "*".repeat(value.len())
    } else {
        value.to_string()
    };

    // Display value with cursor if focused
    let display_value = if focused {
        format!("{}_", masked_value)
    } else {
        masked_value
    };

    let input = Paragraph::new(display_value).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(format!(" {} ", label)),
    );

    frame.render_widget(input, area);
}

fn draw_buttons(frame: &mut Frame, area: Rect, focused_field: ModalField) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // OK button
    let ok_style = if focused_field == ModalField::ButtonOk {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Green)
    };

    let ok_button = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("[ OK ]", ok_style),
        Span::raw(" "),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));

    // Cancel button
    let cancel_style = if focused_field == ModalField::ButtonCancel {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
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

/// Create a centered rectangle with given percentage of width and height
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
