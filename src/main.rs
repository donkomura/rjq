use crossterm::{event::EnableMouseCapture, execute, terminal::enable_raw_mode};
use ratatui::{Terminal, backend::CrosstermBackend};
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
    execute!(
        stderr,
        crossterm::terminal::EnterAlternateScreen,
        EnableMouseCapture
    )?;
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
    use rjq::App;
    use serde_json::json;

    #[test]
    fn test_app_creation() {
        let app = App::new(json!({"test": "data"}));
        assert_eq!(app.input(), "");
        assert!(!app.should_exit());
    }
}
