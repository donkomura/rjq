use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crate::app::App;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    Input(char),
    Backspace,
    Clear,
    None,
}

pub fn get_action(key_event: KeyEvent) -> Action {
    match key_event.code {
        KeyCode::Esc => Action::Quit,
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Char(c) => {
            if c == '\n' {
                Action::Clear
            } else {
                Action::Input(c)
            }
        }
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Enter => Action::Clear,
        _ => Action::None,
    }
}

pub fn update(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.set_exit(true),
        Action::Input(c) => app.push_char(c),
        Action::Backspace => {
            if !app.input().is_empty() {
                app.pop_char();
            }
        }
        Action::Clear => app.clear_input(),
        Action::None => {}
    }
}