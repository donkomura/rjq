#[derive(Debug, Clone)]
pub struct AppConfig {
    pub prompt: &'static str,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { prompt: "query > " }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.prompt, "query > ");
    }

    #[test]
    fn test_custom_prompt() {
        let config = AppConfig::with_prompt("custom > ");
        assert_eq!(config.prompt, "custom > ");
    }
}
