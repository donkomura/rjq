pub mod builder;
pub mod config;
pub mod error;
pub mod state;

pub use builder::{AppBuilder, EnhancedApp};
pub use config::AppConfig;
pub use error::AppError;
pub use state::AppState;

use crate::input::InputEvent;
use crate::query::{CompiledQuery, into_val};
use crate::view::PrettyDocument;
use jaq_json::Val;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Instant;

/// コンテンツ生成のための共通トレイト
pub trait ContentGenerator {
    /// 現在のコンテンツを生成する
    fn generate_current_content(&self) -> String;

    /// スクロール可能なコンテンツの行数を取得する
    fn get_total_lines(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadState {
    Loading,
    Finished,
    Failed(String),
}

const IDENTITY_QUERY: &str = ".";

pub struct App {
    config: AppConfig,
    state: AppState,
    inputs: Vec<Val>,
    query: Option<CompiledQuery>,
    outputs: PrettyDocument,
    load_state: LoadState,
}

impl App {
    pub fn new(json_value: serde_json::Value) -> Self {
        Self::with_config(json_value, AppConfig::default())
    }

    pub fn with_config(json_value: serde_json::Value, config: AppConfig) -> Self {
        let mut app = Self::loading(config);
        app.push_input(json_value);
        app.finish_loading();
        app
    }

    pub fn loading(config: AppConfig) -> Self {
        let mut app = Self {
            config,
            state: AppState::default(),
            inputs: Vec::new(),
            query: None,
            outputs: PrettyDocument::new(),
            load_state: LoadState::Loading,
        };
        app.rerun_query();
        app
    }

    pub fn push_input(&mut self, value: serde_json::Value) {
        self.push_val(into_val(value));
    }

    fn push_val(&mut self, input: Val) {
        if let Some(query) = &self.query {
            for output in query.run(input.clone()) {
                self.outputs.push(output);
            }
        }
        self.inputs.push(input);
    }

    pub fn finish_loading(&mut self) {
        self.load_state = LoadState::Finished;
    }

    pub fn fail_loading(&mut self, message: String) {
        self.load_state = LoadState::Failed(message);
    }

    pub fn load_state(&self) -> &LoadState {
        &self.load_state
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn total_lines(&self) -> usize {
        match self.query {
            Some(_) => self.outputs.total_lines(),
            None => 0,
        }
    }

    pub fn visible_lines(&self, height: usize) -> Vec<String> {
        match self.query {
            Some(_) => self.outputs.lines(self.state.scroll_offset, height),
            None => Vec::new(),
        }
    }

    pub fn status_line(&self) -> String {
        let count = self.inputs.len();
        match &self.load_state {
            LoadState::Loading => format!("loading... {count} values"),
            LoadState::Finished => format!("{count} values"),
            LoadState::Failed(message) => format!("input error after {count} values: {message}"),
        }
    }

    /// Consumes input events until the channel is empty or `deadline` passes,
    /// so a stream of many small values cannot starve key handling and drawing.
    pub fn receive_inputs(&mut self, receiver: &Receiver<InputEvent>, deadline: Instant) {
        while self.load_state == LoadState::Loading && Instant::now() < deadline {
            match receiver.try_recv() {
                Ok(InputEvent::Value(value)) => self.push_val(value.into_inner()),
                Ok(InputEvent::Finished) => self.finish_loading(),
                Ok(InputEvent::Failed(message)) => self.fail_loading(message),
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => {
                    self.fail_loading("input reader stopped before reaching the end".to_string())
                }
            }
        }
    }

    fn rerun_query(&mut self) {
        self.outputs.clear();
        let source = match self.state.input.as_str() {
            "" => IDENTITY_QUERY,
            input => input,
        };
        self.query = CompiledQuery::compile(source).ok();
        if let Some(query) = &self.query {
            for input in &self.inputs {
                for output in query.run(input.clone()) {
                    self.outputs.push(output);
                }
            }
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

    pub fn scroll_offset(&self) -> usize {
        self.state.scroll_offset
    }

    // 状態変更（AppStateに委譲）
    pub fn set_exit(&mut self, exit: bool) {
        self.state.set_exit(exit);
    }

    pub fn clear_input(&mut self) {
        self.state.clear_input();
        self.rerun_query();
    }

    pub fn push_char(&mut self, c: char) {
        self.state.push_char(c);
        self.rerun_query();
    }

    pub fn pop_char(&mut self) {
        self.state.pop_char();
        self.rerun_query();
    }

    pub fn scroll_up(&mut self) {
        self.state.scroll_up();
    }

    pub fn scroll_down(&mut self) {
        let total_lines = self.total_lines();
        let visible_height = self.config.visible_height;
        self.state.scroll_down_bounded(total_lines, visible_height);
    }

    pub fn reset_scroll(&mut self) {
        self.state.reset_scroll();
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
        self.rerun_query();
    }

    pub fn record_query(&mut self, query: String) {
        self.state.query_history.record_query(query);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn type_query(app: &mut App, query: &str) {
        for c in query.chars() {
            app.push_char(c);
        }
    }

    fn pretty_lines(value: &Value) -> Vec<String> {
        serde_json::to_string_pretty(value)
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn test_empty_query_shows_input_as_identity() {
        let input = json!({"name": "test", "items": [1, 2]});
        let app = App::new(input.clone());

        assert_eq!(pretty_lines(&input), app.visible_lines(100));
        assert_eq!(pretty_lines(&input).len(), app.total_lines());
    }

    #[test]
    fn test_query_result_follows_input_changes() {
        let tests = vec![
            ("field", ".name", vec!["\"test\"".to_string()]),
            (
                "multiple outputs as array",
                ".items[]",
                pretty_lines(&json!([1, 2])),
            ),
            ("no outputs", "empty", vec!["null".to_string()]),
            ("compile error shows nothing", ".[", vec![]),
        ];

        for (name, query, expected) in tests {
            let mut app = App::new(json!({"name": "test", "items": [1, 2]}));
            type_query(&mut app, query);

            assert_eq!(expected, app.visible_lines(100), "{name}");
        }
    }

    #[test]
    fn test_query_is_recomputed_after_editing() {
        let mut app = App::new(json!({"a": 1, "ab": 2}));
        type_query(&mut app, ".ab");
        assert_eq!(vec!["2".to_string()], app.visible_lines(10));

        app.pop_char();
        assert_eq!(vec!["1".to_string()], app.visible_lines(10));

        app.clear_input();
        assert_eq!(
            pretty_lines(&json!({"a": 1, "ab": 2})),
            app.visible_lines(10)
        );

        app.apply_suggestion(".ab".to_string());
        assert_eq!(vec!["2".to_string()], app.visible_lines(10));
    }

    #[test]
    fn test_streaming_inputs_apply_current_query_incrementally() {
        let mut app = App::loading(AppConfig::default());
        assert_eq!(&LoadState::Loading, app.load_state());
        assert_eq!(vec!["null".to_string()], app.visible_lines(10));

        type_query(&mut app, ".id");
        app.push_input(json!({"id": 1}));
        assert_eq!(vec!["1".to_string()], app.visible_lines(10));

        app.push_input(json!({"id": 2}));
        app.finish_loading();
        assert_eq!(2, app.input_count());
        assert_eq!(&LoadState::Finished, app.load_state());
        assert_eq!(pretty_lines(&json!([1, 2])), app.visible_lines(10));

        app.clear_input();
        assert_eq!(
            pretty_lines(&json!([{"id": 1}, {"id": 2}])),
            app.visible_lines(10)
        );
    }

    #[test]
    fn test_fail_loading_keeps_loaded_inputs() {
        let mut app = App::loading(AppConfig::default());
        app.push_input(json!(1));
        app.fail_loading("broken".to_string());

        assert_eq!(&LoadState::Failed("broken".to_string()), app.load_state());
        assert_eq!(vec!["1".to_string()], app.visible_lines(10));
    }

    #[test]
    fn test_status_line() {
        let mut loading = App::loading(AppConfig::default());
        loading.push_input(json!(1));
        let mut finished = App::loading(AppConfig::default());
        finished.push_input(json!(1));
        finished.push_input(json!(2));
        finished.finish_loading();
        let mut failed = App::loading(AppConfig::default());
        failed.fail_loading("EOF while parsing".to_string());

        let tests = vec![
            ("loading", loading, "loading... 1 values"),
            ("finished", finished, "2 values"),
            (
                "failed",
                failed,
                "input error after 0 values: EOF while parsing",
            ),
        ];

        for (name, app, expected) in tests {
            assert_eq!(expected, app.status_line(), "{name}");
        }
    }

    #[test]
    fn test_receive_inputs_until_finished() {
        let receiver = crate::input::spawn_reader(std::io::Cursor::new("{\"id\": 1}\n{\"id\": 2}"));
        let mut app = App::loading(AppConfig::default());
        type_query(&mut app, ".id");

        let give_up = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while app.load_state() == &LoadState::Loading && std::time::Instant::now() < give_up {
            app.receive_inputs(
                &receiver,
                std::time::Instant::now() + std::time::Duration::from_millis(10),
            );
        }

        assert_eq!(&LoadState::Finished, app.load_state());
        assert_eq!(pretty_lines(&json!([1, 2])), app.visible_lines(10));
    }

    #[test]
    fn test_receive_inputs_reports_reader_that_stopped_without_finishing() {
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        // SAFETY: the converted value is freshly built and not shared.
        let value = unsafe { crate::input::OwnedVal::new(into_val(json!(1))) };
        sender.send(crate::input::InputEvent::Value(value)).unwrap();
        drop(sender);
        let mut app = App::loading(AppConfig::default());

        app.receive_inputs(
            &receiver,
            std::time::Instant::now() + std::time::Duration::from_millis(10),
        );

        assert_eq!(1, app.input_count());
        assert!(matches!(app.load_state(), LoadState::Failed(_)));
    }

    #[test]
    fn test_visible_lines_follow_scroll_offset() {
        let mut app = App::with_config(json!([1, 2, 3, 4]), AppConfig::with_visible_height(2));

        app.scroll_down();
        app.scroll_down();

        assert_eq!(2, app.scroll_offset());
        assert_eq!(
            vec!["  2,".to_string(), "  3,".to_string()],
            app.visible_lines(2)
        );
    }
}
