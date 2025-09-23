pub mod app;
pub mod events;
pub mod handler;
pub mod syntax;
pub mod terminal;

pub use events::{Action, get_action, update};
pub use handler::{DefaultEventHandler, EventHandler};
pub use syntax::SyntaxHighlighter;
pub use terminal::restore_terminal;
