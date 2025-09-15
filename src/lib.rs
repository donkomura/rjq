pub mod app;
pub mod query;
pub mod ui;

// 公開API
pub use app::{App, AppConfig, AppState};
pub use query::{JsonData, QueryResult};
pub use ui::{Action, get_action, update};
pub use app::error::AppError;

pub type Result<T> = std::result::Result<T, AppError>;