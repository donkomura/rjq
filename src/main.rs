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

#[derive(Default, Debug)]
struct App {
    pub prompt: String,
    pub input: String,
    pub exit: bool,
    pub json_content: String,
}

impl App {
    fn new() -> Result<Self> {
        let json_content = Self::read_stdin()?;
        Ok(App {
            prompt: "query > ".to_string(),
            input: String::new(),
            exit: false,
            json_content,
        })
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

    pub fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        while !self.exit {
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
        frame.set_cursor_position(((self.prompt.len() + self.input.len()) as u16, 0));
    }

    fn handle_events(&mut self, key_event: KeyEvent) -> Result<()> {
        let action = get_action(key_event);
        update(self, action);
        Ok(())
    }
}

// イベントからアクションへのマッピング
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

// アクションに基づく状態更新
fn update(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.exit = true,
        Action::Input(c) => app.input.push(c),
        Action::Backspace => {
            if !app.input.is_empty() {
                app.input.pop();
            }

        }
        Action::Clear => app.input.clear(),
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

        let json_paragraph = Paragraph::new(self.json_content.as_str());
        json_paragraph.render(chunks[1], buf);
    }
}

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut stderr = std::io::stderr();
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;
    let result = app.run(&mut terminal);

    // restore terminal
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    if let Ok(()) = result {
        Ok(())
    } else {
        Err(result.unwrap_err())
    }
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
            json_content: "{}".to_string(),
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
            json_content: r#"{"name": "test"}"#.to_string(),
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
            json_content: "{}".to_string(),
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
