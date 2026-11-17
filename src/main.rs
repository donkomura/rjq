use clap::Parser;
use crossterm::{event::EnableMouseCapture, execute, terminal::enable_raw_mode};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::fs::File;
use std::io::{self, IsTerminal, Read};
use std::time::Duration;

use rjq::ui::theme::{DEFAULT_THEME, default_themes_dir};
use rjq::{App, AppConfig, Result, ThemeCatalog, restore_terminal, spawn_reader};

/// A command-line jq processor with interactive TUI
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(name = "rjq")]
#[command(about = "Interactive jq query processor for JSON data")]
#[command(version)]
struct CliArgs {
    /// JSON file to process (reads from stdin if not provided)
    #[arg(short, long, value_name = "FILE")]
    file: Option<String>,

    /// Seconds to keep showing the last matching result while the query matches nothing
    #[arg(long, value_name = "SECONDS", default_value = "1", value_parser = parse_seconds)]
    no_match_delay: Duration,

    /// Syntax highlighting theme: a built-in name or the name of a .tmTheme file in ~/.config/rjq/themes
    #[arg(long, value_name = "NAME", env = "RJQ_THEME")]
    theme: Option<String>,

    /// List available syntax highlighting themes and exit
    #[arg(long)]
    list_themes: bool,
}

fn parse_seconds(value: &str) -> std::result::Result<Duration, String> {
    let seconds: f64 = value.parse().map_err(|e| format!("{e}"))?;
    Duration::try_from_secs_f64(seconds).map_err(|e| format!("{e}"))
}

fn open_input(file: Option<&str>, stdin_is_terminal: bool) -> io::Result<Box<dyn Read + Send>> {
    match file {
        Some(path) => Ok(Box::new(File::open(path)?)),
        None if stdin_is_terminal => Ok(Box::new(io::empty())),
        None => Ok(Box::new(io::stdin())),
    }
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli_args = CliArgs::parse();
    let catalog = ThemeCatalog::new(default_themes_dir());

    if cli_args.list_themes {
        for name in catalog.names()? {
            println!("{name}");
        }
        return Ok(());
    }

    let theme = catalog.resolve(cli_args.theme.as_deref().unwrap_or(DEFAULT_THEME))?;

    let reader = open_input(cli_args.file.as_deref(), io::stdin().is_terminal())?;
    let inputs = spawn_reader(reader);

    enable_raw_mode()?;
    let mut stderr = std::io::stderr();
    execute!(
        stderr,
        crossterm::terminal::EnterAlternateScreen,
        EnableMouseCapture
    )?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    let config = AppConfig {
        no_match_delay: cli_args.no_match_delay,
        ..AppConfig::default()
    }
    .with_theme(theme);
    let mut app = App::loading(config);
    let res = app.run(&mut terminal, inputs);

    restore_terminal(&mut terminal).ok();

    res
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
        let args = CliArgs::parse_from(["rjq"]);
        assert_eq!(args.file, None);
    }

    #[test]
    fn test_cli_args_with_file_long() {
        use clap::Parser;
        let args = CliArgs::parse_from(["rjq", "--file", "test.json"]);
        assert_eq!(args.file, Some("test.json".to_string()));
    }

    #[test]
    fn test_cli_args_with_file_short() {
        use clap::Parser;
        let args = CliArgs::parse_from(["rjq", "-f", "test.json"]);
        assert_eq!(args.file, Some("test.json".to_string()));
    }

    #[test]
    fn test_cli_args_no_match_delay() {
        use clap::Parser;
        use std::time::Duration;
        let tests = vec![
            ("default", vec!["rjq"], Duration::from_secs(1)),
            (
                "fractional seconds",
                vec!["rjq", "--no-match-delay", "0.5"],
                Duration::from_millis(500),
            ),
            ("zero", vec!["rjq", "--no-match-delay", "0"], Duration::ZERO),
        ];

        for (name, argv, expected) in tests {
            let args = CliArgs::parse_from(argv);
            assert_eq!(expected, args.no_match_delay, "{name}");
        }
    }

    #[test]
    fn test_cli_args_rejects_invalid_no_match_delay() {
        use clap::Parser;
        for value in ["-1", "abc", "inf", "NaN"] {
            let result = CliArgs::try_parse_from(["rjq", "--no-match-delay", value]);
            assert!(result.is_err(), "{value}");
        }
    }

    #[test]
    fn test_cli_args_theme() {
        use clap::Parser;
        let tests = vec![
            ("no theme", vec!["rjq"], None, false),
            (
                "theme name",
                vec!["rjq", "--theme", "ocean-dark"],
                Some("ocean-dark".to_string()),
                false,
            ),
            ("list themes", vec!["rjq", "--list-themes"], None, true),
        ];

        for (name, argv, theme, list_themes) in tests {
            let args = CliArgs::parse_from(argv);
            assert_eq!(theme, args.theme, "{name}");
            assert_eq!(list_themes, args.list_themes, "{name}");
        }
    }

    #[test]
    fn test_cli_args_help() {
        use clap::Parser;
        let result = CliArgs::try_parse_from(["rjq", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayHelp);
    }

    #[test]
    fn test_open_input() {
        let path = std::env::temp_dir().join(format!("rjq_open_input_{}.json", std::process::id()));
        fs::write(&path, r#"{"test": "file_data"}"#).unwrap();
        let path_str = path.to_str().unwrap().to_string();

        let tests = vec![
            (
                "file is read even when stdin is a terminal",
                Some(path_str.clone()),
                true,
                r#"{"test": "file_data"}"#,
            ),
            (
                "terminal stdin without file yields no input",
                None,
                true,
                "",
            ),
        ];

        for (name, file, stdin_is_terminal, expected) in tests {
            let mut content = String::new();
            open_input(file.as_deref(), stdin_is_terminal)
                .unwrap()
                .read_to_string(&mut content)
                .unwrap();
            assert_eq!(expected, content, "{name}");
        }
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_open_input_missing_file() {
        let result = open_input(Some("/nonexistent/rjq/input.json"), true);

        assert!(result.is_err());
    }
}
