use crate::app::error::AppError;

#[derive(Debug, Default)]
pub struct AppState {
    pub input: String,
    pub exit: bool,
    pub last_error: Option<AppError>,
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
}