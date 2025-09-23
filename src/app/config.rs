#[derive(Debug, Clone)]
pub struct AppConfig {
    pub prompt: &'static str,
    pub visible_height: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            prompt: "query > ",
            visible_height: 20,
        }
    }
}

impl AppConfig {
    pub fn with_prompt(prompt: &'static str) -> Self {
        Self {
            prompt,
            visible_height: 20,
        }
    }

    pub fn with_visible_height(visible_height: usize) -> Self {
        Self {
            prompt: "query > ",
            visible_height,
        }
    }

    pub fn with_prompt_and_height(prompt: &'static str, visible_height: usize) -> Self {
        Self {
            prompt,
            visible_height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.prompt, "query > ");
        assert_eq!(config.visible_height, 20);
    }

    #[test]
    fn test_custom_prompt() {
        let config = AppConfig::with_prompt("test > ");
        assert_eq!(config.prompt, "test > ");
        assert_eq!(config.visible_height, 20);
    }

    #[test]
    fn test_custom_visible_height() {
        let config = AppConfig::with_visible_height(30);
        assert_eq!(config.prompt, "query > ");
        assert_eq!(config.visible_height, 30);
    }

    #[test]
    fn test_custom_prompt_and_height() {
        let config = AppConfig::with_prompt_and_height("test > ", 25);
        assert_eq!(config.prompt, "test > ");
        assert_eq!(config.visible_height, 25);
    }
}