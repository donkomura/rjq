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
use crate::ui::Theme;
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

struct QueryRun {
    source: String,
    query: CompiledQuery,
    outputs: PrettyDocument,
    matched: bool,
}

impl QueryRun {
    fn new(source: String, query: CompiledQuery, inputs: &[Val]) -> Self {
        let mut run = Self {
            matched: source == IDENTITY_QUERY,
            source,
            query,
            outputs: PrettyDocument::new(),
        };
        for input in inputs {
            run.push(input);
        }
        run
    }

    fn push(&mut self, input: &Val) {
        for output in self.query.run(input.clone()) {
            self.matched |= !matches!(output, Val::Null);
            self.outputs.push(output);
        }
    }
}

struct Candidate {
    evaluation: Result<QueryRun, AppError>,
    deadline: Option<Instant>,
}

impl Candidate {
    fn is_due(&self, now: Instant) -> bool {
        self.deadline.is_some_and(|deadline| now >= deadline)
    }
}

pub struct App {
    config: AppConfig,
    state: AppState,
    inputs: Vec<Val>,
    shown: QueryRun,
    candidate: Option<Candidate>,
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
        let identity = CompiledQuery::compile(IDENTITY_QUERY).expect("identity query compiles");
        Self {
            config,
            state: AppState::default(),
            inputs: Vec::new(),
            shown: QueryRun::new(IDENTITY_QUERY.to_string(), identity, &[]),
            candidate: None,
            load_state: LoadState::Loading,
        }
    }

    pub fn push_input(&mut self, value: serde_json::Value) {
        self.push_val(into_val(value));
    }

    fn push_val(&mut self, input: Val) {
        self.shown.push(&input);
        if let Some(Candidate {
            evaluation: Ok(run),
            ..
        }) = &mut self.candidate
        {
            run.push(&input);
            if run.matched {
                self.promote_candidate();
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
        self.shown.outputs.total_lines()
    }

    pub fn visible_lines(&self, height: usize) -> Vec<String> {
        self.shown.outputs.lines(self.state.scroll_offset, height)
    }

    pub fn status_line(&self) -> String {
        let count = self.inputs.len();
        let load = match &self.load_state {
            LoadState::Loading => format!("loading... {count} values"),
            LoadState::Finished => format!("{count} values"),
            LoadState::Failed(message) => format!("input error after {count} values: {message}"),
        };
        match self.candidate {
            Some(_) => format!("{load} | showing: {}", self.shown.source),
            None => load,
        }
    }

    pub fn next_deadline(&self) -> Option<Instant> {
        self.candidate.as_ref().and_then(|c| c.deadline)
    }

    pub fn tick(&mut self, now: Instant) {
        if self.candidate.as_ref().is_some_and(|c| c.is_due(now)) {
            self.promote_candidate();
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

    fn show(&mut self, run: QueryRun) {
        self.shown = run;
        self.state.reset_scroll();
    }

    fn display(&mut self, evaluation: Result<QueryRun, AppError>) {
        match evaluation {
            Ok(run) => self.show(run),
            Err(error) => self.state.last_error = Some(error),
        }
    }

    fn promote_candidate(&mut self) {
        if let Some(candidate) = self.candidate.take() {
            self.display(candidate.evaluation);
        }
    }

    fn rerun_query(&mut self) {
        let showing_last_match = self.shown.matched && self.state.last_error.is_none();
        self.state.last_error = None;
        self.candidate = None;
        let source = match self.state.input.as_str() {
            "" => IDENTITY_QUERY,
            input => input,
        };
        if source == self.shown.source {
            return;
        }
        let evaluation = CompiledQuery::compile(source)
            .map(|query| QueryRun::new(source.to_string(), query, &self.inputs));
        let matched = matches!(&evaluation, Ok(run) if run.matched);
        if matched || !showing_last_match {
            self.display(evaluation);
        } else {
            self.candidate = Some(Candidate {
                evaluation,
                deadline: Instant::now().checked_add(self.config.no_match_delay),
            });
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

    pub fn theme(&self) -> &Theme {
        &self.config.theme
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
    use std::time::Duration;

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
            (
                "no outputs keeps the last matching result",
                "empty",
                pretty_lines(&json!({"name": "test", "items": [1, 2]})),
            ),
            (
                "compile error keeps the last matching result",
                ".[",
                pretty_lines(&json!({"name": "test", "items": [1, 2]})),
            ),
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

    fn after_no_match_delay() -> Instant {
        Instant::now() + AppConfig::default().no_match_delay + Duration::from_millis(100)
    }

    #[test]
    fn test_unmatched_query_keeps_last_matching_preview() {
        let metadata = pretty_lines(&json!({"name": "rjq", "version": null}));
        let tests = vec![
            ("compile error", ".metadata."),
            ("all outputs are null", ".metadata.na"),
            ("runtime error", ".metadata[0]"),
        ];

        for (name, query) in tests {
            let mut app = App::new(json!({"metadata": {"name": "rjq", "version": null}}));
            type_query(&mut app, ".metadata");
            type_query(&mut app, &query[".metadata".len()..]);

            assert_eq!(metadata, app.visible_lines(10), "{name}");
            assert_eq!(metadata.len(), app.total_lines(), "{name}");
            assert_eq!("1 values | showing: .metadata", app.status_line(), "{name}");
        }
    }

    #[test]
    fn test_matching_query_is_shown_immediately() {
        let mut app = App::new(json!({"metadata": {"name": "rjq"}}));
        type_query(&mut app, ".metadata.name");

        assert_eq!(vec!["\"rjq\"".to_string()], app.visible_lines(10));
        assert_eq!("1 values", app.status_line());
        assert_eq!(None, app.next_deadline());
    }

    #[test]
    fn test_partially_null_outputs_are_a_match() {
        let mut app = App::new(json!([1, null]));
        type_query(&mut app, ".[]");

        assert_eq!(pretty_lines(&json!([1, null])), app.visible_lines(10));
    }

    #[test]
    fn test_unmatched_query_is_shown_after_no_match_delay() {
        let mut app = App::new(json!({"metadata": {"name": "rjq"}}));
        type_query(&mut app, ".metadata.na");
        assert!(app.next_deadline().is_some());

        app.tick(Instant::now());
        assert_eq!(pretty_lines(&json!({"name": "rjq"})), app.visible_lines(10));

        app.tick(after_no_match_delay());
        assert_eq!(vec!["null".to_string()], app.visible_lines(10));
        assert_eq!("1 values", app.status_line());
        assert_eq!(None, app.next_deadline());
    }

    #[test]
    fn test_compile_error_is_reported_after_no_match_delay() {
        let mut app = App::new(json!({"metadata": {"name": "rjq"}}));
        type_query(&mut app, ".metadata.");

        app.tick(after_no_match_delay());
        assert!(matches!(app.last_error(), Some(AppError::QueryCompile(_))));
        assert_eq!("1 values", app.status_line());

        app.push_char('n');
        assert!(app.last_error().is_none());
        assert_eq!(vec!["null".to_string()], app.visible_lines(10));
        assert_eq!("1 values", app.status_line());
        assert_eq!(None, app.next_deadline());
    }

    #[test]
    fn test_compile_error_stays_while_query_keeps_failing_to_compile() {
        let mut app = App::new(json!({"metadata": {"name": "rjq"}}));
        type_query(&mut app, ".\"foo");
        app.tick(after_no_match_delay());
        assert!(matches!(app.last_error(), Some(AppError::QueryCompile(_))));

        type_query(&mut app, " bar");

        assert!(matches!(app.last_error(), Some(AppError::QueryCompile(_))));
        assert_eq!("1 values", app.status_line());
        assert_eq!(None, app.next_deadline());
    }

    #[test]
    fn test_unmatched_result_shown_after_delay_is_not_kept_as_last_match() {
        let mut app = App::new(json!({"metadata": {"name": "rjq"}}));
        type_query(&mut app, ".metadata.na");
        app.tick(after_no_match_delay());

        app.push_char('m');

        assert_eq!(vec!["null".to_string()], app.visible_lines(10));
        assert_eq!("1 values", app.status_line());
        assert_eq!(None, app.next_deadline());

        app.push_char('e');
        assert_eq!(vec!["\"rjq\"".to_string()], app.visible_lines(10));
    }

    #[test]
    fn test_no_match_delay_too_long_for_instant_never_shows_unmatched_result() {
        let config = AppConfig {
            no_match_delay: Duration::MAX,
            ..AppConfig::default()
        };
        let mut app = App::with_config(json!({"a": 1}), config);

        type_query(&mut app, ".b");
        app.tick(Instant::now() + Duration::from_secs(60 * 60 * 24 * 365));

        assert_eq!(pretty_lines(&json!({"a": 1})), app.visible_lines(10));
        assert_eq!("1 values | showing: .", app.status_line());
        assert_eq!(None, app.next_deadline());
    }

    #[test]
    fn test_no_match_delay_is_configurable() {
        let config = AppConfig {
            no_match_delay: Duration::from_secs(5),
            ..AppConfig::default()
        };
        let mut app = App::with_config(json!({"a": 1}), config);
        type_query(&mut app, ".b");

        app.tick(after_no_match_delay());
        assert_eq!(pretty_lines(&json!({"a": 1})), app.visible_lines(10));

        app.tick(Instant::now() + Duration::from_secs(6));
        assert_eq!(vec!["null".to_string()], app.visible_lines(10));
    }

    #[test]
    fn test_editing_back_to_shown_query_cancels_pending_result() {
        let mut app = App::new(json!({"a": 1}));
        type_query(&mut app, ".a.");
        app.pop_char();

        assert_eq!(vec!["1".to_string()], app.visible_lines(10));
        assert_eq!("1 values", app.status_line());
        assert_eq!(None, app.next_deadline());
    }

    #[test]
    fn test_empty_query_always_shows_identity() {
        let mut app = App::new(json!(null));
        type_query(&mut app, ".a");
        app.clear_input();

        assert_eq!(vec!["null".to_string()], app.visible_lines(10));
        assert_eq!("1 values", app.status_line());
    }

    #[test]
    fn test_streaming_inputs_while_query_is_unmatched() {
        let mut app = App::loading(AppConfig::default());
        app.push_input(json!({"id": 1, "tag": null}));
        type_query(&mut app, ".id");
        type_query(&mut app, ".");

        app.push_input(json!({"id": 2, "tag": null}));
        assert_eq!(pretty_lines(&json!([1, 2])), app.visible_lines(10));

        app.clear_input();
        type_query(&mut app, ".tag");
        assert_eq!("loading... 2 values | showing: .", app.status_line());

        app.push_input(json!({"id": 3, "tag": "new"}));
        assert_eq!(
            pretty_lines(&json!([null, null, "new"])),
            app.visible_lines(10)
        );
        assert_eq!("loading... 3 values", app.status_line());
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
