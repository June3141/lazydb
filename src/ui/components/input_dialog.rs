use crate::app::{App, InputDialogType};
use ratatui::prelude::*;
use ratatui::widgets::*;

pub fn render(frame: &mut Frame, app: &App) {
    if let Some(dialog_state) = &app.input_dialog_state {
        let area = frame.area();
        
        // Create a centered popup area
        let popup_area = centered_rect(50, 20, area);
        
        // Clear the background
        let clear = Clear;
        frame.render_widget(clear, popup_area);
        
        // Create the input block
        let title = match dialog_state.input_type {
            InputDialogType::NewProject => "New Project",
        };
        
        let input_text = &dialog_state.input_text;
        let cursor_pos = dialog_state.cursor_position;
        
        // Create input text with cursor
        let mut input_with_cursor = input_text.clone();
        if cursor_pos <= input_with_cursor.len() {
            input_with_cursor.insert(cursor_pos, '|');
        }
        
        let input_paragraph = Paragraph::new(input_with_cursor)
            .style(Style::default().fg(Color::White))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(title)
                    .border_style(Style::default().fg(Color::Yellow))
            )
            .wrap(Wrap { trim: false });
        
        frame.render_widget(input_paragraph, popup_area);
        
        // Render help text
        let help_area = Rect {
            x: popup_area.x,
            y: popup_area.y + popup_area.height,
            width: popup_area.width,
            height: 1,
        };
        
        let help_text = "Enter: Submit | Esc: Cancel";
        let help_paragraph = Paragraph::new(help_text)
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        
        frame.render_widget(help_paragraph, help_area);
    }
}

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