use super::{QueryCache, QueryExecutor};
use crate::app::error::AppError;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

pub struct CachedQueryExecutor<E: QueryExecutor, C: QueryCache> {
    executor: E,
    cache: RefCell<C>,
}

impl<E: QueryExecutor, C: QueryCache> CachedQueryExecutor<E, C> {
    pub fn new(executor: E, cache: C) -> Self {
        Self {
            executor,
            cache: RefCell::new(cache),
        }
    }

    fn cache_key(data: &Value, query: &str) -> String {
        let mut hasher = DefaultHasher::new();
        data.to_string().hash(&mut hasher);
        query.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

impl<E: QueryExecutor, C: QueryCache> QueryExecutor for CachedQueryExecutor<E, C> {
    fn execute(&self, data: &Value, query: &str) -> Result<Vec<Value>, AppError> {
        let key = Self::cache_key(data, query);

        if let Some(cached_result) = self.cache.borrow().get(&key) {
            return Ok(cached_result);
        }

        let result = self.executor.execute(data, query)?;
        self.cache.borrow_mut().set(key, result.clone());

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::{InMemoryQueryCache, JaqQueryExecutor};
    use serde_json::json;

    #[test]
    fn test_cached_query_executor() {
        let executor = JaqQueryExecutor;
        let cache = InMemoryQueryCache::new();
        let cached_executor = CachedQueryExecutor::new(executor, cache);

        let data = json!({"name": "test", "value": 42});

        let result1 = cached_executor.execute(&data, ".name").unwrap();
        let result2 = cached_executor.execute(&data, ".name").unwrap();

        assert_eq!(result1, result2);
        assert_eq!(result1.len(), 1);
        assert_eq!(result1[0], json!("test"));
    }

    #[test]
    fn test_cache_key_generation() {
        let data = json!({"test": "data"});
        let key1 =
            CachedQueryExecutor::<JaqQueryExecutor, InMemoryQueryCache>::cache_key(&data, ".test");
        let key2 =
            CachedQueryExecutor::<JaqQueryExecutor, InMemoryQueryCache>::cache_key(&data, ".test");
        let key3 =
            CachedQueryExecutor::<JaqQueryExecutor, InMemoryQueryCache>::cache_key(&data, ".other");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }
}
