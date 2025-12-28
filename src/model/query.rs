#[derive(Debug, Clone)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub execution_time_ms: u64,
    #[allow(dead_code)]
    pub total_rows: usize,
}

/// Available page sizes for pagination
pub const PAGE_SIZES: [usize; 3] = [50, 100, 500];

/// Pagination state for data display
#[derive(Debug, Clone)]
pub struct Pagination {
    pub current_page: usize,
    pub page_size: usize,
    pub total_rows: usize,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            current_page: 0,
            page_size: PAGE_SIZES[0],
            total_rows: 0,
        }
    }
}

impl Pagination {
    pub fn new(total_rows: usize) -> Self {
        Self {
            current_page: 0,
            page_size: PAGE_SIZES[0],
            total_rows,
        }
    }

    /// Total number of pages
    pub fn total_pages(&self) -> usize {
        if self.total_rows == 0 {
            1
        } else {
            self.total_rows.div_ceil(self.page_size)
        }
    }

    /// Go to next page if possible
    pub fn next_page(&mut self) {
        if self.current_page + 1 < self.total_pages() {
            self.current_page += 1;
        }
    }

    /// Go to previous page if possible
    pub fn prev_page(&mut self) {
        if self.current_page > 0 {
            self.current_page -= 1;
        }
    }

    /// Go to first page
    pub fn first_page(&mut self) {
        self.current_page = 0;
    }

    /// Go to last page
    pub fn last_page(&mut self) {
        self.current_page = self.total_pages().saturating_sub(1);
    }

    /// Cycle to next page size
    pub fn cycle_page_size(&mut self) {
        let current_idx = PAGE_SIZES
            .iter()
            .position(|&s| s == self.page_size)
            .unwrap_or(0);
        let next_idx = (current_idx + 1) % PAGE_SIZES.len();
        self.page_size = PAGE_SIZES[next_idx];
        // Reset to first page when changing page size
        self.current_page = 0;
    }

    /// Get start index for current page
    pub fn start_index(&self) -> usize {
        self.current_page * self.page_size
    }

    /// Get end index for current page (exclusive)
    pub fn end_index(&self) -> usize {
        std::cmp::min(self.start_index() + self.page_size, self.total_rows)
    }

    /// Check if there is a next page
    pub fn has_next(&self) -> bool {
        self.current_page + 1 < self.total_pages()
    }

    /// Check if there is a previous page
    pub fn has_prev(&self) -> bool {
        self.current_page > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_default() {
        let p = Pagination::default();
        assert_eq!(p.current_page, 0);
        assert_eq!(p.page_size, 50);
        assert_eq!(p.total_rows, 0);
    }

    #[test]
    fn test_pagination_new() {
        let p = Pagination::new(250);
        assert_eq!(p.current_page, 0);
        assert_eq!(p.page_size, 50);
        assert_eq!(p.total_rows, 250);
        assert_eq!(p.total_pages(), 5);
    }

    #[test]
    fn test_pagination_total_pages() {
        // Empty result
        let p = Pagination::new(0);
        assert_eq!(p.total_pages(), 1);

        // Exact multiple
        let p = Pagination::new(100);
        assert_eq!(p.total_pages(), 2);

        // Not exact multiple
        let p = Pagination::new(125);
        assert_eq!(p.total_pages(), 3);
    }

    #[test]
    fn test_pagination_navigation() {
        let mut p = Pagination::new(150);
        assert_eq!(p.total_pages(), 3);

        // Initial state
        assert!(!p.has_prev());
        assert!(p.has_next());

        // Go to next page
        p.next_page();
        assert_eq!(p.current_page, 1);
        assert!(p.has_prev());
        assert!(p.has_next());

        // Go to last page
        p.next_page();
        assert_eq!(p.current_page, 2);
        assert!(p.has_prev());
        assert!(!p.has_next());

        // Try to go beyond last page (should stay)
        p.next_page();
        assert_eq!(p.current_page, 2);

        // Go back
        p.prev_page();
        assert_eq!(p.current_page, 1);

        // Go to first
        p.first_page();
        assert_eq!(p.current_page, 0);

        // Go to last
        p.last_page();
        assert_eq!(p.current_page, 2);
    }

    #[test]
    fn test_pagination_indices() {
        let mut p = Pagination::new(125);

        // First page
        assert_eq!(p.start_index(), 0);
        assert_eq!(p.end_index(), 50);

        // Second page
        p.next_page();
        assert_eq!(p.start_index(), 50);
        assert_eq!(p.end_index(), 100);

        // Third page (partial)
        p.next_page();
        assert_eq!(p.start_index(), 100);
        assert_eq!(p.end_index(), 125);
    }

    #[test]
    fn test_pagination_cycle_page_size() {
        let mut p = Pagination::new(500);
        assert_eq!(p.page_size, 50);

        p.cycle_page_size();
        assert_eq!(p.page_size, 100);
        assert_eq!(p.current_page, 0); // Reset to first page

        p.cycle_page_size();
        assert_eq!(p.page_size, 500);

        p.cycle_page_size();
        assert_eq!(p.page_size, 50); // Cycles back
    }
}
