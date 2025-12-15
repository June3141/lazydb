use crate::app::{
    AddConnectionModal, ConfirmModalField, ConnectionModalField, DeleteProjectModal, ModalState,
    ProjectModal, ProjectModalField, SearchProjectModal,
};
use crate::model::Project;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub fn draw_modal(frame: &mut Frame, modal_state: &ModalState, projects: &[Project]) {
    match modal_state {
        ModalState::None => {}
        ModalState::AddConnection(modal) => {
            draw_add_connection_modal(frame, modal);
        }
        ModalState::AddProject(modal) => {
            draw_project_modal(frame, modal, " Add Project ");
        }
        ModalState::EditProject(_, modal) => {
            draw_project_modal(frame, modal, " Edit Project ");
        }
        ModalState::DeleteProject(modal) => {
            draw_delete_project_modal(frame, modal);
        }
        ModalState::SearchProject(modal) => {
            draw_search_project_modal(frame, modal, projects);
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

fn draw_connection_buttons(frame: &mut Frame, area: Rect, focused_field: ConnectionModalField) {
    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // OK button
    let ok_style = if focused_field == ConnectionModalField::ButtonOk {
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
    let cancel_style = if focused_field == ConnectionModalField::ButtonCancel {
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

fn draw_project_modal(frame: &mut Frame, modal: &ProjectModal, title: &str) {
    let area = centered_rect(40, 30, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal container
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta));

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
    let cancel_style = if focused_field == ProjectModalField::ButtonCancel {
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

fn draw_delete_project_modal(frame: &mut Frame, modal: &DeleteProjectModal) {
    let area = centered_rect(50, 25, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal container
    let block = Block::default()
        .title(" Delete Project ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red));

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
        Style::default().fg(Color::Yellow),
    )]))
    .alignment(Alignment::Center);
    frame.render_widget(warning, chunks[0]);

    // Project name
    let project_name = Paragraph::new(Line::from(vec![Span::styled(
        format!("\"{}\"", modal.project_name),
        Style::default()
            .fg(Color::White)
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

    // Delete button (danger)
    let delete_style = if focused_field == ConfirmModalField::ButtonOk {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Red)
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
            .fg(Color::Black)
            .bg(Color::Gray)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
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

fn draw_search_project_modal(frame: &mut Frame, modal: &SearchProjectModal, projects: &[Project]) {
    let area = centered_rect(50, 60, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal container
    let block = Block::default()
        .title(" Search Projects ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    frame.render_widget(block, area);

    // Inner area for content
    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Layout for search input and results
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search input
            Constraint::Length(1), // Results count
            Constraint::Min(5),    // Results list
            Constraint::Length(2), // Help text
        ])
        .split(inner);

    // Draw search input field
    let search_display = format!("{}_", modal.query);
    let search_input = Paragraph::new(search_display)
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Search "),
        );
    frame.render_widget(search_input, chunks[0]);

    // Draw results count
    let count_text = format!(
        " {} of {} projects ",
        modal.filtered_indices.len(),
        projects.len()
    );
    let count_paragraph = Paragraph::new(count_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(count_paragraph, chunks[1]);

    // Draw results list
    let results_area = chunks[2];
    let visible_height = results_area.height as usize;

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = if modal.selected_idx >= visible_height {
        modal.selected_idx - visible_height + 1
    } else {
        0
    };

    let results_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray))
        .title(" Results ");
    let results_inner = results_block.inner(results_area);
    frame.render_widget(results_block, results_area);

    // Render each visible result
    for (display_idx, &proj_idx) in modal
        .filtered_indices
        .iter()
        .skip(scroll_offset)
        .take(visible_height)
        .enumerate()
    {
        if let Some(project) = projects.get(proj_idx) {
            let actual_idx = scroll_offset + display_idx;
            let is_selected = actual_idx == modal.selected_idx;

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Highlight matching text
            let line = if !modal.query.is_empty() {
                highlight_match(&project.name, &modal.query, is_selected)
            } else {
                Line::from(Span::styled(&project.name, style))
            };

            let item_area = Rect {
                x: results_inner.x,
                y: results_inner.y + display_idx as u16,
                width: results_inner.width,
                height: 1,
            };

            let paragraph = Paragraph::new(line).style(style);
            frame.render_widget(paragraph, item_area);
        }
    }

    // Draw help text
    let help_text = Line::from(vec![
        Span::styled("Enter", Style::default().fg(Color::Green)),
        Span::raw(": select  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(": cancel  "),
        Span::styled("↑/↓", Style::default().fg(Color::Cyan)),
        Span::raw(": navigate"),
    ]);
    let help = Paragraph::new(help_text).alignment(Alignment::Center);
    frame.render_widget(help, chunks[3]);
}

/// Highlight matching substring in project name
fn highlight_match(text: &str, query: &str, is_selected: bool) -> Line<'static> {
    let text_lower = text.to_lowercase();
    let query_lower = query.to_lowercase();

    let base_style = if is_selected {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else {
        Style::default().fg(Color::White)
    };

    let highlight_style = if is_selected {
        Style::default()
            .fg(Color::Yellow)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    };

    if let Some(start) = text_lower.find(&query_lower) {
        let end = start + query.len();
        let before = &text[..start];
        let matched = &text[start..end];
        let after = &text[end..];

        Line::from(vec![
            Span::styled(before.to_string(), base_style),
            Span::styled(matched.to_string(), highlight_style),
            Span::styled(after.to_string(), base_style),
        ])
    } else {
        Line::from(Span::styled(text.to_string(), base_style))
    }
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
