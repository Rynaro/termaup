use serde::{Deserialize, Serialize};

use super::user::User;
use crate::serde_helpers::deserialize_string_or_number;

/// A checklist attached to a ClickUp task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checklist {
    /// Checklist ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Checklist name.
    pub name: String,
    /// Whether the entire checklist is resolved.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_option_bool_or_int"
    )]
    pub resolved: Option<bool>,
    /// Items in the checklist.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_null_as_default"
    )]
    pub items: Vec<ChecklistItem>,
    /// Display order.
    #[serde(default)]
    pub orderindex: Option<i32>,
    /// Number of items in the checklist.
    #[serde(default)]
    pub task_count: Option<i32>,
    /// Number of unresolved items.
    #[serde(default)]
    pub unresolved: Option<i32>,
}

/// An individual item within a checklist.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    /// Item ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Item name / description.
    pub name: String,
    /// Whether this item is resolved (checked).
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_bool_or_int"
    )]
    pub resolved: bool,
    /// User assigned to this item, if any.
    #[serde(default)]
    pub assignee: Option<User>,
    /// Display order.
    #[serde(default)]
    pub orderindex: Option<i32>,
    /// Parent item ID (for nested checklist items).
    #[serde(default)]
    pub parent: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_checklist_with_items() {
        let json = serde_json::json!({
            "id": "checklist_1",
            "name": "QA Checklist",
            "resolved": false,
            "orderindex": 0,
            "task_count": 3,
            "unresolved": 2,
            "items": [
                {
                    "id": "item_1",
                    "name": "Write unit tests",
                    "resolved": 1,
                    "assignee": {
                        "id": 42,
                        "username": "alice",
                        "email": "alice@example.com"
                    },
                    "orderindex": 0,
                    "parent": null
                },
                {
                    "id": 200,
                    "name": "Run integration tests",
                    "resolved": 0,
                    "assignee": null,
                    "orderindex": 1,
                    "parent": null
                }
            ]
        });

        let checklist: Checklist = serde_json::from_value(json).expect("deserialize checklist");

        assert_eq!(checklist.id, "checklist_1");
        assert_eq!(checklist.name, "QA Checklist");
        assert_eq!(checklist.resolved, Some(false));
        assert_eq!(checklist.items.len(), 2);
        assert_eq!(checklist.task_count, Some(3));
        assert_eq!(checklist.unresolved, Some(2));

        let first = &checklist.items[0];
        assert_eq!(first.id, "item_1");
        assert_eq!(first.name, "Write unit tests");
        assert!(first.resolved);
        assert!(first.assignee.is_some());
        assert_eq!(first.assignee.as_ref().unwrap().username, "alice");

        let second = &checklist.items[1];
        assert_eq!(second.id, "200", "numeric ID should deserialize as string");
        assert!(!second.resolved);
        assert!(second.assignee.is_none());
    }

    #[test]
    fn test_deserialize_checklist_empty_items() {
        let json = serde_json::json!({
            "id": 999,
            "name": "Empty Checklist"
        });

        let checklist: Checklist =
            serde_json::from_value(json).expect("deserialize empty checklist");

        assert_eq!(checklist.id, "999");
        assert_eq!(checklist.name, "Empty Checklist");
        assert!(checklist.items.is_empty());
        assert!(checklist.resolved.is_none());
        assert!(checklist.orderindex.is_none());
    }

    #[test]
    fn test_deserialize_checklist_with_nested_items() {
        let json = serde_json::json!({
            "id": "cl_nested",
            "name": "Nested Checklist",
            "items": [
                {
                    "id": "parent_item",
                    "name": "Parent task",
                    "resolved": false,
                    "parent": null,
                    "orderindex": 0
                },
                {
                    "id": "child_item",
                    "name": "Sub-task of parent",
                    "resolved": true,
                    "parent": "parent_item",
                    "orderindex": 0
                }
            ]
        });

        let checklist: Checklist =
            serde_json::from_value(json).expect("deserialize nested checklist");

        assert_eq!(checklist.items.len(), 2);

        let parent = &checklist.items[0];
        assert!(parent.parent.is_none());

        let child = &checklist.items[1];
        assert_eq!(
            child.parent.as_deref(),
            Some("parent_item"),
            "child should reference parent item"
        );
        assert!(child.resolved);
    }
}
