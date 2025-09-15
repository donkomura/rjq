pub mod error;
pub mod config;
pub mod state;
pub mod builder;

use crate::query::JsonData;
pub use error::AppError;
pub use config::AppConfig;
pub use state::AppState;
pub use builder::{AppBuilder, EnhancedApp};

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

    // クエリ実行（計算結果を返すのみ、状態には保存しない）
    pub fn execute_current_query(&self) -> crate::Result<crate::query::QueryResult> {
        self.data.execute_query(&self.state.input)
    }
}