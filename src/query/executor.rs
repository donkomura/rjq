use crate::app::error::AppError;
use serde_json::Value;
use jaq_core::{
    Ctx, RcIter,
    load::{Arena, File, Loader},
};
use jaq_json::Val;

pub trait QueryExecutor {
    fn execute(&self, data: &Value, query: &str) -> Result<Vec<Value>, AppError>;
}

pub struct JaqQueryExecutor;

impl QueryExecutor for JaqQueryExecutor {
    fn execute(&self, data: &Value, query: &str) -> Result<Vec<Value>, AppError> {
        if query.is_empty() {
            return Err(AppError::QueryCompile("Empty query".to_string()));
        }

        let program = File {
            code: query,
            path: (),
        };
        let loader = Loader::new(jaq_std::defs().chain(jaq_json::defs()));
        let arena = Arena::default();
        let modules = loader.load(&arena, program).map_err(|e| {
            AppError::QueryCompile(format!("Loader: {:?}", e))
        })?;
        let filter = jaq_core::Compiler::default()
            .with_funs(jaq_std::funs().chain(jaq_json::funs()))
            .compile(modules)
            .map_err(|e| AppError::QueryCompile(format!("Compiler: {:?}", e)))?;

        let inputs = RcIter::new(core::iter::empty());
        let results = filter.run((Ctx::new([], &inputs), Val::from(data.clone())));
        let values: Vec<Value> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|val| val.into())
            .collect();

        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_jaq_query_executor() {
        let executor = JaqQueryExecutor;
        let data = json!({"name": "test", "value": 42});

        let result = executor.execute(&data, ".name").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], json!("test"));
    }

    #[test]
    fn test_invalid_query() {
        let executor = JaqQueryExecutor;
        let data = json!({"test": "data"});

        let result = executor.execute(&data, "invalid query syntax");
        assert!(result.is_err());
    }
}