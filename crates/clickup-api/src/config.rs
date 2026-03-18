use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{ClickUpError, Result};

/// Output format for CLI command results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// Render output as a table (default).
    #[default]
    Table,
    /// Render output as JSON.
    Json,
    /// Render output as Markdown.
    Markdown,
}

/// Application configuration persisted to `config.toml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The default workspace (team) ID to use when none is specified.
    #[serde(default)]
    pub default_workspace_id: Option<String>,

    /// The default space ID to use when none is specified.
    #[serde(default)]
    pub default_space_id: Option<String>,

    /// Base URL for the ClickUp API.
    #[serde(default = "default_api_base_url")]
    pub api_base_url: String,

    /// Default output format for CLI commands.
    #[serde(default)]
    pub output_format: OutputFormat,
}

fn default_api_base_url() -> String {
    "https://api.clickup.com/api/v2".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_workspace_id: None,
            default_space_id: None,
            api_base_url: default_api_base_url(),
            output_format: OutputFormat::default(),
        }
    }
}

impl Config {
    /// Returns the configuration directory (`~/.config/clickup-rs/`).
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clickup-rs")
    }

    /// Returns the path to the configuration file.
    pub fn config_file_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Loads configuration from disk. Returns defaults if the file does not exist.
    pub fn load() -> Result<Self> {
        let path = Self::config_file_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&path)
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to read config file: {e}")))?;

        toml::from_str(&contents)
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to parse config file: {e}")))
    }

    /// Persists this configuration to disk, creating directories as needed.
    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| {
            ClickUpError::ConfigError(format!("Failed to create config directory: {e}"))
        })?;

        let contents = toml::to_string_pretty(self)
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to serialize config: {e}")))?;

        let path = Self::config_file_path();
        std::fs::write(&path, contents)
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to write config file: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let config = Config::default();
        assert_eq!(config.default_workspace_id, None);
        assert_eq!(config.default_space_id, None);
        assert_eq!(config.api_base_url, "https://api.clickup.com/api/v2");
        assert_eq!(config.output_format, OutputFormat::Table);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config {
            default_workspace_id: Some("team_123".into()),
            default_space_id: Some("space_456".into()),
            api_base_url: "https://api.clickup.com/api/v2".into(),
            output_format: OutputFormat::Json,
        };

        let toml_str = toml::to_string_pretty(&config).expect("serialize");
        let deserialized: Config = toml::from_str(&toml_str).expect("deserialize");

        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_deserialize_missing_optional_fields() {
        let toml_str = r#"
            api_base_url = "https://api.clickup.com/api/v2"
        "#;

        let config: Config = toml::from_str(toml_str).expect("deserialize");
        assert_eq!(config.default_workspace_id, None);
        assert_eq!(config.default_space_id, None);
        assert_eq!(config.output_format, OutputFormat::Table);
    }

    #[test]
    fn test_output_format_default_is_table() {
        assert_eq!(OutputFormat::default(), OutputFormat::Table);
    }
}
