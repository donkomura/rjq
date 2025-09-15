use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use jaq_core::{
    Ctx, RcIter,
    load::{Arena, File, Loader},
};
use jaq_json::Val;
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Paragraph, Widget},
};
use serde_json::json;
use std::io::{self, Read};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("Query compilation error: {0}")]
    QueryCompile(String),
    #[error("Query execution error: {0}")]
    QueryExecution(String),
    #[error("Terminal error: {0}")]
    Terminal(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    Input(char),
    Backspace,
    Clear,
    None,
}

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
pub enum QueryResult {
    Single(serde_json::Value),
    Multiple(Vec<serde_json::Value>),
    Empty,
}

impl QueryResult {
    pub fn format_pretty(&self) -> String {
        match self {
            QueryResult::Single(val) => {
                serde_json::to_string_pretty(val)
                    .unwrap_or_else(|_| "Error formatting result".to_string())
            }
            QueryResult::Multiple(vals) => {
                serde_json::to_string_pretty(vals)
                    .unwrap_or_else(|_| "Error formatting result".to_string())
            }
            QueryResult::Empty => "null".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct JsonData {
    inner: serde_json::Value,
}

impl JsonData {
    pub fn new(value: serde_json::Value) -> Self {
        Self { inner: value }
    }

    pub fn get(&self) -> &serde_json::Value {
        &self.inner
    }

    pub fn execute_query(&self, query: &str) -> Result<QueryResult> {
        if query.is_empty() {
            return Err(AppError::QueryCompile("Empty query".to_string()));
        }

        let program = File {
            code: query,
            path: (),
        };
        let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = Arena::default();
        let modules = loader.load(&arena, program).map_err(|e| {
            AppError::QueryCompile(format!("Loader: {:?}", e))
        })?;
        let filter = jaq_core::Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules)
            .map_err(|e| AppError::QueryCompile(format!("Compiler: {:?}", e)))?;

        let inputs = RcIter::new(core::iter::empty());
        let results = filter.run((Ctx::new([], &inputs), Val::from(self.inner.clone())));
        let values: Vec<serde_json::Value> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|val| val.into())
            .collect();

        Ok(match values.len() {
            0 => QueryResult::Empty,
            1 => QueryResult::Single(values.into_iter().next().unwrap()),
            _ => QueryResult::Multiple(values),
        })
    }
}

#[derive(Debug)]
struct App {
    config: AppConfig,
    state: AppState,
    data: JsonData,
}

impl App {
    fn new(json_value: serde_json::Value) -> Self {
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
    pub fn execute_current_query(&self) -> Result<QueryResult> {
        self.data.execute_query(&self.state.input)
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        while !self.should_exit() {
            terminal.draw(|frame| self.draw(frame))?;
            if let Event::Key(key_event) = event::read()? {
                self.handle_events(key_event)?;
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());

        // カーソル位置は別途設定
        frame.set_cursor_position(((self.prompt().len() + self.input().len()) as u16, 0));
    }

    fn handle_events(&mut self, key_event: KeyEvent) -> Result<()> {
        let action = get_action(key_event);
        update(self, action);
        Ok(())
    }
}

fn get_action(key_event: KeyEvent) -> Action {
    match key_event.code {
        KeyCode::Esc => Action::Quit,
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char(c) => {
            if c == '\n' {
                Action::Clear
            } else {
                Action::Input(c)
            }
        }
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Enter => Action::Clear,
        _ => Action::None,
    }
}

fn update(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.set_exit(true),
        Action::Input(c) => app.push_char(c),
        Action::Backspace => {
            if !app.input().is_empty() {
                app.pop_char();
            }
        }
        Action::Clear => app.clear_input(),
        Action::None => {}
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let prompt_text = format!("{}{}", self.prompt(), self.input());
        let prompt_paragraph = Paragraph::new(prompt_text);
        prompt_paragraph.render(chunks[0], buf);

        if let Some(error) = self.last_error() {
            let error_text = format!("Error: {}", error);
            let error_paragraph = Paragraph::new(error_text);
            error_paragraph.render(chunks[1], buf);
        } else {
            // クエリ結果を動的に計算して表示
            let result_text = match self.execute_current_query() {
                Ok(result) => result.format_pretty(),
                Err(_) => {
                    // 初期状態の場合は元のJSONを表示
                    if self.input().is_empty() {
                        serde_json::to_string_pretty(self.data.get())
                            .unwrap_or_else(|_| "Error formatting JSON".to_string())
                    } else {
                        "".to_string()
                    }
                }
            };
            let json_paragraph = Paragraph::new(result_text);
            json_paragraph.render(chunks[1], buf);
        }
    }
}

fn read_stdin() -> Result<String> {
    if atty::is(atty::Stream::Stdin) {
        Ok(String::new())
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    }
}

fn restore_terminal<B: Backend + std::io::Write>(terminal: &mut Terminal<B>) -> Result<()> {
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    disable_raw_mode()?;
    terminal.show_cursor()?;
    Ok(())
}

fn main() -> Result<()> {
    let input_string = read_stdin()?;
    let json_value: serde_json::Value = if input_string.trim().is_empty() {
        json!({"example": "data", "number": 42, "array": [1, 2, 3]})
    } else {
        serde_json::from_str(&input_string)?
    };

    enable_raw_mode()?;
    let mut stderr = std::io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(json_value);
    let res = app.run(&mut terminal);

    restore_terminal(&mut terminal)?;

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use serde_json::json;

    // テスト用のヘルパー関数
    fn create_test_app(json_value: serde_json::Value) -> App {
        App::new(json_value)
    }

    fn execute_query(app: &mut App, query: &str) -> String {
        app.state.input = query.to_string();
        match app.execute_current_query() {
            Ok(result) => result.format_pretty(),
            Err(_) => "Error".to_string(),
        }
    }

    #[test]
    fn test_basic_input() {
        let mut app = create_test_app(json!({}));

        // 基本的な入力テスト
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        app.handle_events(key_event).unwrap();
        assert_eq!(app.input(), "a");
    }

    #[test]
    fn test_render() {
        // 基本描画テスト（プロンプト確認）
        let app = create_test_app(json!({"name": "test"}));
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 3));
        app.render(buf.area, &mut buf);

        let prompt_line = buf.content[0..50]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(prompt_line.contains("query >"));

        // 入力付き描画テスト
        let mut app_with_input = create_test_app(json!({}));
        update(&mut app_with_input, Action::Input('.'));

        let mut input_buf = Buffer::empty(Rect::new(0, 0, 30, 2));
        app_with_input.render(input_buf.area, &mut input_buf);

        let input_prompt_line = input_buf.content[0..30]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(input_prompt_line.contains("query > ."));
    }

    // === ロジック部分のテスト ===

    #[test]
    fn test_identity_query() {
        let mut app = create_test_app(json!({"name": "test", "value": 42}));
        let result = execute_query(&mut app, ".");

        // パースして比較
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!({"name": "test", "value": 42}));
    }

    #[test]
    fn test_field_access_query() {
        let mut app = create_test_app(json!({"name": "test", "value": 42}));
        let result = execute_query(&mut app, ".name");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!("test"));
    }

    #[test]
    fn test_array_access_query() {
        let mut app = create_test_app(json!({"items": [1, 2, 3, 4, 5]}));
        let result = execute_query(&mut app, ".items[2]");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!(3));
    }

    #[test]
    fn test_array_slice_query() {
        let mut app = create_test_app(json!({"items": [1, 2, 3, 4, 5]}));
        let result = execute_query(&mut app, ".items[1:3]");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!([2, 3]));
    }

    #[test]
    fn test_map_operation() {
        let mut app = create_test_app(json!([1, 2, 3]));
        let result = execute_query(&mut app, "map(. * 2)");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!([2, 4, 6]));
    }

    #[test]
    fn test_filter_operation() {
        let mut app = create_test_app(json!([1, 2, 3, 4, 5]));
        let result = execute_query(&mut app, "map(select(. > 3))");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!([4, 5]));
    }

    #[test]
    fn test_complex_nested_query() {
        let mut app = create_test_app(json!({
            "users": [
                {"name": "Alice", "age": 30, "active": true},
                {"name": "Bob", "age": 25, "active": false},
                {"name": "Charlie", "age": 35, "active": true}
            ]
        }));
        let result = execute_query(&mut app, ".users | map(select(.active)) | map(.name)");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!(["Alice", "Charlie"]));
    }

    #[test]
    fn test_keys_operation() {
        let mut app = create_test_app(json!({"a": 1, "b": 2, "c": 3}));
        let result = execute_query(&mut app, "keys");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!(["a", "b", "c"]));
    }

    #[test]
    fn test_length_operation() {
        let mut app = create_test_app(json!([1, 2, 3, 4, 5]));
        let result = execute_query(&mut app, "length");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!(5));
    }

    #[test]
    fn test_empty_query() {
        let mut app = create_test_app(json!({"test": "data"}));

        // 空のクエリはコンパイルエラーになる
        app.state.input = "".to_string();
        let result = app.execute_current_query();
        assert!(result.is_err());
    }

    #[test]
    fn test_null_result() {
        let mut app = create_test_app(json!({"test": "data"}));
        let result = execute_query(&mut app, ".nonexistent");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, serde_json::Value::Null);
    }

    #[test]
    fn test_mathematical_operations() {
        let mut app = create_test_app(json!({"a": 10, "b": 5}));
        let result = execute_query(&mut app, ".a + .b");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!(15));
    }

    #[test]
    fn test_string_operations() {
        let mut app = create_test_app(json!({"first": "Hello", "second": "World"}));
        let result = execute_query(&mut app, ".first + \" \" + .second");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed, json!("Hello World"));
    }

    #[test]
    fn test_type_function() {
        let mut app = create_test_app(
            json!({"string": "text", "number": 42, "bool": true, "null": null, "array": [], "object": {}}),
        );
        let result = execute_query(&mut app, "map_values(type)");

        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        let expected = json!({
            "string": "string",
            "number": "number",
            "bool": "boolean",
            "null": "null",
            "array": "array",
            "object": "object"
        });
        assert_eq!(parsed, expected);
    }

    // === Actionのテスト ===

    #[test]
    fn test_action_processing() {
        let mut app = create_test_app(json!({}));

        // 文字入力のテスト
        update(&mut app, Action::Input('.'));
        update(&mut app, Action::Input('t'));
        update(&mut app, Action::Input('e'));
        update(&mut app, Action::Input('s'));
        update(&mut app, Action::Input('t'));
        assert_eq!(app.input(), ".test");

        // バックスペースのテスト
        update(&mut app, Action::Backspace);
        assert_eq!(app.input(), ".tes");

        // クリアのテスト
        update(&mut app, Action::Clear);
        assert_eq!(app.input(), "");

        // 終了フラグのテスト
        assert!(!app.should_exit());
        update(&mut app, Action::Quit);
        assert!(app.should_exit());
    }

    #[test]
    fn test_key_event_mapping() {
        // Escキー
        let action = get_action(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert_eq!(action, Action::Quit);

        // Ctrl+C
        let action = get_action(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert_eq!(action, Action::Quit);

        // Enter
        let action = get_action(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(action, Action::Clear);

        // Backspace
        let action = get_action(KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
        assert_eq!(action, Action::Backspace);

        // 文字キー
        let action = get_action(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
        assert_eq!(action, Action::Input('a'));

        // その他のキー
        let action = get_action(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(action, Action::None);
    }

    #[test]
    fn test_app_creation() {
        let json_data = json!({"test": "value"});
        let app = App::new(json_data.clone());

        assert_eq!(app.prompt(), "query > ");
        assert_eq!(app.input(), "");
        assert!(!app.should_exit());
        assert_eq!(app.data.get(), &json_data);
        assert!(app.last_error().is_none());
    }

    // === エラーハンドリングのテスト ===

    #[test]
    fn test_invalid_jaq_query() {
        let mut app = create_test_app(json!({"test": "data"}));

        // 無効なjaqクエリでexecute_current_query()を呼び出し、エラーが返されることを確認
        app.state.input = "invalid_syntax[".to_string();
        let result = app.execute_current_query();
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_jaq_query() {
        let mut app = create_test_app(json!({"test": "data"}));

        // 文法的に正しくないクエリ
        app.state.input = "..[".to_string();
        let result = app.execute_current_query();
        assert!(result.is_err());
    }

    #[test]
    fn test_undefined_function_query() {
        let mut app = create_test_app(json!({"test": "data"}));

        // 存在しない関数を使用したクエリ
        app.state.input = "undefined_function()".to_string();
        let result = app.execute_current_query();
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_invalid_query() {
        let mut app = create_test_app(json!({"items": [1, 2, 3]}));

        // 複雑だが無効なクエリ
        app.state.input = ".items | map(select(. > ) | .nonexistent".to_string();
        let result = app.execute_current_query();
        assert!(result.is_err());
    }

    #[test]
    fn test_partial_query_handling() {
        let mut app = create_test_app(json!({"test": "data"}));

        // 部分的に完成していないクエリ（通常の入力中の状態をシミュレート）
        app.state.input = ".test |".to_string();
        let result = app.execute_current_query();
        assert!(result.is_err());
    }

    #[test]
    fn test_backspace_on_empty_input() {
        let mut app = create_test_app(json!({}));

        // 空の入力でBackspaceアクションを実行
        assert_eq!(app.input(), "");
        update(&mut app, Action::Backspace);
        assert_eq!(app.input(), ""); // 変化なし
    }

    #[test]
    fn test_clear_resets_error() {
        let mut app = create_test_app(json!({}));

        // エラー状態を設定
        app.state.last_error = Some(AppError::QueryCompile("test error".to_string()));
        assert!(app.last_error().is_some());

        // Clearアクションでエラーがリセットされることを確認
        update(&mut app, Action::Clear);
        assert!(app.last_error().is_none());
        assert_eq!(app.input(), "");
    }

    #[test]
    fn test_error_rendering() {
        let mut app = create_test_app(json!({}));

        // エラー状態を設定
        app.state.last_error = Some(AppError::QueryCompile("Test error message".to_string()));

        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 3));
        app.render(buf.area, &mut buf);

        // エラーメッセージが表示されていることを確認
        let error_line = buf.content[50..100] // 2行目（エラー表示エリア）
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(error_line.contains("Error:"));
        assert!(error_line.contains("Test error message"));
    }

    #[test]
    fn test_complex_json_with_invalid_query() {
        let complex_json = json!({
            "users": [
                {"name": "Alice", "scores": [95, 87, 92]},
                {"name": "Bob", "scores": [78, 85, 90]},
                {"name": "Charlie", "scores": [88, 91, 94]}
            ],
            "metadata": {
                "total": 3,
                "average_score": 88.5
            }
        });

        let mut app = create_test_app(complex_json);

        // 複雑なJSONに対する無効なクエリ
        app.state.input = ".users[].scores | map(select(. > )) | average".to_string();
        let result = app.execute_current_query();
        assert!(result.is_err());
    }
}
