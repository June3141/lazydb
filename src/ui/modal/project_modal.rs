//! Project modal rendering

use crate::app::{ConfirmModalField, DeleteProjectModal, ProjectModal, ProjectModalField};
use crate::ui::theme;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::helpers::{centered_rect, draw_input_field};

pub fn draw_project_modal(frame: &mut Frame, modal: &ProjectModal, title: &str) {
    let area = centered_rect(40, 30, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal container
    let block = Block::default()
        .title(title)
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
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Buttons
        ])
        .split(inner);

    // Draw Name field
    draw_input_field(
        frame,
        chunks[0],
        "Name",
        &modal.name,
        modal.focused_field == ProjectModalField::Name,
        false,
    );

    // Draw buttons
    draw_project_buttons(frame, chunks[2], modal.focused_field);
}

fn draw_project_buttons(frame: &mut Frame, area: Rect, focused_field: ProjectModalField) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // OK button
    let ok_style = if focused_field == ProjectModalField::ButtonOk {
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
    let cancel_style = if focused_field == ProjectModalField::ButtonCancel {
        Style::default()
            .fg(theme::BG)
            .bg(theme::MUTED)
            .add_modifier(Modifier::BOLD)
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

pub fn draw_delete_project_modal(frame: &mut Frame, modal: &DeleteProjectModal) {
    let area = centered_rect(50, 25, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal container
    let block = Block::default()
        .title(" Delete Project ")
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

    // Layout for content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Message
            Constraint::Length(1), // Project name
            Constraint::Length(1), // Spacer
            Constraint::Length(3), // Buttons
        ])
        .split(inner);

    // Warning message
    let warning = Paragraph::new(Line::from(vec![Span::styled(
        "Are you sure you want to delete this project?",
        theme::header(),
    )]))
    .alignment(Alignment::Center);
    frame.render_widget(warning, chunks[0]);

    // Project name
    let project_name = Paragraph::new(Line::from(vec![Span::styled(
        format!("\"{}\"", modal.project_name),
        Style::default()
            .fg(theme::TEXT)
            .add_modifier(Modifier::BOLD),
    )]))
    .alignment(Alignment::Center);
    frame.render_widget(project_name, chunks[1]);

    // Draw buttons
    draw_confirm_buttons(frame, chunks[3], modal.focused_field);
}

fn draw_confirm_buttons(frame: &mut Frame, area: Rect, focused_field: ConfirmModalField) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Delete button (using accent for emphasis)
    let delete_style = if focused_field == ConfirmModalField::ButtonOk {
        Style::default()
            .fg(theme::BG)
            .bg(theme::ACCENT)
            .add_modifier(Modifier::BOLD)
    } else {
        theme::header()
    };

    let delete_button = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("[ Delete ]", delete_style),
        Span::raw(" "),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::NONE));

    // Cancel button
    let cancel_style = if focused_field == ConfirmModalField::ButtonCancel {
        Style::default()
            .fg(theme::BG)
            .bg(theme::MUTED)
            .add_modifier(Modifier::BOLD)
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

    frame.render_widget(cancel_button, button_chunks[0]);
    frame.render_widget(delete_button, button_chunks[1]);
}
