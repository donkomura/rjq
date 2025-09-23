# rjq

An interactive CLI tool for JSON querying.

## Features

- **Interactive TUI**: Terminal-based user interface with real-time JSON processing
- **jaq Integration**: Uses the powerful jaq library for JSON querying with jq-compatible syntax
- **Syntax Highlighting**: Beautiful color-coded JSON output for better readability
  - Keys: Blue
  - Strings: Green
  - Numbers: Cyan
  - Booleans: Yellow
  - Null values: Gray
  - Brackets: White
  - Punctuation: Gray
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
# Process JSON from a file
rjq data.json

# Process JSON from stdin
echo '{"name": "John", "age": 30}' | rjq

# Use with a file argument
rjq -f data.json
```

### Interactive Mode

Once rjq starts, you can:

1. **Enter jq queries**: Type any jq-compatible query in the input field
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
Usage: rjq [OPTIONS] [JSON_FILE]

Arguments:
  [JSON_FILE]  JSON file to process (optional, will read from stdin if not provided)

Options:
  -f, --file <FILE>  JSON file to process
  -h, --help         Print help
  -V, --version      Print version
```

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