use clap::Parser;
use crossterm::{event::EnableMouseCapture, execute, terminal::enable_raw_mode};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Read};
use std::fs;

use rjq::{App, Result};

/// A command-line jq processor with interactive TUI
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(name = "rjq")]
#[command(about = "Interactive jq query processor for JSON data")]
#[command(version)]
struct CliArgs {
    /// JSON file to process (reads from stdin if not provided)
    #[arg(short, long, value_name = "FILE")]
    file: Option<String>,
}


fn load_json_data(args: &CliArgs, stdin_input: &str) -> Result<serde_json::Value> {
    if let Some(file_path) = &args.file {
        let file_content = fs::read_to_string(file_path)?;
        Ok(serde_json::from_str(&file_content)?)
    } else if stdin_input.trim().is_empty() {
        Ok(serde_json::Value::String(String::new()))
    } else {
        Ok(serde_json::from_str(stdin_input)?)
    }
}

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
    let cli_args = CliArgs::parse();

    let input_string = read_stdin()?;
    let json_value = load_json_data(&cli_args, &input_string)?;

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
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn test_app_creation() {
        let app = App::new(json!({"test": "data"}));
        assert_eq!(app.input(), "");
        assert!(!app.should_exit());
    }

    #[test]
    fn test_cli_args_default() {
        use clap::Parser;
        let args = CliArgs::parse_from(&["rjq"]);
        assert_eq!(args.file, None);
    }

    #[test]
    fn test_cli_args_with_file_long() {
        use clap::Parser;
        let args = CliArgs::parse_from(&["rjq", "--file", "test.json"]);
        assert_eq!(args.file, Some("test.json".to_string()));
    }

    #[test]
    fn test_cli_args_with_file_short() {
        use clap::Parser;
        let args = CliArgs::parse_from(&["rjq", "-f", "test.json"]);
        assert_eq!(args.file, Some("test.json".to_string()));
    }

    #[test]
    fn test_cli_args_help() {
        use clap::Parser;
        let result = CliArgs::try_parse_from(&["rjq", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_load_json_from_stdin_empty() {
        use clap::Parser;
        let args = CliArgs::parse_from(&["rjq"]);
        let result = load_json_data(&args, "").unwrap();
        assert_eq!(result, serde_json::Value::String(String::new()));
    }

    #[test]
    fn test_load_json_from_stdin_with_data() {
        use clap::Parser;
        let args = CliArgs::parse_from(&["rjq"]);
        let input = r#"{"key": "value"}"#;
        let result = load_json_data(&args, input).unwrap();
        assert_eq!(result, json!({"key": "value"}));
    }

    #[test]
    fn test_load_json_from_file() {
        use clap::Parser;
        let temp_file = "test_temp.json";
        let test_data = json!({"test": "file_data"});

        // Create temporary test file
        fs::write(temp_file, test_data.to_string()).expect("Failed to write test file");

        let args = CliArgs::parse_from(&["rjq", "-f", temp_file]);
        let result = load_json_data(&args, "").unwrap();

        // Clean up
        fs::remove_file(temp_file).ok();

        assert_eq!(result, test_data);
    }
}
