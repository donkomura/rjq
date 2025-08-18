use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::Paragraph,
};
use std::{
    io::{self, Read, Result},
};

#[derive(Default, Debug)]
struct App {
    prompt: String,
    input: String,
    exit: bool,
    json_content: String,
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
        let size = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(size);

        let prompt_text = format!("{}{}", self.prompt, self.input);
        let prompt_paragraph = Paragraph::new(prompt_text.clone());
        frame.render_widget(prompt_paragraph, chunks[0]);

        let json_paragraph = Paragraph::new(self.json_content.as_str());
        frame.render_widget(json_paragraph, chunks[1]);

        frame.set_cursor_position((
            (self.prompt.len() + self.input.len()) as u16,
            0,
        ));
    }

    fn handle_events(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Esc => self.exit = true,
            KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.exit = true
            }
            KeyCode::Char(c) => {
                if c == '\n' {
                    self.input.clear();
                } else {
                    self.input.push(c);
                }
            }
            KeyCode::Backspace => {
                if !self.input.is_empty() {
                    self.input.pop();
                }
            }
            KeyCode::Enter => {
                self.input.clear();
            }
            _ => {}
        }
        Ok(())
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
