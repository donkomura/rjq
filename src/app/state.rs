use crate::app::error::AppError;

#[derive(Debug, Default)]
pub struct AppState {
    pub input: String,
    pub exit: bool,
    pub last_error: Option<AppError>,
    pub scroll_offset: usize,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
        self.last_error = None;
    }

    pub fn push_char(&mut self, c: char) {
        self.input.push(c);
    }

    pub fn pop_char(&mut self) {
        self.input.pop();
    }

    pub fn set_exit(&mut self, exit: bool) {
        self.exit = exit;
    }

    pub fn set_error(&mut self, error: AppError) {
        self.last_error = Some(error);
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down_bounded(&mut self, total_lines: usize, visible_height: usize) {
        let max_scroll = total_lines.saturating_sub(visible_height);

        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
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
        let mut state = AppState::new();
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
        let mut state = AppState::new();
        assert!(!state.exit);

        state.set_exit(true);
        assert!(state.exit);
    }

    #[test]
    fn test_scroll_operations() {
        let mut state = AppState::new();
        assert_eq!(state.scroll_offset, 0);

        // Test scroll up (at boundary should not go below 0)
        state.scroll_up();
        assert_eq!(state.scroll_offset, 0);

        // Test bounded scroll down
        state.scroll_down_bounded(10, 5);
        assert_eq!(state.scroll_offset, 1);

        state.scroll_down_bounded(10, 5);
        assert_eq!(state.scroll_offset, 2);

        // Test scroll up
        state.scroll_up();
        assert_eq!(state.scroll_offset, 1);

        state.scroll_up();
        assert_eq!(state.scroll_offset, 0);

        // Test reset scroll
        state.scroll_down_bounded(10, 5);
        state.scroll_down_bounded(10, 5);
        state.reset_scroll();
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_scroll_with_bounds() {
        let mut state = AppState::new();

        // Test with sufficient content (10 lines, 3 visible)
        // max_scroll = max(0, 10 - 3) = 7
        let total_lines = 10;
        let visible_height = 3;

        // Should scroll down normally within bounds
        for i in 1..=7 {
            state.scroll_down_bounded(total_lines, visible_height);
            assert_eq!(state.scroll_offset, i);
        }

        // Should not exceed max_scroll (7)
        state.scroll_down_bounded(total_lines, visible_height);
        assert_eq!(state.scroll_offset, 7);

        state.scroll_down_bounded(total_lines, visible_height);
        assert_eq!(state.scroll_offset, 7); // Still 7, not 8
    }

    #[test]
    fn test_scroll_bounds_edge_cases() {
        let mut state = AppState::new();

        // Case 1: Content shorter than visible area
        // max_scroll = max(0, 2 - 5) = 0
        state.scroll_down_bounded(2, 5);
        assert_eq!(state.scroll_offset, 0); // Should not scroll

        // Case 2: Content exactly matches visible area
        // max_scroll = max(0, 5 - 5) = 0
        state.reset_scroll();
        state.scroll_down_bounded(5, 5);
        assert_eq!(state.scroll_offset, 0); // Should not scroll

        // Case 3: Content one line longer than visible area
        // max_scroll = max(0, 6 - 5) = 1
        state.reset_scroll();
        state.scroll_down_bounded(6, 5);
        assert_eq!(state.scroll_offset, 1);

        state.scroll_down_bounded(6, 5);
        assert_eq!(state.scroll_offset, 1); // Should not exceed 1
    }

    #[test]
    fn test_scroll_up_down_integration() {
        let mut state = AppState::new();
        let total_lines = 10;
        let visible_height = 3;
        // max_scroll = 10 - 3 = 7

        // Scroll down to middle position
        for _ in 0..4 {
            state.scroll_down_bounded(total_lines, visible_height);
        }
        assert_eq!(state.scroll_offset, 4);

        // Should be able to scroll up normally
        state.scroll_up();
        assert_eq!(state.scroll_offset, 3);

        state.scroll_up();
        assert_eq!(state.scroll_offset, 2);

        // Continue scrolling up to beginning
        state.scroll_up();
        assert_eq!(state.scroll_offset, 1);

        state.scroll_up();
        assert_eq!(state.scroll_offset, 0);

        // Should not go below 0
        state.scroll_up();
        assert_eq!(state.scroll_offset, 0);

        // Should be able to scroll down again
        state.scroll_down_bounded(total_lines, visible_height);
        assert_eq!(state.scroll_offset, 1);
    }
}
