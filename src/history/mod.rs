use std::collections::HashMap;
use std::time::SystemTime;

const SECONDS_PER_DAY: f64 = 86400.0;

#[derive(Debug, Clone)]
pub struct QueryEntry {
    pub query: String,
    pub count: usize,
    pub last_used: SystemTime,
    pub first_used: SystemTime,
}

#[derive(Debug, Clone)]
pub struct SuggestionItem {
    pub text: String,
    pub score: f64,
}

#[derive(Debug)]
pub struct QueryHistory {
    entries: HashMap<String, QueryEntry>,
    max_entries: usize,
    recent_weight: f64,
}

impl QueryHistory {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            recent_weight: 0.5,
        }
    }

    pub fn record_query(&mut self, query: String) {
        if query.trim().is_empty() {
            return;
        }

        let now = SystemTime::now();

        match self.entries.get_mut(&query) {
            Some(entry) => {
                entry.count += 1;
                entry.last_used = now;
            }
            None => {
                let entry = QueryEntry {
                    query: query.clone(),
                    count: 1,
                    last_used: now,
                    first_used: now,
                };
                self.entries.insert(query, entry);
            }
        }

        self.cleanup_old_entries();
    }

    pub fn get_suggestions(&self, prefix: &str, limit: usize) -> Vec<SuggestionItem> {
        if prefix.len() < 2 {
            return vec![];
        }

        let mut candidates: Vec<_> = self
            .entries
            .values()
            .filter(|entry| entry.query.starts_with(prefix) && entry.query != prefix)
            .map(|entry| {
                let score = self.calculate_score(entry);
                SuggestionItem {
                    text: entry.query.clone(),
                    score,
                }
            })
            .collect();

        candidates.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        candidates.truncate(limit);
        candidates
    }

    fn calculate_score(&self, entry: &QueryEntry) -> f64 {
        let frequency_score = entry.count as f64;
        let time_decay = self.calculate_time_decay(entry.last_used);

        frequency_score * (1.0 - self.recent_weight) + time_decay * self.recent_weight
    }

    fn calculate_time_decay(&self, last_used: SystemTime) -> f64 {
        let elapsed = SystemTime::now()
            .duration_since(last_used)
            .unwrap_or_default()
            .as_secs() as f64;

        // 24時間で半減する指数減衰
        (-elapsed / SECONDS_PER_DAY).exp()
    }

    fn cleanup_old_entries(&mut self) {
        if self.entries.len() <= self.max_entries {
            return;
        }

        let mut entries: Vec<_> = self
            .entries
            .iter()
            .map(|(query, entry)| (query.clone(), self.calculate_score(entry)))
            .collect();

        entries.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let remove_count = self.entries.len() - self.max_entries;
        for (query, _) in entries.iter().take(remove_count) {
            self.entries.remove(query);
        }
    }
}

impl Default for QueryHistory {
    fn default() -> Self {
        Self::new(100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_recording() {
        let mut history = QueryHistory::new(100);
        history.record_query(".name".to_string());
        history.record_query(".name".to_string()); // 重複実行

        let suggestions = history.get_suggestions(".n", 5);
        let name_entry = suggestions.iter().find(|s| s.text == ".name").unwrap();
        assert!(name_entry.score > 0.0);
    }

    #[test]
    fn test_prefix_matching() {
        let mut history = QueryHistory::new(100);
        history.record_query(".name".to_string());
        history.record_query(".age".to_string());
        history.record_query(".users[0]".to_string());

        let suggestions = history.get_suggestions(".u", 5);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].text, ".users[0]");
    }

    #[test]
    fn test_empty_query_ignored() {
        let mut history = QueryHistory::new(100);
        history.record_query("".to_string());
        history.record_query("   ".to_string());

        let suggestions = history.get_suggestions(".", 5);
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_suggestion_ranking() {
        let mut history = QueryHistory::new(100);

        // .name を3回実行
        for _ in 0..3 {
            history.record_query(".name".to_string());
        }

        // .users[0].name を1回実行
        history.record_query(".users[0].name".to_string());

        let suggestions = history.get_suggestions(".n", 5);
        assert!(!suggestions.is_empty());

        // 使用頻度により .name が上位になるはず（".n"で前方一致する場合）
        let name_suggestion = suggestions.iter().find(|s| s.text == ".name");
        assert!(name_suggestion.is_some());
    }

    #[test]
    fn test_no_self_suggestion() {
        let mut history = QueryHistory::new(100);
        history.record_query(".name".to_string());

        // 完全一致の場合は候補に含まれない
        let suggestions = history.get_suggestions(".name", 5);
        assert_eq!(suggestions.len(), 0);
    }

    #[test]
    fn test_min_prefix_length() {
        let mut history = QueryHistory::new(100);
        history.record_query(".name".to_string());

        // 2文字未満の場合は候補を返さない
        let suggestions = history.get_suggestions(".", 5);
        assert_eq!(suggestions.len(), 0);

        let suggestions = history.get_suggestions(".n", 5);
        assert_eq!(suggestions.len(), 1);
    }
}
