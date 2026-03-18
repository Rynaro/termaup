use serde::{Deserialize, Serialize};

use super::list::List;
use crate::serde_helpers::{
    deserialize_i32_or_string, deserialize_option_string_or_number, deserialize_string_or_number,
};

/// A ClickUp folder within a space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    /// Folder ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Folder name.
    pub name: String,
    /// Display order index.
    #[serde(default, deserialize_with = "deserialize_i32_or_string")]
    pub orderindex: i32,
    /// Whether the folder is hidden.
    #[serde(default)]
    pub hidden: bool,
    /// The parent space.
    pub space: FolderSpace,
    /// Number of tasks in the folder (returned as a string by the API).
    #[serde(default, deserialize_with = "deserialize_option_string_or_number")]
    pub task_count: Option<String>,
    /// Lists inside this folder.
    #[serde(default)]
    pub lists: Vec<List>,
}

/// Minimal space reference embedded in a folder.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderSpace {
    /// Space ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Space name (may be absent).
    #[serde(default)]
    pub name: Option<String>,
}

/// Response wrapper returned by `GET /space/{id}/folder`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldersResponse {
    /// List of folders.
    pub folders: Vec<Folder>,
}
