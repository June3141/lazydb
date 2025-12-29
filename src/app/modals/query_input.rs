//! Query input modal for executing arbitrary SQL queries

/// Modal state for query input
#[derive(Debug, Clone, Default)]
pub struct QueryInputModal {
    /// The SQL query text being edited
    pub query: String,
    /// Cursor position (character index)
    pub cursor_pos: usize,
}

impl QueryInputModal {
    /// Create a new query input modal with optional initial query
    pub fn new(initial_query: &str) -> Self {
        let query = initial_query.to_string();
        let cursor_pos = query.len();
        Self { query, cursor_pos }
    }

    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, c: char) {
        self.query.insert(self.cursor_pos, c);
        self.cursor_pos += c.len_utf8();
    }

    /// Insert a newline at the current cursor position
    pub fn insert_newline(&mut self) {
        self.insert_char('\n');
    }

    /// Delete the character before the cursor (backspace)
    pub fn delete_char_before(&mut self) {
        if self.cursor_pos > 0 {
            // Find the previous character boundary
            let prev_pos = self.query[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.query.remove(prev_pos);
            self.cursor_pos = prev_pos;
        }
    }

    /// Delete the character at the cursor (delete key)
    pub fn delete_char_at(&mut self) {
        if self.cursor_pos < self.query.len() {
            self.query.remove(self.cursor_pos);
        }
    }

    /// Move cursor left by one character
    pub fn move_cursor_left(&mut self) {
        if self.cursor_pos > 0 {
            self.cursor_pos = self.query[..self.cursor_pos]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    /// Move cursor right by one character
    pub fn move_cursor_right(&mut self) {
        if self.cursor_pos < self.query.len() {
            self.cursor_pos = self.query[self.cursor_pos..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor_pos + i)
                .unwrap_or(self.query.len());
        }
    }

    /// Move cursor to the beginning of the current line
    pub fn move_cursor_home(&mut self) {
        // Find the start of the current line
        self.cursor_pos = self.query[..self.cursor_pos]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
    }

    /// Move cursor to the end of the current line
    pub fn move_cursor_end(&mut self) {
        // Find the end of the current line
        self.cursor_pos = self.query[self.cursor_pos..]
            .find('\n')
            .map(|i| self.cursor_pos + i)
            .unwrap_or(self.query.len());
    }

    /// Move cursor up one line
    pub fn move_cursor_up(&mut self) {
        let current_line_start = self.query[..self.cursor_pos]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let col = self.cursor_pos - current_line_start;

        if current_line_start > 0 {
            // Find the previous line
            let prev_line_end = current_line_start - 1;
            let prev_line_start = self.query[..prev_line_end]
                .rfind('\n')
                .map(|i| i + 1)
                .unwrap_or(0);
            let prev_line_len = prev_line_end - prev_line_start;

            self.cursor_pos = prev_line_start + col.min(prev_line_len);
        }
    }

    /// Move cursor down one line
    pub fn move_cursor_down(&mut self) {
        let current_line_start = self.query[..self.cursor_pos]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let col = self.cursor_pos - current_line_start;

        if let Some(newline_pos) = self.query[self.cursor_pos..].find('\n') {
            let next_line_start = self.cursor_pos + newline_pos + 1;
            let next_line_end = self.query[next_line_start..]
                .find('\n')
                .map(|i| next_line_start + i)
                .unwrap_or(self.query.len());
            let next_line_len = next_line_end - next_line_start;

            self.cursor_pos = next_line_start + col.min(next_line_len);
        }
    }

    /// Clear all query text
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_pos = 0;
    }

    /// Get the current query text
    pub fn get_query(&self) -> &str {
        &self.query
    }

    /// Check if the query is empty or contains only whitespace
    pub fn is_empty(&self) -> bool {
        self.query.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_empty() {
        let modal = QueryInputModal::default();
        assert_eq!(modal.query, "");
        assert_eq!(modal.cursor_pos, 0);
    }

    #[test]
    fn test_new_with_initial_query() {
        let modal = QueryInputModal::new("SELECT * FROM users");
        assert_eq!(modal.query, "SELECT * FROM users");
        assert_eq!(modal.cursor_pos, 19); // cursor at end
    }

    #[test]
    fn test_insert_char() {
        let mut modal = QueryInputModal::default();
        modal.insert_char('S');
        modal.insert_char('E');
        modal.insert_char('L');
        assert_eq!(modal.query, "SEL");
        assert_eq!(modal.cursor_pos, 3);
    }

    #[test]
    fn test_insert_newline() {
        let mut modal = QueryInputModal::new("SELECT *");
        modal.insert_newline();
        modal.insert_char('F');
        assert_eq!(modal.query, "SELECT *\nF");
    }

    #[test]
    fn test_delete_char_before() {
        let mut modal = QueryInputModal::new("SELECT");
        modal.delete_char_before();
        assert_eq!(modal.query, "SELEC");
        assert_eq!(modal.cursor_pos, 5);
    }

    #[test]
    fn test_delete_char_before_at_start() {
        let mut modal = QueryInputModal::default();
        modal.delete_char_before(); // should do nothing
        assert_eq!(modal.query, "");
        assert_eq!(modal.cursor_pos, 0);
    }

    #[test]
    fn test_delete_char_at() {
        let mut modal = QueryInputModal::new("SELECT");
        modal.cursor_pos = 0;
        modal.delete_char_at();
        assert_eq!(modal.query, "ELECT");
    }

    #[test]
    fn test_move_cursor_left() {
        let mut modal = QueryInputModal::new("ABC");
        assert_eq!(modal.cursor_pos, 3);
        modal.move_cursor_left();
        assert_eq!(modal.cursor_pos, 2);
        modal.move_cursor_left();
        assert_eq!(modal.cursor_pos, 1);
    }

    #[test]
    fn test_move_cursor_left_at_start() {
        let mut modal = QueryInputModal::new("ABC");
        modal.cursor_pos = 0;
        modal.move_cursor_left(); // should stay at 0
        assert_eq!(modal.cursor_pos, 0);
    }

    #[test]
    fn test_move_cursor_right() {
        let mut modal = QueryInputModal::new("ABC");
        modal.cursor_pos = 0;
        modal.move_cursor_right();
        assert_eq!(modal.cursor_pos, 1);
        modal.move_cursor_right();
        assert_eq!(modal.cursor_pos, 2);
    }

    #[test]
    fn test_move_cursor_right_at_end() {
        let mut modal = QueryInputModal::new("ABC");
        modal.move_cursor_right(); // should stay at end
        assert_eq!(modal.cursor_pos, 3);
    }

    #[test]
    fn test_move_cursor_home() {
        let mut modal = QueryInputModal::new("SELECT *\nFROM users");
        modal.cursor_pos = 14; // in the middle of "FROM"
        modal.move_cursor_home();
        assert_eq!(modal.cursor_pos, 9); // start of "FROM users"
    }

    #[test]
    fn test_move_cursor_end() {
        let mut modal = QueryInputModal::new("SELECT *\nFROM users");
        modal.cursor_pos = 9; // start of "FROM users"
        modal.move_cursor_end();
        assert_eq!(modal.cursor_pos, 19); // end of query
    }

    #[test]
    fn test_move_cursor_up() {
        let mut modal = QueryInputModal::new("SELECT *\nFROM users");
        modal.cursor_pos = 14; // "FROM" (col 5 of line 2)
        modal.move_cursor_up();
        assert_eq!(modal.cursor_pos, 5); // "CT *" (col 5 of line 1)
    }

    #[test]
    fn test_move_cursor_down() {
        let mut modal = QueryInputModal::new("SELECT *\nFROM users");
        modal.cursor_pos = 3; // "ECT" (col 3 of line 1)
        modal.move_cursor_down();
        assert_eq!(modal.cursor_pos, 12); // "OM" (col 3 of line 2)
    }

    #[test]
    fn test_clear() {
        let mut modal = QueryInputModal::new("SELECT * FROM users");
        modal.clear();
        assert_eq!(modal.query, "");
        assert_eq!(modal.cursor_pos, 0);
    }

    #[test]
    fn test_is_empty() {
        let modal = QueryInputModal::default();
        assert!(modal.is_empty());

        let modal = QueryInputModal::new("   ");
        assert!(modal.is_empty());

        let modal = QueryInputModal::new("SELECT");
        assert!(!modal.is_empty());
    }

    #[test]
    fn test_insert_in_middle() {
        let mut modal = QueryInputModal::new("SELCT");
        modal.cursor_pos = 3; // after "SEL"
        modal.insert_char('E');
        assert_eq!(modal.query, "SELECT");
        assert_eq!(modal.cursor_pos, 4);
    }

    #[test]
    fn test_unicode_handling() {
        let mut modal = QueryInputModal::default();
        modal.insert_char('日');
        modal.insert_char('本');
        modal.insert_char('語');
        assert_eq!(modal.query, "日本語");
        modal.delete_char_before();
        assert_eq!(modal.query, "日本");
    }
}
