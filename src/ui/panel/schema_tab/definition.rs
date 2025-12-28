//! View Definition sub-tab rendering

use crate::app::App;
use crate::ui::theme;
use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn draw_definition_content(frame: &mut Frame, app: &App, area: Rect) {
    if let Some(table) = app.selected_table_info() {
        if !table.table_type.is_view() {
            let msg =
                Paragraph::new("Definition is only available for Views and Materialized Views")
                    .style(theme::muted());
            frame.render_widget(msg, area);
            return;
        }

        if let Some(definition) = &table.view_definition {
            // Display the view definition (SELECT statement) with syntax-like coloring
            let lines: Vec<Line> = definition
                .lines()
                .map(|line| {
                    // Simple SQL keyword highlighting
                    let styled_line = highlight_sql_line(line);
                    Line::from(styled_line)
                })
                .collect();

            let paragraph = Paragraph::new(lines)
                .style(theme::text())
                .wrap(ratatui::widgets::Wrap { trim: false });
            frame.render_widget(paragraph, area);
        } else {
            let empty = Paragraph::new("View definition not loaded").style(theme::muted());
            frame.render_widget(empty, area);
        }
    } else {
        let empty = Paragraph::new("Select a view to see its definition").style(theme::muted());
        frame.render_widget(empty, area);
    }
}

/// Simple SQL keyword highlighting for view definitions
/// Uses string slices to avoid unnecessary allocations
fn highlight_sql_line(line: &str) -> Vec<Span<'static>> {
    const KEYWORDS: &[&str] = &[
        "SELECT",
        "FROM",
        "WHERE",
        "AND",
        "OR",
        "NOT",
        "IN",
        "IS",
        "NULL",
        "JOIN",
        "LEFT",
        "RIGHT",
        "INNER",
        "OUTER",
        "ON",
        "AS",
        "ORDER",
        "BY",
        "GROUP",
        "HAVING",
        "LIMIT",
        "OFFSET",
        "UNION",
        "ALL",
        "DISTINCT",
        "CREATE",
        "VIEW",
        "MATERIALIZED",
        "WITH",
        "CASE",
        "WHEN",
        "THEN",
        "ELSE",
        "END",
        "TRUE",
        "FALSE",
        "LIKE",
        "ILIKE",
        "BETWEEN",
        "EXISTS",
        "CAST",
        "COALESCE",
    ];

    let mut spans = Vec::new();
    let mut pos = 0;
    let bytes = line.as_bytes();

    while pos < line.len() {
        let remaining = &line[pos..];

        // Check for string literals (single or double quotes)
        if remaining.starts_with('\'') || remaining.starts_with('"') {
            let quote_char = remaining.chars().next().unwrap();
            let mut end_pos = 1;
            while end_pos < remaining.len() {
                if remaining[end_pos..].starts_with(quote_char) {
                    end_pos += 1;
                    break;
                }
                end_pos += 1;
            }
            spans.push(Span::styled(
                remaining[..end_pos].to_string(),
                theme::header(), // String literals use accent color (Yellow)
            ));
            pos += end_pos;
            continue;
        }

        // Check for keywords (case-insensitive)
        let mut found = false;
        for keyword in KEYWORDS {
            if remaining.len() >= keyword.len()
                && remaining[..keyword.len()].eq_ignore_ascii_case(keyword)
            {
                // Verify it's a complete word
                let next_pos = pos + keyword.len();
                let is_word_boundary = next_pos >= line.len()
                    || !bytes[next_pos].is_ascii_alphanumeric() && bytes[next_pos] != b'_';

                if is_word_boundary {
                    spans.push(Span::styled(
                        remaining[..keyword.len()].to_string(),
                        theme::selected().add_modifier(Modifier::BOLD), // SQL keywords use Cyan with bold (no background)
                    ));
                    pos += keyword.len();
                    found = true;
                    break;
                }
            }
        }

        if !found {
            // Handle single character
            let ch = remaining.chars().next().unwrap();
            let style = if ch.is_ascii_digit() {
                theme::header() // Numbers use accent color (Yellow)
            } else {
                theme::text() // Regular text
            };
            spans.push(Span::styled(ch.to_string(), style));
            pos += ch.len_utf8();
        }
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_span_content(spans: &[Span]) -> Vec<String> {
        spans.iter().map(|s| s.content.to_string()).collect()
    }

    #[test]
    fn test_highlight_sql_keyword() {
        let spans = highlight_sql_line("SELECT");
        assert_eq!(get_span_content(&spans), vec!["SELECT"]);
    }

    #[test]
    fn test_highlight_sql_keyword_lowercase() {
        let spans = highlight_sql_line("select");
        assert_eq!(get_span_content(&spans), vec!["select"]);
    }

    #[test]
    fn test_highlight_sql_keyword_mixed_case() {
        let spans = highlight_sql_line("Select");
        assert_eq!(get_span_content(&spans), vec!["Select"]);
    }

    #[test]
    fn test_highlight_partial_keyword_not_matched() {
        // "SELECTING" should not highlight "SELECT" as a keyword
        let spans = highlight_sql_line("SELECTING");
        let content = get_span_content(&spans);
        // Should be individual characters since SELECTING is not a keyword
        assert_eq!(content.join(""), "SELECTING");
        assert!(content.len() > 1); // Not a single span
    }

    #[test]
    fn test_highlight_string_literal_single_quote() {
        let spans = highlight_sql_line("'hello world'");
        assert_eq!(get_span_content(&spans), vec!["'hello world'"]);
    }

    #[test]
    fn test_highlight_string_literal_double_quote() {
        let spans = highlight_sql_line("\"column_name\"");
        assert_eq!(get_span_content(&spans), vec!["\"column_name\""]);
    }

    #[test]
    fn test_highlight_numeric_literal() {
        let spans = highlight_sql_line("123");
        assert_eq!(get_span_content(&spans), vec!["1", "2", "3"]);
    }

    #[test]
    fn test_highlight_mixed_sql() {
        let spans = highlight_sql_line("SELECT * FROM users");
        let content = get_span_content(&spans);
        assert!(content.contains(&"SELECT".to_string()));
        assert!(content.contains(&"FROM".to_string()));
    }

    #[test]
    fn test_highlight_keyword_with_underscore_suffix() {
        // "SELECT_QUERY" should not match SELECT as keyword
        let spans = highlight_sql_line("SELECT_QUERY");
        let content = get_span_content(&spans);
        assert_eq!(content.join(""), "SELECT_QUERY");
        assert!(content.len() > 1);
    }
}
