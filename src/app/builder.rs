use super::{AppConfig, AppState, ContentGenerator};
use crate::query::JsonData;
use crate::query::{CachedQueryExecutor, InMemoryQueryCache, JaqQueryExecutor, QueryExecutor};
use crate::ui::{DefaultEventHandler, EventHandler};

pub struct AppBuilder<Q, E>
where
    Q: QueryExecutor,
    E: EventHandler,
{
    json_value: serde_json::Value,
    config: AppConfig,
    query_executor: Q,
    event_handler: E,
}

impl AppBuilder<JaqQueryExecutor, DefaultEventHandler> {
    pub fn new(json_value: serde_json::Value) -> Self {
        Self {
            json_value,
            config: AppConfig::default(),
            query_executor: JaqQueryExecutor,
            event_handler: DefaultEventHandler,
        }
    }
}

impl<Q, E> AppBuilder<Q, E>
where
    Q: QueryExecutor,
    E: EventHandler,
{
    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_query_executor<Q2: QueryExecutor>(self, executor: Q2) -> AppBuilder<Q2, E> {
        AppBuilder {
            json_value: self.json_value,
            config: self.config,
            query_executor: executor,
            event_handler: self.event_handler,
        }
    }

    pub fn with_event_handler<E2: EventHandler>(self, handler: E2) -> AppBuilder<Q, E2> {
        AppBuilder {
            json_value: self.json_value,
            config: self.config,
            query_executor: self.query_executor,
            event_handler: handler,
        }
    }

    pub fn with_cache(self) -> AppBuilder<CachedQueryExecutor<Q, InMemoryQueryCache>, E> {
        let cached_executor =
            CachedQueryExecutor::new(self.query_executor, InMemoryQueryCache::new());
        AppBuilder {
            json_value: self.json_value,
            config: self.config,
            query_executor: cached_executor,
            event_handler: self.event_handler,
        }
    }

    pub fn build(self) -> EnhancedApp<Q, E> {
        EnhancedApp {
            config: self.config,
            state: AppState::default(),
            data: JsonData::new(self.json_value),
            query_executor: self.query_executor,
            event_handler: self.event_handler,
        }
    }
}

pub struct EnhancedApp<Q: QueryExecutor, E: EventHandler> {
    config: AppConfig,
    state: AppState,
    data: JsonData,
    query_executor: Q,
    event_handler: E,
}

impl<Q: QueryExecutor, E: EventHandler> ContentGenerator for EnhancedApp<Q, E> {
    fn generate_current_content(&self) -> String {
        match self.execute_current_query() {
            Ok(result) => result.format_pretty(),
            Err(_) => {
                if self.state.input.is_empty() {
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

impl<Q: QueryExecutor, E: EventHandler> EnhancedApp<Q, E> {
    // 既存のApp APIと互換性を保つメソッド群
    pub fn input(&self) -> &str {
        &self.state.input
    }

    pub fn should_exit(&self) -> bool {
        self.state.exit
    }

    pub fn prompt(&self) -> &str {
        self.config.prompt
    }

    pub fn last_error(&self) -> Option<&crate::app::error::AppError> {
        self.state.last_error.as_ref()
    }

    pub fn data(&self) -> &JsonData {
        &self.data
    }

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

    pub fn scroll_offset(&self) -> usize {
        self.state.scroll_offset
    }

    // 強化されたクエリ実行メソッド（依存性注入されたExecutorを使用）
    pub fn execute_current_query(&self) -> crate::Result<crate::query::QueryResult> {
        if self.state.input.is_empty() {
            return Err(crate::app::error::AppError::QueryCompile(
                "Empty query".to_string(),
            ));
        }

        let results = self
            .query_executor
            .execute(self.data.get(), &self.state.input)?;

        Ok(match results.len() {
            0 => crate::query::QueryResult::Empty,
            1 => crate::query::QueryResult::Single(results.into_iter().next().unwrap()),
            _ => crate::query::QueryResult::Multiple(results),
        })
    }

    // イベント処理メソッド（依存性注入されたEventHandlerを使用）
    pub fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        let action = self.event_handler.handle_key_event(key_event);
        self.update_with_action(action);
    }

    fn update_with_action(&mut self, action: crate::ui::Action) {
        match action {
            crate::ui::Action::Quit => self.set_exit(true),
            crate::ui::Action::Input(c) => {
                self.push_char(c);
                self.reset_scroll();
            }
            crate::ui::Action::Backspace => {
                if !self.input().is_empty() {
                    self.pop_char();
                }
                self.reset_scroll();
            }
            crate::ui::Action::Clear => {
                self.clear_input();
                self.reset_scroll();
            }
            crate::ui::Action::ScrollUp => self.scroll_up(),
            crate::ui::Action::ScrollDown => self.scroll_down(),
            crate::ui::Action::None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_app_builder_basic() {
        let app = AppBuilder::new(json!({"test": "data"})).build();

        assert_eq!(app.input(), "");
        assert!(!app.should_exit());
        assert_eq!(app.prompt(), "query > ");
    }

    #[test]
    fn test_app_builder_with_config() {
        let config = AppConfig::with_prompt("custom > ");
        let app = AppBuilder::new(json!({"test": "data"}))
            .with_config(config)
            .build();

        assert_eq!(app.prompt(), "custom > ");
    }

    #[test]
    fn test_app_builder_with_cache() {
        let app = AppBuilder::new(json!({"name": "test"}))
            .with_cache()
            .build();

        assert_eq!(app.input(), "");
        assert_eq!(app.data().get(), &json!({"name": "test"}));
    }
}
