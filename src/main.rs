use clap::Parser;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use rjq::{app::App, ui::restore_terminal};
use std::io;

#[derive(Parser)]
#[command(name = "rjq")]
#[command(about = "A terminal UI for jq")]
struct CliArgs {
    /// JSON file to read
    file: Option<String>,
    
    /// Custom prompt string
    #[arg(short, long, default_value = "query > ")]
    prompt: String,
    
    /// Visible height for terminal view
    #[arg(short = 'H', long, default_value = "20")]
    height: usize,
}

fn main() -> rjq::Result<()> {
    let args = CliArgs::parse();
    let stdin_input = read_stdin();
    
    let json_data = load_json_data(&args, &stdin_input)?;
    
    // Create app config from command line arguments
    let config = rjq::app::AppConfig::with_prompt_and_height(
        Box::leak(args.prompt.into_boxed_str()),
        args.height,
    );
    
    let mut app = App::with_config(json_data, config);
    
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // Run the app
    let result = app.run(&mut terminal);
    
    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    
    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }
    
    Ok(())
}

fn read_stdin() -> String {
    use std::io::Read;
    let mut buffer = String::new();
    let _ = io::stdin().read_to_string(&mut buffer);
    buffer
}

fn load_json_data(args: &CliArgs, stdin_input: &str) -> rjq::Result<serde_json::Value> {
    if let Some(filename) = &args.file {
        let content = std::fs::read_to_string(filename)
            .map_err(|e| rjq::app::AppError::Io(e))?;
        serde_json::from_str(&content)
            .map_err(|e| rjq::app::AppError::JsonParse(e))
    } else if !stdin_input.trim().is_empty() {
        serde_json::from_str(stdin_input)
            .map_err(|e| rjq::app::AppError::JsonParse(e))
    } else {
        Ok(serde_json::Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    
    #[test]
    fn test_load_json_data_empty() {
        use clap::Parser;
        let args = CliArgs::parse_from(["rjq"]);
        let result = load_json_data(&args, "").unwrap();
        assert_eq!(result, serde_json::Value::Null);
    }
    
    #[test]
    fn test_load_json_data_from_stdin() {
        use clap::Parser;
        let args = CliArgs::parse_from(["rjq"]);
        let json_str = r#"{"name": "test", "value": 42}"#;
        let result = load_json_data(&args, json_str).unwrap();
        assert_eq!(result, json!({"name": "test", "value": 42}));
    }
    
    #[test]
    fn test_load_json_data_invalid_json() {
        use clap::Parser;
        let args = CliArgs::parse_from(["rjq"]);
        let invalid_json = "{ invalid json }";
        let result = load_json_data(&args, invalid_json);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_cli_args_parsing() {
        use clap::Parser;
        let args = CliArgs::parse_from(["rjq", "--prompt", "custom> ", "--height", "30"]);
        assert_eq!(args.prompt, "custom> ");
        assert_eq!(args.height, 30);
    }
}