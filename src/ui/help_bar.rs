use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw_help_bar(frame: &mut Frame, area: Rect) {
    let help_items = [
        ("q", "Quit"),
        ("↑/k", "Up"),
        ("↓/j", "Down"),
        ("Tab", "Focus"),
        ("S-hjkl", "Pane"),
        ("Enter", "Select/Expand"),
        ("s", "Schema"),
        ("d", "Data"),
    ];

    let spans: Vec<Span> = help_items
        .iter()
        .flat_map(|(key, desc)| {
            vec![
                Span::styled(
                    format!(" {} ", key),
                    Style::default().fg(Color::Black).bg(Color::Gray),
                ),
                Span::styled(format!(" {} ", desc), Style::default().fg(Color::Gray)),
            ]
        })
        .collect();

    let help = Paragraph::new(Line::from(spans)).style(Style::default().bg(Color::Black));

    frame.render_widget(help, area);
}
