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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_list_full() {
        let json = serde_json::json!({
            "id": "l1",
            "name": "Backlog",
            "orderindex": 1,
            "status": {
                "status": "Open",
                "color": "#d3d3d3",
                "type": "open"
            },
            "content": "All backlog items",
            "task_count": "15",
            "space": { "id": "s1" },
            "folder": { "id": "f1", "name": "Sprint 1", "hidden": false }
        });

        let list: List = serde_json::from_value(json).expect("deserialize list");
        assert_eq!(list.id, "l1");
        assert_eq!(list.name, "Backlog");
        assert_eq!(list.orderindex, 1);
        let status = list.status.expect("status should be present");
        assert_eq!(status.status, "Open");
        assert_eq!(list.content.as_deref(), Some("All backlog items"));
        assert_eq!(list.task_count.as_deref(), Some("15"));
        assert_eq!(list.space.id, "s1");
        assert_eq!(list.folder.id, "f1");
        assert_eq!(list.folder.name.as_deref(), Some("Sprint 1"));
    }

    #[test]
    fn test_deserialize_list_minimal() {
        let json = serde_json::json!({
            "id": "l2",
            "name": "Empty"
        });

        let list: List = serde_json::from_value(json).expect("deserialize minimal list");
        assert_eq!(list.id, "l2");
        assert_eq!(list.name, "Empty");
        assert_eq!(list.orderindex, 0);
        assert!(list.status.is_none());
        assert!(list.content.is_none());
        assert!(list.task_count.is_none());
        assert_eq!(list.space.id, "");
        assert_eq!(list.folder.id, "");
        assert!(list.folder.name.is_none());
    }

    #[test]
    fn test_deserialize_list_numeric_ids() {
        let json = serde_json::json!({
            "id": 500,
            "name": "Numeric",
            "orderindex": "3",
            "task_count": 10,
            "space": { "id": 100 },
            "folder": { "id": 200 }
        });

        let list: List = serde_json::from_value(json).expect("deserialize list with numeric ids");
        assert_eq!(list.id, "500");
        assert_eq!(list.orderindex, 3);
        assert_eq!(list.task_count.as_deref(), Some("10"));
        assert_eq!(list.space.id, "100");
        assert_eq!(list.folder.id, "200");
    }

    #[test]
    fn test_deserialize_lists_response() {
        let json = serde_json::json!({
            "lists": [
                { "id": "l1", "name": "First" },
                { "id": "l2", "name": "Second" }
            ]
        });

        let resp: ListsResponse = serde_json::from_value(json).expect("deserialize lists response");
        assert_eq!(resp.lists.len(), 2);
        assert_eq!(resp.lists[0].name, "First");
        assert_eq!(resp.lists[1].name, "Second");
    }
}
