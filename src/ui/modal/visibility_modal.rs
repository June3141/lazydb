//! Column visibility modal rendering

use crate::app::{
    ColumnVisibilityModal, ColumnVisibilitySettings, ColumnsVisibility, ConstraintsVisibility,
    ForeignKeysVisibility, IndexesVisibility, SchemaSubTab, TriggersVisibility,
};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

use super::helpers::centered_rect;

pub fn draw_column_visibility_modal(
    frame: &mut Frame,
    modal: &ColumnVisibilityModal,
    settings: &ColumnVisibilitySettings,
) {
    let area = centered_rect(45, 50, frame.area());

    // Clear the area behind the modal
    frame.render_widget(Clear, area);

    // Modal title based on current tab
    let title = match modal.tab {
        SchemaSubTab::Columns => " Column Visibility - Columns ",
        SchemaSubTab::Indexes => " Column Visibility - Indexes ",
        SchemaSubTab::ForeignKeys => " Column Visibility - Foreign Keys ",
        SchemaSubTab::Constraints => " Column Visibility - Constraints ",
        SchemaSubTab::Triggers => " Column Visibility - Triggers ",
        SchemaSubTab::Definition => " View Definition ",
    };

    // Modal container
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    frame.render_widget(block, area);

    // Inner area for content
    let inner = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    // Get column names and visibility based on tab
    let (column_names, visibility_fn): (&[&str], Box<dyn Fn(usize) -> bool>) = match modal.tab {
        SchemaSubTab::Columns => (
            ColumnsVisibility::all_columns(),
            Box::new(|i| settings.columns.is_visible(i)),
        ),
        SchemaSubTab::Indexes => (
            IndexesVisibility::all_columns(),
            Box::new(|i| settings.indexes.is_visible(i)),
        ),
        SchemaSubTab::ForeignKeys => (
            ForeignKeysVisibility::all_columns(),
            Box::new(|i| settings.foreign_keys.is_visible(i)),
        ),
        SchemaSubTab::Constraints => (
            ConstraintsVisibility::all_columns(),
            Box::new(|i| settings.constraints.is_visible(i)),
        ),
        SchemaSubTab::Triggers => (
            TriggersVisibility::all_columns(),
            Box::new(|i| settings.triggers.is_visible(i)),
        ),
        SchemaSubTab::Definition => {
            // Definition tab has no column visibility settings
            (
                &[] as &[&str],
                Box::new(|_| true) as Box<dyn Fn(usize) -> bool>,
            )
        }
    };

    // Layout for column list and help text
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // Column list
            Constraint::Length(2), // Help text
        ])
        .split(inner);

    // Draw column list
    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray))
        .title(" Columns ");
    let list_inner = list_block.inner(chunks[0]);
    frame.render_widget(list_block, chunks[0]);

    let visible_height = list_inner.height as usize;
    let total_items = column_names.len();

    // Calculate scroll offset to keep selected item visible
    let scroll_offset = if visible_height == 0 || total_items <= visible_height {
        0
    } else if modal.selected_idx >= visible_height {
        (modal.selected_idx - visible_height + 1).min(total_items.saturating_sub(visible_height))
    } else {
        0
    };

    for (display_idx, idx) in (scroll_offset..total_items)
        .take(visible_height)
        .enumerate()
    {
        let col_name = column_names[idx];
        let is_selected = idx == modal.selected_idx;
        let is_visible = visibility_fn(idx);

        let checkbox = if is_visible { "[x]" } else { "[ ]" };

        let style = if is_selected {
            Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD)
        } else if is_visible {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let line = Line::from(vec![
            Span::styled(format!(" {} ", checkbox), style),
            Span::styled(col_name, style),
        ]);

        let item_area = Rect {
            x: list_inner.x,
            y: list_inner.y + display_idx as u16,
            width: list_inner.width,
            height: 1,
        };

        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, item_area);
    }

    // Draw help text
    let help_text = Line::from(vec![
        Span::styled("Space/Enter", Style::default().fg(Color::Green)),
        Span::raw(": toggle  "),
        Span::styled("Esc", Style::default().fg(Color::Red)),
        Span::raw(": close  "),
        Span::styled("j/k", Style::default().fg(Color::Cyan)),
        Span::raw(": navigate"),
    ]);
    let help = Paragraph::new(help_text).alignment(Alignment::Center);
    frame.render_widget(help, chunks[1]);
}
