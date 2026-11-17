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
        }
        Action::Backspace => {
            if !app.input().is_empty() {
                app.pop_char();
            }
        }
        Action::Clear => {
            if !app.input().trim().is_empty() {
                // Record in history when executing a query
                app.record_query(app.input().to_string());
            }
            app.clear_input();
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
    use crate::app::AppConfig;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_editing_query_keeps_scroll_until_result_changes() {
        let items: Vec<u32> = (0..50).collect();
        let mut app = App::with_config(
            serde_json::json!({ "items": items }),
            AppConfig::with_visible_height(10),
        );
        for c in ".items".chars() {
            update(&mut app, Action::Input(c));
        }
        update(&mut app, Action::ScrollDown);
        update(&mut app, Action::ScrollDown);
        assert_eq!(2, app.scroll_offset());

        update(&mut app, Action::Input('.'));
        assert_eq!(
            2,
            app.scroll_offset(),
            "unmatched input keeps the shown result"
        );

        update(&mut app, Action::Backspace);
        assert_eq!(2, app.scroll_offset(), "editing back to the shown query");

        update(&mut app, Action::Input('['));
        update(&mut app, Action::Input('0'));
        update(&mut app, Action::Input(']'));
        assert_eq!(
            0,
            app.scroll_offset(),
            "a new matching result starts at the top"
        );
    }

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
