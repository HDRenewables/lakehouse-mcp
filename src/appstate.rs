//! Application state module.
//!
//! Holds the shared state, currently:
//! - configuration, this is immutable and shared across threads.

use std::sync::Arc;

use crate::config::Config;

/// Shared application state.
#[derive(Debug, Clone)]
pub struct AppState {
    /// Thread-safe reference to the shared configuration.
    pub config: Arc<Config>,
}

impl AppState {
    /// Create a new AppState wrapping the config.
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}
