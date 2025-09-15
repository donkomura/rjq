#[derive(Debug, Clone)]
pub struct AppConfig {
    pub prompt: &'static str,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prompt: "query > ",
        }
    }
}

impl AppConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_prompt(prompt: &'static str) -> Self {
        Self { prompt }
    }
}