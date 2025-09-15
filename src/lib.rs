pub mod app;
pub mod query;
pub mod ui;

// 公開API
pub use app::{App, AppConfig, AppState, AppError, AppBuilder, EnhancedApp};
pub use query::{JsonData, QueryResult, QueryExecutor, JaqQueryExecutor, QueryCache, InMemoryQueryCache, CachedQueryExecutor};
pub use ui::{Action, get_action, update, restore_terminal, EventHandler, DefaultEventHandler};

pub type Result<T> = std::result::Result<T, AppError>;