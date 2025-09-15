pub mod app;
pub mod events;
pub mod handler;
pub mod terminal;

pub use events::{Action, get_action, update};
pub use handler::{DefaultEventHandler, EventHandler};
pub use terminal::restore_terminal;
