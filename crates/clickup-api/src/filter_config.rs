use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::error::{ClickUpError, Result};

/// Persistent filter settings for a single list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ListFilterConfig {
    /// Status names to include (empty = all).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub statuses: Vec<String>,

    /// Assignee user IDs to include (empty = all).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assignees: Vec<String>,

    /// Priority labels to include (empty = all).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub priorities: Vec<String>,

    /// Tag names to include (empty = all).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Due-date filter preset.
    #[serde(default)]
    pub due_date_filter: DueDateFilter,

    /// Whether "Me Mode" is active (show only tasks assigned to current user).
    #[serde(default)]
    pub me_mode: bool,

    /// Preferred view mode for this list.
    #[serde(default)]
    pub view_mode: ViewModeConfig,

    /// Whether to include closed tasks.
    #[serde(default)]
    pub include_closed: bool,
}

/// Due-date filter presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DueDateFilter {
    /// No date filtering.
    #[default]
    All,
    /// Tasks past their due date.
    Overdue,
    /// Tasks due today.
    Today,
    /// Tasks due this week (next 7 days).
    ThisWeek,
    /// Tasks with no due date set.
    NoDueDate,
}

/// View mode preference stored per list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ViewModeConfig {
    /// Standard flat task list.
    #[default]
    List,
    /// Vertical sections grouped by status.
    VisionSections,
    /// Horizontal kanban board grouped by status.
    VisionBoard,
}

/// Top-level structure for the filters TOML file.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct FilterFile {
    /// Per-list filter configurations, keyed by list ID.
    #[serde(default)]
    filters: HashMap<String, ListFilterConfig>,
}

/// Manages persistent filter storage at `~/.config/clickup-rs/filters.toml`.
pub struct FilterStore;

impl FilterStore {
    /// Returns the path to the filters file.
    pub fn filters_file_path() -> PathBuf {
        Config::config_dir().join("filters.toml")
    }

    /// Loads filter configuration for a specific list.
    ///
    /// Returns `None` if no filters are saved for this list.
    pub fn load(list_id: &str) -> Result<Option<ListFilterConfig>> {
        let file = Self::load_file()?;
        Ok(file.filters.get(list_id).cloned())
    }

    /// Saves filter configuration for a specific list.
    pub fn save(list_id: &str, config: &ListFilterConfig) -> Result<()> {
        let mut file = Self::load_file()?;
        file.filters.insert(list_id.to_string(), config.clone());
        Self::write_file(&file)
    }

    /// Removes saved filters for a specific list.
    pub fn remove(list_id: &str) -> Result<()> {
        let mut file = Self::load_file()?;
        file.filters.remove(list_id);
        Self::write_file(&file)
    }

    fn load_file() -> Result<FilterFile> {
        let path = Self::filters_file_path();
        if !path.exists() {
            return Ok(FilterFile::default());
        }

        let contents = std::fs::read_to_string(&path)
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to read filters file: {e}")))?;

        toml::from_str(&contents)
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to parse filters file: {e}")))
    }

    fn write_file(file: &FilterFile) -> Result<()> {
        let dir = Config::config_dir();
        std::fs::create_dir_all(&dir).map_err(|e| {
            ClickUpError::ConfigError(format!("Failed to create config directory: {e}"))
        })?;

        let contents = toml::to_string_pretty(file)
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to serialize filters: {e}")))?;

        let path = Self::filters_file_path();
        std::fs::write(&path, contents)
            .map_err(|e| ClickUpError::ConfigError(format!("Failed to write filters file: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_filter_config_default_is_empty() {
        let config = ListFilterConfig::default();
        assert!(config.statuses.is_empty());
        assert!(config.assignees.is_empty());
        assert!(config.priorities.is_empty());
        assert!(config.tags.is_empty());
        assert_eq!(config.due_date_filter, DueDateFilter::All);
        assert!(!config.me_mode);
        assert_eq!(config.view_mode, ViewModeConfig::List);
        assert!(!config.include_closed);
    }

    #[test]
    fn test_list_filter_config_serialization_roundtrip() {
        let config = ListFilterConfig {
            statuses: vec!["in progress".into(), "review".into()],
            assignees: vec!["12345".into()],
            priorities: vec!["high".into(), "urgent".into()],
            tags: vec!["frontend".into()],
            due_date_filter: DueDateFilter::ThisWeek,
            me_mode: true,
            view_mode: ViewModeConfig::VisionSections,
            include_closed: false,
        };

        let toml_str = toml::to_string_pretty(&config).expect("serialize");
        let deserialized: ListFilterConfig = toml::from_str(&toml_str).expect("deserialize");
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_filter_file_serialization() {
        let mut file = FilterFile::default();
        file.filters.insert(
            "list_abc".to_string(),
            ListFilterConfig {
                me_mode: true,
                ..Default::default()
            },
        );

        let toml_str = toml::to_string_pretty(&file).expect("serialize");
        assert!(toml_str.contains("[filters.list_abc]"));
        assert!(toml_str.contains("me_mode = true"));

        let deserialized: FilterFile = toml::from_str(&toml_str).expect("deserialize");
        assert!(deserialized.filters.contains_key("list_abc"));
        assert!(deserialized.filters["list_abc"].me_mode);
    }

    #[test]
    fn test_due_date_filter_variants() {
        let variants = [
            (DueDateFilter::All, "\"all\""),
            (DueDateFilter::Overdue, "\"overdue\""),
            (DueDateFilter::Today, "\"today\""),
            (DueDateFilter::ThisWeek, "\"this_week\""),
            (DueDateFilter::NoDueDate, "\"no_due_date\""),
        ];
        for (variant, expected) in variants {
            let json = serde_json::to_string(&variant).expect("serialize");
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_view_mode_config_variants() {
        let variants = [
            (ViewModeConfig::List, "\"list\""),
            (ViewModeConfig::VisionSections, "\"vision_sections\""),
            (ViewModeConfig::VisionBoard, "\"vision_board\""),
        ];
        for (variant, expected) in variants {
            let json = serde_json::to_string(&variant).expect("serialize");
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_deserialize_minimal_filter_config() {
        let toml_str = r#"
            me_mode = true
        "#;
        let config: ListFilterConfig = toml::from_str(toml_str).expect("deserialize");
        assert!(config.me_mode);
        assert!(config.statuses.is_empty());
        assert_eq!(config.view_mode, ViewModeConfig::List);
    }
}
