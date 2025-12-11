use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: String,           // "light", "dark", "auto"
    pub language: String,        // "en", "zh", etc.
    pub auto_start: bool,        // Start with system
    pub log_level: String,       // "debug", "info", "warn", "error"
    pub default_key_path: Option<String>,
    pub window_width: u32,
    pub window_height: u32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: "en".to_string(),
            auto_start: false,
            log_level: "info".to_string(),
            default_key_path: None,
            window_width: 1200,
            window_height: 800,
        }
    }
}