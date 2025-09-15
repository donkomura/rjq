use serde_json::Value;
use std::collections::HashMap;

pub trait QueryCache {
    fn get(&self, key: &str) -> Option<Vec<Value>>;
    fn set(&mut self, key: String, value: Vec<Value>);
    fn clear(&mut self);
}

pub struct InMemoryQueryCache {
    cache: HashMap<String, Vec<Value>>,
}

impl InMemoryQueryCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }
}

impl Default for InMemoryQueryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryCache for InMemoryQueryCache {
    fn get(&self, key: &str) -> Option<Vec<Value>> {
        self.cache.get(key).cloned()
    }

    fn set(&mut self, key: String, value: Vec<Value>) {
        self.cache.insert(key, value);
    }

    fn clear(&mut self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_in_memory_cache() {
        let mut cache = InMemoryQueryCache::new();
        let key = "test_query";
        let value = vec![json!("result")];

        assert!(cache.get(key).is_none());

        cache.set(key.to_string(), value.clone());
        assert_eq!(cache.get(key), Some(value));

        cache.clear();
        assert!(cache.get(key).is_none());
    }
}
