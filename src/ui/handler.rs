use crate::app::App;
use crate::ui::events::Action;
use crossterm::event::KeyEvent;

pub trait EventHandler {
    fn handle_key_event(&self, key_event: KeyEvent) -> Action;
    fn update_app(&self, app: &mut App, action: Action);
}

pub struct DefaultEventHandler;

impl EventHandler for DefaultEventHandler {
    fn handle_key_event(&self, key_event: KeyEvent) -> Action {
        crate::ui::events::get_action(key_event)
    }

    fn update_app(&self, app: &mut App, action: Action) {
        crate::ui::events::update(app, action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};
    use serde_json::json;

    #[test]
    fn test_default_event_handler() {
        let handler = DefaultEventHandler;
        let key_event = crossterm::event::KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

        let action = handler.handle_key_event(key_event);
        assert_eq!(action, Action::Input('a'));
    }

    #[test]
    fn test_update_app() {
        let handler = DefaultEventHandler;
        let mut app = App::new(json!({"test": "data"}));

        handler.update_app(&mut app, Action::Input('a'));
        assert_eq!(app.input(), "a");
    }
}
