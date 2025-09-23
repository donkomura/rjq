use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("JSON parsing error: {0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("Query compilation error: {0}")]
    QueryCompile(String),
    #[error("Query execution error: {0}")]
    QueryExecution(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
