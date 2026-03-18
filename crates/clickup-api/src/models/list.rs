use serde::{Deserialize, Serialize};

use super::status::Status;
use crate::serde_helpers::{
    deserialize_i32_or_string, deserialize_option_string_or_number, deserialize_string_or_number,
};

/// A ClickUp list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct List {
    /// List ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// List name.
    pub name: String,
    /// Display order index.
    #[serde(default, deserialize_with = "deserialize_i32_or_string")]
    pub orderindex: i32,
    /// Current list status.
    #[serde(default)]
    pub status: Option<Status>,
    /// List description / content.
    #[serde(default)]
    pub content: Option<String>,
    /// Number of tasks (returned as a string by the API).
    #[serde(default, deserialize_with = "deserialize_option_string_or_number")]
    pub task_count: Option<String>,
    /// The parent space.
    #[serde(default)]
    pub space: ListSpace,
    /// The parent folder.
    #[serde(default)]
    pub folder: ListFolder,
}

/// Minimal space reference embedded in a list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListSpace {
    /// Space ID.
    #[serde(default, deserialize_with = "deserialize_string_or_number")]
    pub id: String,
}

/// Minimal folder reference embedded in a list.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListFolder {
    /// Folder ID.
    #[serde(default, deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Folder name (may be absent).
    #[serde(default)]
    pub name: Option<String>,
    /// Whether the folder is hidden.
    #[serde(default)]
    pub hidden: Option<bool>,
}

/// Response wrapper returned by `GET /folder/{id}/list` and
/// `GET /space/{id}/list`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListsResponse {
    /// List of lists.
    pub lists: Vec<List>,
}
