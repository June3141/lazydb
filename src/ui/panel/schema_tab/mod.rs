//! Schema tab rendering (Columns, Indexes, Foreign Keys, Constraints, Triggers, Definition)

mod columns;
mod constraints;
mod definition;
mod foreign_keys;
mod indexes;
mod triggers;

use crate::app::{App, SchemaSubTab};
use crate::ui::theme;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Tabs,
    Frame,
};

pub fn draw_schema_content(frame: &mut Frame, app: &App, area: Rect) {
    // Split for sub-tabs and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Sub-tabs
            Constraint::Min(1),    // Content
        ])
        .split(area);

    // Check if selected table is a view (to show Definition tab)
    let is_view = app
        .selected_table_info()
        .map(|t| t.table_type.is_view())
        .unwrap_or(false);

    // Draw sub-tabs (Definition tab only shown for views)
    let sub_tab_titles: Vec<&str> = if is_view {
        vec![
            "Columns [1]",
            "Indexes [2]",
            "Foreign Keys [3]",
            "Constraints [4]",
            "Triggers [5]",
            "Definition [6]",
        ]
    } else {
        vec![
            "Columns [1]",
            "Indexes [2]",
            "Foreign Keys [3]",
            "Constraints [4]",
            "Triggers [5]",
        ]
    };

    let selected_sub_tab = match app.schema_sub_tab {
        SchemaSubTab::Columns => 0,
        SchemaSubTab::Indexes => 1,
        SchemaSubTab::ForeignKeys => 2,
        SchemaSubTab::Constraints => 3,
        SchemaSubTab::Triggers => 4,
        SchemaSubTab::Definition => {
            if is_view {
                5
            } else {
                0
            }
        }
    };

    let sub_tabs = Tabs::new(sub_tab_titles)
        .select(selected_sub_tab)
        .style(theme::muted())
        .highlight_style(theme::header())
        .divider(" | ");

    frame.render_widget(sub_tabs, chunks[0]);

    // Draw content based on selected sub-tab
    // For non-views with Definition tab selected, fall back to Columns content
    let effective_sub_tab = if !is_view && app.schema_sub_tab == SchemaSubTab::Definition {
        SchemaSubTab::Columns
    } else {
        app.schema_sub_tab
    };

    match effective_sub_tab {
        SchemaSubTab::Columns => columns::draw_columns_content(frame, app, chunks[1]),
        SchemaSubTab::Indexes => indexes::draw_indexes_content(frame, app, chunks[1]),
        SchemaSubTab::ForeignKeys => foreign_keys::draw_foreign_keys_content(frame, app, chunks[1]),
        SchemaSubTab::Constraints => constraints::draw_constraints_content(frame, app, chunks[1]),
        SchemaSubTab::Triggers => triggers::draw_triggers_content(frame, app, chunks[1]),
        SchemaSubTab::Definition => definition::draw_definition_content(frame, app, chunks[1]),
    }
}
