use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{Paragraph, Widget},
};
use jaq_json::Val;
use jaq_core::{load::{Arena, File, Loader}, Ctx, Filter, Native, RcIter};
use serde_json::json;
use std::io::{self, Read, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    Input(char),
    Backspace,
    Clear,
    None,
}

#[derive(Debug)]
struct App {
    pub prompt: String,
    pub input: String,
    pub exit: bool,
    pub json_value: serde_json::Value,
    pub filtered_result: String,
    pub error: Option<io::Error>,
}

impl App {
    fn new(json_value: serde_json::Value) -> Self {
        let filtered_result = serde_json::to_string_pretty(&json_value)
            .unwrap_or_else(|_| "Error formatting JSON".to_string());
        
        App {
            prompt: "query > ".to_string(),
            input: String::new(),
            exit: false,
            json_value,
            filtered_result,
            error: None,
        }
    }

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            if let Event::Key(key_event) = event::read()? {
                self.handle_events(key_event)?;
            }
            let inputs = RcIter::new(core::iter::empty());
            let filter = self.filter()?;
            let results = filter.run((Ctx::new([], &inputs), Val::from(self.json_value.clone())));
            let values: Vec<serde_json::Value> = results.into_iter()
                .filter_map(|r| r.ok())
                .map(|val| val.into())
                .collect();
            
            self.filtered_result = if values.is_empty() {
                "null".to_string()
            } else if values.len() == 1 {
                serde_json::to_string_pretty(&values[0]).unwrap_or_else(|_| "Error formatting result".to_string())
            } else {
                serde_json::to_string_pretty(&values).unwrap_or_else(|_| "Error formatting result".to_string())
            };
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());

        // カーソル位置は別途設定
        frame.set_cursor_position(((self.prompt.len() + self.input.len()) as u16, 0));
    }

    fn handle_events(&mut self, key_event: KeyEvent) -> Result<()> {
        let action = get_action(key_event);
        update(self, action);
        Ok(())
    }

    fn filter(&mut self) -> Result<Filter<Native<Val>>> {
        let program = File {
            code: self.input.as_str(),
            path: ()
        };
        let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = Arena::default();
        let modules = loader.load(&arena, program).map_err(|e| {
            eprintln!("{:?}", e);
            std::io::Error::other(format!("Loader: {:?}", e))
        })?;
        let compiler = jaq_core::Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules);
        compiler.map_err(|e| {
            eprintln!("{:?}", e);
            std::io::Error::other(format!("Compiler setup: {:?}", e))
        })
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
        Action::Quit => app.exit = true,
        Action::Input(c) => app.input.push(c),
        Action::Backspace => {
            if !app.input.is_empty() {
                app.input.pop();
            }
        }
        Action::Clear => {
            app.input.clear();
            app.error = None;
        }
        Action::None => {}
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area);

        let prompt_text = format!("{}{}", self.prompt, self.input);
        let prompt_paragraph = Paragraph::new(prompt_text);
        prompt_paragraph.render(chunks[0], buf);

        if let Some(ref error) = self.error {
            let error_text = format!("Error: {}", error);
            let error_paragraph = Paragraph::new(error_text);
            error_paragraph.render(chunks[1], buf);
        } else {
            let json_paragraph = Paragraph::new(self.filtered_result.as_str());
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
        serde_json::from_str(&input_string)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("Invalid JSON: {}", e)))?
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

    #[test]
    fn test_basic_input() {
        let mut app = App {
            prompt: "query > ".to_string(),
            input: String::new(),
            exit: false,
            json_value: serde_json::json!({}),
            filtered_result: "{}".to_string(),
            error: None,
        };

        // 基本的な入力テスト
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        app.handle_events(key_event).unwrap();
        assert_eq!(app.input, "a");
    }

    #[test]
    fn test_render() {
        // 基本描画テスト（プロンプト + JSON）
        let app = App {
            prompt: "query > ".to_string(),
            input: String::new(),
            exit: false,
            json_value: serde_json::json!({"name": "test"}),
            filtered_result: r#"{"name": "test"}"#.to_string(),
            error: None,
        };
        let mut buf = Buffer::empty(Rect::new(0, 0, 30, 3));
        app.render(buf.area, &mut buf);

        let prompt_line = buf.content[0..30]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(prompt_line.contains("query >"));

        let json_line = buf.content[30..60]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(json_line.contains("name"));
        assert!(json_line.contains("test"));

        // 入力付き描画テスト
        let mut app_with_input = App {
            prompt: "query > ".to_string(),
            input: String::new(),
            exit: false,
            json_value: serde_json::json!({}),
            filtered_result: "{}".to_string(),
            error: None,
        };
        update(&mut app_with_input, Action::Input('h'));
        update(&mut app_with_input, Action::Input('i'));

        let mut input_buf = Buffer::empty(Rect::new(0, 0, 30, 2));
        app_with_input.render(input_buf.area, &mut input_buf);

        let input_prompt_line = input_buf.content[0..30]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(input_prompt_line.contains("query > hi"));
    }
}
