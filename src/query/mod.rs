use jaq_core::{
    Ctx, RcIter,
    load::{Arena, File, Loader},
};
use jaq_json::Val;
use crate::app::error::AppError;

#[derive(Debug)]
pub enum QueryResult {
    Single(serde_json::Value),
    Multiple(Vec<serde_json::Value>),
    Empty,
}

impl QueryResult {
    pub fn format_pretty(&self) -> String {
        match self {
            QueryResult::Single(val) => {
                serde_json::to_string_pretty(val)
                    .unwrap_or_else(|_| "Error formatting result".to_string())
            }
            QueryResult::Multiple(vals) => {
                serde_json::to_string_pretty(vals)
                    .unwrap_or_else(|_| "Error formatting result".to_string())
            }
            QueryResult::Empty => "null".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct JsonData {
    inner: serde_json::Value,
}

impl JsonData {
    pub fn new(value: serde_json::Value) -> Self {
        Self { inner: value }
    }

    pub fn get(&self) -> &serde_json::Value {
        &self.inner
    }

    pub fn execute_query(&self, query: &str) -> crate::Result<QueryResult> {
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
        let results = filter.run((Ctx::new([], &inputs), Val::from(self.inner.clone())));
        let values: Vec<serde_json::Value> = results
            .into_iter()
            .filter_map(|r| r.ok())
            .map(|val| val.into())
            .collect();

        Ok(match values.len() {
            0 => QueryResult::Empty,
            1 => QueryResult::Single(values.into_iter().next().unwrap()),
            _ => QueryResult::Multiple(values),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_data_creation() {
        let data = JsonData::new(json!({"test": "value"}));
        assert_eq!(data.get(), &json!({"test": "value"}));
    }

    #[test]
    fn test_identity_query() {
        let data = JsonData::new(json!({"name": "test"}));
        let result = data.execute_query(".").unwrap();

        match result {
            QueryResult::Single(val) => assert_eq!(val, json!({"name": "test"})),
            _ => panic!("Expected single result"),
        }
    }

    #[test]
    fn test_query_formatting() {
        let result = QueryResult::Single(json!({"key": "value"}));
        let formatted = result.format_pretty();
        assert!(formatted.contains("key"));
        assert!(formatted.contains("value"));
    }
}