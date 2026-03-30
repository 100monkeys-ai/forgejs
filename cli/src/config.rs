//! CLI configuration: reads `~/.forge/config.toml` for global settings.
//!
//! Global settings include:
//! - The default Foundry registry URL (defaults to https://registry.forgejs.com)
//! - Authentication tokens for the Foundry registry
//! - Preferred editor for `forge studio --open`

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

/// Global Forge CLI configuration, read from `~/.forge/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliConfig {
    /// URL of the Foundry registry (default: https://registry.forgejs.com)
    #[serde(default)]
    pub registry_url: Option<String>,
    /// Authentication token for the Foundry registry
    #[serde(default)]
    pub auth_token: Option<String>,
}

impl CliConfig {
    /// Load the CLI config from `~/.forge/config.toml`.
    /// Returns a default config if the file does not exist.
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() {
            return Self::default();
        }
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    }

    /// Returns the path to the CLI config file.
    pub fn path() -> Utf8PathBuf {
        config_path()
    }
}

fn config_path() -> Utf8PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    Utf8PathBuf::from(home).join(".forge").join("config.toml")
}
