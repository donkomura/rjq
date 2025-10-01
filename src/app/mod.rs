pub mod builder;
pub mod config;
pub mod error;
pub mod state;

use crate::query::JsonData;
pub use builder::{AppBuilder, EnhancedApp};
pub use config::AppConfig;
pub use error::AppError;
pub use state::AppState;

/// コンテンツ生成のための共通トレイト
pub trait ContentGenerator {
    /// 現在のコンテンツを生成する
    fn generate_current_content(&self) -> String;

    /// スクロール可能なコンテンツの行数を取得する
    fn get_total_lines(&self) -> usize;
}

#[derive(Debug)]
pub struct App {
    config: AppConfig,
    state: AppState,
    data: JsonData,
}

impl ContentGenerator for App {
    fn generate_current_content(&self) -> String {
        match self.execute_current_query() {
            Ok(result) => result.format_pretty(),
            Err(_) => {
                if self.input().is_empty() {
                    serde_json::to_string_pretty(self.data.get())
                        .unwrap_or_else(|_| "Error formatting JSON".to_string())
                } else {
                    "".to_string()
                }
            }
        }
    }

    fn get_total_lines(&self) -> usize {
        self.generate_current_content().lines().count()
    }
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
        let total_lines = self.get_total_lines();
        let visible_height = self.config.visible_height;
        self.state.scroll_down_bounded(total_lines, visible_height);
    }

    pub fn reset_scroll(&mut self) {
        self.state.reset_scroll();
    }

    // クエリ実行（計算結果を返すのみ、状態には保存しない）
    pub fn execute_current_query(&self) -> crate::Result<crate::query::QueryResult> {
        self.data.execute_query(&self.state.input)
    }

    // 候補機能
    pub fn get_best_suggestion(&self) -> Option<String> {
        if self.state.input.len() < 2 {
            return None;
        }

        let suggestions = self
            .state
            .query_history
            .get_suggestions(&self.state.input, 1);
        suggestions.first().map(|s| s.text.clone())
    }

    pub fn apply_suggestion(&mut self, suggestion: String) {
        self.state.input = suggestion;
    }

    pub fn record_query(&mut self, query: String) {
        self.state.query_history.record_query(query);
    }
}
