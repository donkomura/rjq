# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2026-11-24

### Added
- History-based inline query suggestions, accepted with `Tab`
  - Candidates are ranked by usage frequency and recency
- Selectable syntax highlighting themes via `--theme <NAME>` or `RJQ_THEME`
  - Built-in themes: `ansi` (default), `eighties-dark`, `github`, `mocha-dark`, `ocean-dark`, `ocean-light`, `solarized-dark`, `solarized-light`
  - External TextMate `.tmTheme` files placed in `~/.config/rjq/themes`, referred to by name
- `--list-themes` to show available themes
- Keep showing the last matching result while typing a query that has no match yet
  - The status line shows which query is actually displayed
  - `--no-match-delay <SECONDS>` (default: 1) controls how long to wait before showing the non-matching result
- Streaming input: values are parsed incrementally and results are shown while reading, including NDJSON input
- Status line showing input loading progress

### Changed
- Object keys are displayed in input order (same as jq) instead of alphabetical order
- Exit with status code 1 on runtime errors
- Greatly reduced memory usage for large inputs (peak RSS for a 97MB JSON dropped from about 4.6GB to about 0.8GB)

### Fixed
- Startup failure (`Failed to initialize input reader`) when JSON is piped via stdin on macOS
- Panic when a huge value is passed to `--no-match-delay`
- Scroll position resetting on every keystroke even when the displayed result does not change
- Error and previous result alternating on each keystroke while the query has a compile error

## [0.1.0] - 2025-09-23

### Added
- Initial release of rjq - An interactive CLI tool for JSON querying
- Interactive TUI with real-time JSON processing using ratatui
- jaq integration for JSON querying with jq-compatible syntax
- JSON syntax highlighting with color-coded elements
  - Keys: Blue
  - Strings: Green
  - Numbers: Cyan
  - Booleans: Yellow
  - Null values: Gray
  - Brackets: White
  - Punctuation: Gray
- Scrolling support for navigating large JSON files with arrow keys
- File and stdin input support
- Command line options: `-f/--file`, `--help`, `--version`
- Cross-platform binary distribution for:
  - macOS: x86_64 (Intel) and aarch64 (Apple Silicon)
  - Linux: x86_64 and aarch64 (GNU and musl variants)
  - Windows: x86_64 (MSVC)
- Comprehensive error handling with clear error messages
- CI/CD pipeline with automated testing and releases

### Technical Details
- Built with Rust using Edition 2024
- Uses jaq-core, jaq-json, jaq-std for JSON processing
- Terminal UI powered by ratatui and crossterm
- Custom JSON tokenizer for syntax highlighting
- Automated release workflow with taiki-e actions

[Unreleased]: https://github.com/donkomura/rjq/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/donkomura/rjq/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/donkomura/rjq/releases/tag/v0.1.0