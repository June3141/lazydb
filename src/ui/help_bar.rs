use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::theme;

pub fn draw_help_bar(frame: &mut Frame, area: Rect) {
    let help_items = [
        ("q", "Quit"),
        ("↑/k", "Up"),
        ("↓/j", "Down"),
        ("Tab", "Focus"),
        ("S-hjkl", "Pane"),
        ("Enter", "Select"),
        ("BS", "Back"),
        ("a", "Add"),
        ("s/d/r", "Schema/Data/Relations"),
        ("1-4", "SubTab"),
    ];

    let spans: Vec<Span> = help_items
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default().fg(theme::BG).bg(theme::MUTED),
                ),
                Span::styled(format!(" {} ", desc), theme::muted()),
            ]
        })
        .collect();

    let help = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme::BG));

    frame.render_widget(help, area);
}
