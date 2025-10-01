use super::error::AppError;
use crate::history::QueryHistory;

#[derive(Debug)]
#[derive(Default)]
pub struct AppState {
    pub input: String,
    pub exit: bool,
    pub last_error: Option<AppError>,
    pub scroll_offset: usize,
    pub query_history: QueryHistory,
}

impl AppState {
    pub fn set_exit(&mut self, exit: bool) {
        self.exit = exit;
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn pop_char(&mut self) {
        self.input.pop();
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    /// Scroll down with bounds checking based on content and visible area
    pub fn scroll_down_bounded(&mut self, total_lines: usize, visible_height: usize) {
        if total_lines > visible_height {
            let max_scroll = total_lines - visible_height;
            self.scroll_offset = (self.scroll_offset + 1).min(max_scroll);
        }
    }

    pub fn reset_scroll(&mut self) {
        self.scroll_offset = 0;
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_input_operations() {
        let mut state = AppState::default();
        assert_eq!(state.input, "");

        state.push_char('a');
        state.push_char('b');
        assert_eq!(state.input, "ab");

        state.pop_char();
        assert_eq!(state.input, "a");

        state.clear_input();
        assert_eq!(state.input, "");
    }

    #[test]
    fn test_state_exit_flag() {
        let mut state = AppState::default();
        assert!(!state.exit);

        state.set_exit(true);
        assert!(state.exit);

        state.set_exit(false);
        assert!(!state.exit);
    }

    #[test]
    fn test_scroll_operations() {
        let mut state = AppState::default();
        assert_eq!(state.scroll_offset, 0);

        // Test scroll up from 0 (should stay at 0)
        state.scroll_up();
        assert_eq!(state.scroll_offset, 0);

        // Test scroll down with bounds
        state.scroll_down_bounded(50, 20); // 50 total lines, 20 visible
        assert_eq!(state.scroll_offset, 1);

        state.scroll_down_bounded(50, 20);
        assert_eq!(state.scroll_offset, 2);

        // Test reset
        state.reset_scroll();
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_with_bounds() {
        let mut state = AppState::default();
        let total_lines = 25;
        let visible_height = 20;
        let max_scroll = total_lines - visible_height; // 5

        // Scroll to maximum
        for _ in 0..10 {
            state.scroll_down_bounded(total_lines, visible_height);
        }
        assert_eq!(state.scroll_offset, max_scroll);

        // Try to scroll beyond maximum (should stay at max)
        state.scroll_down_bounded(total_lines, visible_height);
        assert_eq!(state.scroll_offset, max_scroll);
    }

    #[test]
    fn test_scroll_bounds_edge_cases() {
        let mut state = AppState::default();

        // Case: content fits in visible area (no scrolling allowed)
        state.scroll_down_bounded(10, 20); // 10 lines, 20 visible
        assert_eq!(state.scroll_offset, 0);

        // Case: content exactly fits (no scrolling allowed)
        state.scroll_down_bounded(20, 20); // 20 lines, 20 visible
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_up_down_integration() {
        let mut state = AppState::default();
        let total_lines = 30;
        let visible_height = 20;

        // Scroll down a few times
        state.scroll_down_bounded(total_lines, visible_height);
        state.scroll_down_bounded(total_lines, visible_height);
        assert_eq!(state.scroll_offset, 2);

        // Scroll up once
        state.scroll_up();
        assert_eq!(state.scroll_offset, 1);

        // Scroll up to 0
        state.scroll_up();
        assert_eq!(state.scroll_offset, 0);

        // Should be able to scroll down again
        state.scroll_down_bounded(total_lines, visible_height);
        assert_eq!(state.scroll_offset, 1);
    }
}
