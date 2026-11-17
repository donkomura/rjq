pub mod app;
pub mod history;
pub mod input;
pub mod query;
pub mod ui;
pub mod view;

// 公開API
pub use app::{App, AppBuilder, AppConfig, AppError, AppState, EnhancedApp, LoadState};
pub use input::{InputEvent, OwnedVal, spawn_reader};
pub use query::{
    CachedQueryExecutor, CompiledQuery, InMemoryQueryCache, JaqQueryExecutor, JsonData, QueryCache,
    QueryExecutor, QueryResult, into_val,
};
pub use ui::{
    Action, DefaultEventHandler, EventHandler, Theme, ThemeCatalog, ThemeError, get_action,
    restore_terminal, update,
};
pub use view::PrettyDocument;

pub type Result<T> = std::result::Result<T, AppError>;
