# rjq

An interactive CLI tool for JSON querying.

## Features

- **Interactive TUI**: Terminal-based user interface with real-time JSON processing
- **jaq Integration**: Uses the powerful jaq library for JSON querying with jq-compatible syntax
- **Syntax Highlighting**: Beautiful color-coded JSON output for better readability
- **Scrolling Support**: Navigate through large JSON files with Up/Down arrow keys
- **File and Stdin Input**: Process JSON from files or pipe data directly
- **Error Handling**: Clear error messages for invalid queries or malformed JSON

## Installation

### From Source

```bash
git clone https://github.com/donkomura/rjq.git
cd rjq
cargo build --release
```

The binary will be available at `target/release/rjq`.

## Usage

### Basic Usage

```bash
# Process JSON from stdin
echo '{"name": "John", "age": 30}' | rjq

# Use with a file argument
rjq -f data.json
```

### Interactive Mode

Once rjq starts, you can:

1. **Enter jq queries**: Type any jq-compatible query in the input field
   - While the query is incomplete or matches nothing (compile error, no output, or only `null`), the last matching result stays visible and the status line shows which query it belongs to. The actual result appears after 1 second without typing (configurable with `--no-match-delay`)
2. **Navigate results**: Use `↑`/`↓` arrow keys to scroll through large JSON outputs
3. **Clear input**: Press `Ctrl+U` to clear the current query
4. **Exit**: Press `Ctrl+C` or `q` to quit

### Example Queries

```bash
# Identity (show all data)
.

# Get a specific field
.name

# Filter arrays
.users[] | select(.age > 25)

# Map over arrays
.items | map(.price * 1.1)

# Get object keys
keys

# Get array length
.users | length
```

## Command Line Options

```
Usage: rjq [OPTIONS]

Options:
  -f, --file <FILE>               JSON file to process (reads from stdin if not provided)
      --no-match-delay <SECONDS>  Seconds to keep showing the last matching result while the query matches nothing [default: 1]
      --theme <NAME>              Syntax highlighting theme: a built-in name or the name of a .tmTheme file in ~/.config/rjq/themes [env: RJQ_THEME=]
      --list-themes               List available syntax highlighting themes and exit
  -h, --help                      Print help
  -V, --version                   Print version
```

## Syntax Highlighting Themes

The default `ansi` theme uses the 16 ANSI colors, so it follows your terminal's palette. If the colors are hard to read, pick another theme with `--theme` or the `RJQ_THEME` environment variable. `--theme` wins when both are set.

```bash
# List built-in themes and themes in the theme directory
rjq --list-themes

# Use a built-in theme
rjq --theme ocean-dark -f data.json

# Set a default theme in your shell profile
export RJQ_THEME=solarized-dark
```

External themes use the TextMate `.tmTheme` format, the same format used by Sublime Text, bat, and delta. Put a theme file in `$XDG_CONFIG_HOME/rjq/themes` (`~/.config/rjq/themes` when `XDG_CONFIG_HOME` is unset) and refer to it by its file name without the extension. File paths are not accepted by `--theme`:

```bash
mkdir -p ~/.config/rjq/themes
cp Dracula.tmTheme ~/.config/rjq/themes/
rjq --theme Dracula -f data.json

# Reuse the themes you already have for bat
cp ~/.config/bat/themes/*.tmTheme ~/.config/rjq/themes/
```

Built-in themes are `ansi`, `eighties-dark`, `github`, `mocha-dark`, `ocean-dark`, `ocean-light`, `solarized-dark`, and `solarized-light`.

A theme name is resolved in this order: a built-in theme name (case-insensitive), then `<NAME>.tmTheme` in the theme directory. rjq exits with an error if none of them match.

Theme colors other than `ansi` are drawn in 24-bit color, and only the foreground colors and font styles are applied. The terminal background is kept, so choose a theme that matches your terminal's light or dark background.

## Architecture

- **Backend**: jaq library for JSON processing
- **Frontend**: ratatui for terminal UI
- **Syntax Highlighting**: Custom tokenizer with real-time color coding
- **Event Handling**: Crossterm for keyboard input management

## Examples

### Processing a simple JSON file

```json
{
  "users": [
    {"name": "Alice", "age": 30, "active": true},
    {"name": "Bob", "age": 25, "active": false}
  ]
}
```

Queries you can try:
- `.users[0].name` → `"Alice"`
- `.users[] | select(.active)` → `{"name": "Alice", "age": 30, "active": true}`
- `.users | map(.age)` → `[30, 25]`

## Development

### Requirements

- Rust 1.70 or later
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Linting

```bash
cargo clippy
cargo fmt
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.
