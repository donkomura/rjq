use super::error::AppError;

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

    pub fn set_error(&mut self, error: AppError) {
        self.last_error = Some(error);
    }

    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    pub fn scroll_down_bounded(&mut self, total_lines: usize, visible_height: usize) {
        let max_scroll = total_lines.saturating_sub(visible_height);

        if max_scroll > 0 && self.scroll_offset < max_scroll {
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
    fn test_new_state() {
        let state = AppState::new();
        assert_eq!(state.input, "");
        assert!(!state.exit);
        assert!(state.last_error.is_none());
        assert_eq!(state.scroll_offset, 0);
    }

    #[test]
    fn test_input_operations() {
        let mut state = AppState::new();
        assert_eq!(state.input, "");

        state.push_char('h');
        state.push_char('i');
        assert_eq!(state.input, "hi");

        state.pop_char();
        assert_eq!(state.input, "h");

        state.clear_input();
        assert_eq!(state.input, "");
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
    fn test_exit_flag() {
        let mut state = AppState::new();
        assert!(!state.exit);

        state.set_exit(true);
        assert!(state.exit);

        state.set_exit(false);
        assert!(!state.exit);
    }
}