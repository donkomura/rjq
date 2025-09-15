pub mod events;
pub mod app;
pub mod terminal;

pub use events::{Action, get_action, update};
pub use terminal::restore_terminal;