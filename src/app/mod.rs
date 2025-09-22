pub mod builder;
pub mod config;
pub mod error;
pub mod state;

use crate::query::JsonData;
pub use builder::{AppBuilder, EnhancedApp};
pub use config::AppConfig;
pub use error::AppError;
pub use state::AppState;

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

    pub fn with_config(json_value: serde_json::Value, config: AppConfig) -> Self {
        Self {
            config,
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

    pub fn scroll_offset(&self) -> usize {
        self.state.scroll_offset
    }

    // 状態変更（AppStateに委譲）
    pub fn set_exit(&mut self, exit: bool) {
        self.state.set_exit(exit);
    }

    pub fn clear_input(&mut self) {
        self.state.clear_input();
    }

    pub fn push_char(&mut self, c: char) {
        self.state.push_char(c);
    }

    pub fn pop_char(&mut self) {
        self.state.pop_char();
    }

    pub fn scroll_up(&mut self) {
        self.state.scroll_up();
    }

    pub fn scroll_down(&mut self) {
        // Apply dynamic bounds based on current content
        let content = match self.execute_current_query() {
            Ok(result) => result.format_pretty(),
            Err(_) => {
                if self.input().is_empty() {
                    serde_json::to_string_pretty(self.data.get())
                        .unwrap_or_else(|_| "Error formatting JSON".to_string())
                } else {
                    "".to_string()
                }
            }
        };

        let total_lines = content.lines().count();
        // Use a reasonable default for visible height (will be overridden by UI)
        let default_visible_height = 20;
        self.state.scroll_down_bounded(total_lines, default_visible_height);
    }

    pub fn scroll_down_with_content(&mut self, content: &str, visible_height: usize) {
        let total_lines = content.lines().count();
        self.state.scroll_down_bounded(total_lines, visible_height);
    }

    pub fn reset_scroll(&mut self) {
        self.state.reset_scroll();
    }

    // クエリ実行（計算結果を返すのみ、状態には保存しない）
    pub fn execute_current_query(&self) -> crate::Result<crate::query::QueryResult> {
        self.data.execute_query(&self.state.input)
    }
}
