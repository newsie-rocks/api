//! Configuration

/// Configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// API URL
    pub api_url: String,
    /// Authentication token
    pub token: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_url: "http://localhost:3000".to_string(),
            token: None,
        }
    }
}
