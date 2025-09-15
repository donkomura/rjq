pub mod events;
pub mod app;
pub mod terminal;
pub mod handler;

pub use events::{Action, get_action, update};
pub use terminal::restore_terminal;
pub use handler::{EventHandler, DefaultEventHandler};