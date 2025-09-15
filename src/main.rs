use crossterm::{
    event::EnableMouseCapture,
    execute,
    terminal::enable_raw_mode,
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
};
use serde_json::json;
use std::io::{self, Read};

use rjq::{App, Result};

fn read_stdin() -> std::result::Result<String, std::io::Error> {
    if atty::is(atty::Stream::Stdin) {
        Ok(String::new())
    } else {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        Ok(buffer)
    }
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
    execute!(stderr, crossterm::terminal::EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(json_value);
    let res = app.run(&mut terminal);

    rjq::restore_terminal(&mut terminal).ok();

    if let Err(e) = res {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use serde_json::json;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::widgets::Widget;
    use rjq::App;

    fn create_test_app(json_value: serde_json::Value) -> App {
        App::new(json_value)
    }

    #[test]
    fn test_basic_input() {
        let mut app = create_test_app(json!({}));
        let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        app.handle_events(key_event).unwrap();
        assert_eq!(app.input(), "a");
    }

    #[test]
    fn test_render() {
        let app = create_test_app(json!({"name": "test"}));
        let mut buf = Buffer::empty(Rect::new(0, 0, 50, 3));
        app.render(buf.area, &mut buf);

        let prompt_line = buf.content[0..50]
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(prompt_line.contains("query >"));
    }
}