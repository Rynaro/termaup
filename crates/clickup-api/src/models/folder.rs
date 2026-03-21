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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_folder_full() {
        let json = serde_json::json!({
            "id": "f1",
            "name": "Sprint 1",
            "orderindex": 0,
            "hidden": false,
            "space": { "id": "s1", "name": "Engineering" },
            "task_count": "42",
            "lists": [
                {
                    "id": "l1",
                    "name": "Backlog"
                }
            ]
        });

        let folder: Folder = serde_json::from_value(json).expect("deserialize folder");
        assert_eq!(folder.id, "f1");
        assert_eq!(folder.name, "Sprint 1");
        assert_eq!(folder.orderindex, 0);
        assert!(!folder.hidden);
        assert_eq!(folder.space.id, "s1");
        assert_eq!(folder.space.name.as_deref(), Some("Engineering"));
        assert_eq!(folder.task_count.as_deref(), Some("42"));
        assert_eq!(folder.lists.len(), 1);
        assert_eq!(folder.lists[0].name, "Backlog");
    }

    #[test]
    fn test_deserialize_folder_minimal() {
        let json = serde_json::json!({
            "id": "f2",
            "name": "Empty",
            "space": { "id": "s1" }
        });

        let folder: Folder = serde_json::from_value(json).expect("deserialize minimal folder");
        assert_eq!(folder.id, "f2");
        assert_eq!(folder.name, "Empty");
        assert_eq!(folder.orderindex, 0);
        assert!(!folder.hidden);
        assert!(folder.space.name.is_none());
        assert!(folder.task_count.is_none());
        assert!(folder.lists.is_empty());
    }

    #[test]
    fn test_deserialize_folder_numeric_ids() {
        let json = serde_json::json!({
            "id": 100,
            "name": "Numeric",
            "space": { "id": 200 },
            "task_count": 5
        });

        let folder: Folder =
            serde_json::from_value(json).expect("deserialize folder with numeric ids");
        assert_eq!(folder.id, "100");
        assert_eq!(folder.space.id, "200");
        assert_eq!(folder.task_count.as_deref(), Some("5"));
    }

    #[test]
    fn test_deserialize_folders_response() {
        let json = serde_json::json!({
            "folders": [
                { "id": "f1", "name": "A", "space": { "id": "s1" } },
                { "id": "f2", "name": "B", "space": { "id": "s1" } }
            ]
        });

        let resp: FoldersResponse =
            serde_json::from_value(json).expect("deserialize folders response");
        assert_eq!(resp.folders.len(), 2);
        assert_eq!(resp.folders[0].name, "A");
        assert_eq!(resp.folders[1].name, "B");
    }
}
