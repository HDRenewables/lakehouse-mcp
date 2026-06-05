//! Config handling module
use serde::Deserialize;
use std::collections::HashMap;

/// Configuration loaded at boot time.
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL for the upstream unified API.
    pub api_base: String,
    /// Map of endpoint slugs to their full paths
    /// (e.g., `bill_revenue` -> `starcharger/api/v2/bill_revenue`).
    pub endpoints: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct TomlConfig {
    endpoints: HashMap<String, String>,
}

impl Config {
    /// Load configuration from environment variables and config.toml.
    pub fn from_env() -> Self {
        let api_base = std::env::var("DATACENTER_API_BASE")
            .unwrap_or_else(|_| panic!("DATACENTER_API_BASE environment variable must be set"));

        let toml_str = std::fs::read_to_string("config.toml")
            .unwrap_or_else(|e| panic!("Failed to read config.toml: {}", e));
        let toml_config: TomlConfig = toml::from_str(&toml_str)
            .unwrap_or_else(|e| panic!("Failed to parse config.toml: {}", e));

        Self {
            api_base,
            endpoints: toml_config.endpoints,
        }
    }

    /// Resolve an endpoint slug to its full path.
    pub fn resolve_endpoint(&self, slug: &str) -> Result<&str, String> {
        self.endpoints
            .get(slug)
            .map(|s| s.as_str())
            .ok_or_else(|| format!("unknown endpoint slug: {}", slug))
    }
}
