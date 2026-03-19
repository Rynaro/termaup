use serde::{Deserialize, Serialize};

use crate::serde_helpers::deserialize_string_or_number;

/// A task linked to the current task.
///
/// Linked tasks represent free-form relationships between tasks in ClickUp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedTask {
    /// The linked task's ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub task_id: String,
    /// The link relationship ID.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_option_string_or_number"
    )]
    pub link_id: Option<String>,
    /// Timestamp when the link was created (milliseconds).
    #[serde(default)]
    pub date_created: Option<String>,
    /// ID of the user who created the link.
    #[serde(default)]
    pub userid: Option<String>,
    /// Workspace ID where the linked task resides.
    #[serde(default)]
    pub workspace_id: Option<String>,
}

/// A dependency relationship between tasks.
///
/// Dependencies indicate that one task blocks or is blocked by another.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskDependency {
    /// The task that has this dependency.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub task_id: String,
    /// The task being depended upon.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub depends_on: String,
    /// Dependency type (1 = waiting on, 0 = blocking).
    #[serde(default, rename = "type")]
    pub dependency_type: Option<i32>,
    /// Timestamp when the dependency was created (milliseconds).
    #[serde(default)]
    pub date_created: Option<String>,
    /// ID of the user who created the dependency.
    #[serde(default)]
    pub userid: Option<String>,
    /// Workspace ID.
    #[serde(default)]
    pub workspace_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_linked_task_full() {
        let json = serde_json::json!({
            "task_id": "abc123",
            "link_id": "link_1",
            "date_created": "1710000000000",
            "userid": "user_1",
            "workspace_id": "ws_1"
        });

        let linked: LinkedTask = serde_json::from_value(json).expect("deserialize linked task");
        assert_eq!(linked.task_id, "abc123");
        assert_eq!(linked.link_id.as_deref(), Some("link_1"));
        assert_eq!(linked.date_created.as_deref(), Some("1710000000000"));
        assert_eq!(linked.userid.as_deref(), Some("user_1"));
        assert_eq!(linked.workspace_id.as_deref(), Some("ws_1"));
    }

    #[test]
    fn test_deserialize_linked_task_minimal() {
        let json = serde_json::json!({
            "task_id": "abc123"
        });

        let linked: LinkedTask =
            serde_json::from_value(json).expect("deserialize minimal linked task");
        assert_eq!(linked.task_id, "abc123");
        assert!(linked.link_id.is_none());
        assert!(linked.date_created.is_none());
        assert!(linked.userid.is_none());
        assert!(linked.workspace_id.is_none());
    }

    #[test]
    fn test_deserialize_linked_task_numeric_ids() {
        let json = serde_json::json!({
            "task_id": 99999,
            "link_id": 42
        });

        let linked: LinkedTask = serde_json::from_value(json).expect("deserialize numeric ids");
        assert_eq!(linked.task_id, "99999");
        assert_eq!(linked.link_id.as_deref(), Some("42"));
    }

    #[test]
    fn test_deserialize_task_dependency_full() {
        let json = serde_json::json!({
            "task_id": "task_a",
            "depends_on": "task_b",
            "type": 1,
            "date_created": "1710000000000",
            "userid": "user_1",
            "workspace_id": "ws_1"
        });

        let dep: TaskDependency =
            serde_json::from_value(json).expect("deserialize task dependency");
        assert_eq!(dep.task_id, "task_a");
        assert_eq!(dep.depends_on, "task_b");
        assert_eq!(dep.dependency_type, Some(1));
        assert_eq!(dep.date_created.as_deref(), Some("1710000000000"));
        assert_eq!(dep.userid.as_deref(), Some("user_1"));
        assert_eq!(dep.workspace_id.as_deref(), Some("ws_1"));
    }

    #[test]
    fn test_deserialize_task_dependency_minimal() {
        let json = serde_json::json!({
            "task_id": "task_a",
            "depends_on": "task_b"
        });

        let dep: TaskDependency =
            serde_json::from_value(json).expect("deserialize minimal dependency");
        assert_eq!(dep.task_id, "task_a");
        assert_eq!(dep.depends_on, "task_b");
        assert!(dep.dependency_type.is_none());
        assert!(dep.date_created.is_none());
    }

    #[test]
    fn test_deserialize_task_dependency_numeric_ids() {
        let json = serde_json::json!({
            "task_id": 100,
            "depends_on": 200,
            "type": 0
        });

        let dep: TaskDependency =
            serde_json::from_value(json).expect("deserialize numeric dependency");
        assert_eq!(dep.task_id, "100");
        assert_eq!(dep.depends_on, "200");
        assert_eq!(dep.dependency_type, Some(0));
    }
}
