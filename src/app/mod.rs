pub mod error;

use crate::query::JsonData;
pub use error::AppError;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub prompt: &'static str,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prompt: "query > ",
        }
    }
}

#[derive(Debug, Default)]
pub struct AppState {
    pub input: String,
    pub exit: bool,
    pub last_error: Option<AppError>,
}

#[derive(Debug)]
pub struct App {
    config: AppConfig,
    state: AppState,
    data: JsonData,
}

impl App {
    pub fn new(json_value: serde_json::Value) -> Self {
        Self {
            config: AppConfig::default(),
            state: AppState::default(),
            data: JsonData::new(json_value),
        }
    }

    // 状態アクセサ
    pub fn input(&self) -> &str {
        &self.state.input
    }

    pub fn should_exit(&self) -> bool {
        self.state.exit
    }

    pub fn prompt(&self) -> &str {
        self.config.prompt
    }

    pub fn last_error(&self) -> Option<&AppError> {
        self.state.last_error.as_ref()
    }

    pub fn data(&self) -> &JsonData {
        &self.data
    }

    // 状態変更
    pub fn set_exit(&mut self, exit: bool) {
        self.state.exit = exit;
    }

    pub fn clear_input(&mut self) {
        self.state.input.clear();
        self.state.last_error = None;
    }

    pub fn push_char(&mut self, c: char) {
        self.state.input.push(c);
    }

    pub fn pop_char(&mut self) {
        self.state.input.pop();
    }

    // クエリ実行（計算結果を返すのみ、状態には保存しない）
    pub fn execute_current_query(&self) -> crate::Result<crate::query::QueryResult> {
        self.data.execute_query(&self.state.input)
    }
}