# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/donkomura/rjq/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/donkomura/rjq/releases/tag/v0.1.0