pub mod app;
pub mod query;
pub mod ui;

// 公開API
pub use app::{App, AppBuilder, AppConfig, AppError, AppState, EnhancedApp};
pub use query::{
    CachedQueryExecutor, InMemoryQueryCache, JaqQueryExecutor, JsonData, QueryCache, QueryExecutor,
    QueryResult,
};
pub use ui::{Action, DefaultEventHandler, EventHandler, get_action, restore_terminal, update};

pub type Result<T> = std::result::Result<T, AppError>;
