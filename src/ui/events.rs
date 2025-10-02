use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Quit,
    Input(char),
    Backspace,
    Clear,
    ScrollUp,
    ScrollDown,
    Tab,
    None,
}

pub fn get_action(key_event: KeyEvent) -> Action {
    match key_event.code {
        KeyCode::Esc => Action::Quit,
        KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
        KeyCode::Up => Action::ScrollUp,
        KeyCode::Down => Action::ScrollDown,
        KeyCode::Char(c) => {
            if c == '\n' {
                Action::Clear
            } else {
                Action::Input(c)
            }
        }
        KeyCode::Backspace => Action::Backspace,
        KeyCode::Enter => Action::Clear,
        KeyCode::Tab => Action::Tab,
        _ => Action::None,
    }
}

pub fn update(app: &mut App, action: Action) {
    match action {
        Action::Quit => app.set_exit(true),
        Action::Input(c) => {
            app.push_char(c);
            app.reset_scroll();
        }
        Action::Backspace => {
            if !app.input().is_empty() {
                app.pop_char();
            }
            app.reset_scroll();
        }
        Action::Clear => {
            if !app.input().trim().is_empty() {
                // Record in history when executing a query
                app.record_query(app.input().to_string());
            }
            app.clear_input();
            app.reset_scroll();
        }
        Action::ScrollUp => app.scroll_up(),
        Action::ScrollDown => app.scroll_down(),
        Action::Tab => {
            // Handle when the TAB key is pressed
            if let Some(suggestion) = app.get_best_suggestion() {
                app.apply_suggestion(suggestion);
            }
        }
        Action::None => {
            // Do nothing for undefined keys
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_key_action_mapping() {
        let action = get_action(crossterm::event::KeyEvent::new(
            KeyCode::Esc,
            KeyModifiers::NONE,
        ));
        assert_eq!(action, Action::Quit);

        let action = get_action(crossterm::event::KeyEvent::new(
            KeyCode::Char('a'),
            KeyModifiers::NONE,
        ));
        assert_eq!(action, Action::Input('a'));

        let action = get_action(crossterm::event::KeyEvent::new(
            KeyCode::Backspace,
            KeyModifiers::NONE,
        ));
        assert_eq!(action, Action::Backspace);

        let action = get_action(crossterm::event::KeyEvent::new(
            KeyCode::Up,
            KeyModifiers::NONE,
        ));
        assert_eq!(action, Action::ScrollUp);

        let action = get_action(crossterm::event::KeyEvent::new(
            KeyCode::Down,
            KeyModifiers::NONE,
        ));
        assert_eq!(action, Action::ScrollDown);

        let action = get_action(crossterm::event::KeyEvent::new(
            KeyCode::Tab,
            KeyModifiers::NONE,
        ));
        assert_eq!(action, Action::Tab);

        // Test space character input
        let action = get_action(crossterm::event::KeyEvent::new(
            KeyCode::Char(' '),
            KeyModifiers::NONE,
        ));
        assert_eq!(action, Action::Input(' '));
    }
}
