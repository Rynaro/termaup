use serde::{Deserialize, Serialize};

use super::user::User;
use crate::serde_helpers::{deserialize_option_string_or_number, deserialize_string_or_number};

/// A ClickUp task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Task ID (alphanumeric).
    pub id: String,
    /// Custom (human-readable) task ID, if enabled.
    #[serde(default)]
    pub custom_id: Option<String>,
    /// Task name / title.
    pub name: String,
    /// Plain-text description.
    #[serde(default)]
    pub description: Option<String>,
    /// Raw text content (without formatting).
    #[serde(default)]
    pub text_content: Option<String>,
    /// Current status.
    pub status: TaskStatus,
    /// Display order.
    #[serde(default)]
    pub orderindex: String,
    /// Creation timestamp (milliseconds).
    #[serde(default)]
    pub date_created: String,
    /// Last update timestamp (milliseconds).
    #[serde(default)]
    pub date_updated: String,
    /// Closed timestamp (milliseconds), if closed.
    #[serde(default)]
    pub date_closed: Option<String>,
    /// Done timestamp (milliseconds), if done.
    #[serde(default)]
    pub date_done: Option<String>,
    /// The user who created the task.
    pub creator: User,
    /// Assigned users.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_null_as_default"
    )]
    pub assignees: Vec<User>,
    /// Task priority.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_maybe_false"
    )]
    pub priority: Option<TaskPriority>,
    /// Due-date timestamp (milliseconds).
    #[serde(default)]
    pub due_date: Option<String>,
    /// Start-date timestamp (milliseconds).
    #[serde(default)]
    pub start_date: Option<String>,
    /// Tags attached to the task.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_null_as_default"
    )]
    pub tags: Vec<Tag>,
    /// Parent list reference.
    pub list: TaskList,
    /// Parent folder reference.
    pub folder: TaskFolder,
    /// Parent space reference.
    pub space: TaskSpace,
    /// Web URL for the task.
    #[serde(default)]
    pub url: String,
    /// Markdown-formatted description (requires `include_markdown_description`).
    #[serde(default)]
    pub markdown_description: Option<String>,
    /// Parent task ID (for subtasks).
    #[serde(default)]
    pub parent: Option<String>,
    /// Subtasks (requires `include_subtasks`).
    #[serde(default)]
    pub subtasks: Option<Vec<Task>>,
    /// Custom field values.
    #[serde(default)]
    pub custom_fields: Option<Vec<CustomField>>,
    /// Checklists attached to the task.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_null_as_default"
    )]
    pub checklists: Vec<super::checklist::Checklist>,
    /// Tasks linked to this task.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_null_as_default"
    )]
    pub linked_tasks: Vec<super::linked_task::LinkedTask>,
    /// Task dependencies (blocking/waiting relationships).
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_null_as_default"
    )]
    pub dependencies: Vec<super::linked_task::TaskDependency>,
    /// Estimated time for this task (milliseconds).
    #[serde(default)]
    pub time_estimate: Option<u64>,
    /// Time spent on this task (milliseconds). May be a number or `{"time": ms}`.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_time_value"
    )]
    pub time_spent: Option<u64>,
    /// Users watching this task.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_null_as_default"
    )]
    pub watchers: Vec<User>,
    /// File attachments on this task.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_null_as_default"
    )]
    pub attachments: Vec<Attachment>,
    /// Story points assigned to this task.
    #[serde(default)]
    pub points: Option<serde_json::Value>,
    /// Permission level of the current user on this task.
    #[serde(default)]
    pub permission_level: Option<String>,
}

/// A file attachment on a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment ID.
    pub id: String,
    /// File title / name.
    #[serde(default)]
    pub title: Option<String>,
    /// URL to download the attachment.
    #[serde(default)]
    pub url: Option<String>,
    /// File extension (e.g. "pdf", "png").
    #[serde(default)]
    pub extension: Option<String>,
    /// Upload timestamp (milliseconds).
    #[serde(default)]
    pub date: Option<String>,
    /// Thumbnail URL for image attachments.
    #[serde(default)]
    pub thumbnail_small: Option<String>,
}

/// Inline status on a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatus {
    /// Status name.
    #[serde(default)]
    pub status: String,
    /// Hex colour.
    #[serde(default)]
    pub color: String,
    /// Status type (e.g. "open", "closed", "custom").
    #[serde(default, rename = "type")]
    pub status_type: String,
}

/// Task priority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskPriority {
    /// Priority ID.
    #[serde(default, deserialize_with = "deserialize_option_string_or_number")]
    pub id: Option<String>,
    /// Priority label (e.g. "urgent", "high").
    #[serde(default)]
    pub priority: Option<String>,
    /// Hex colour.
    #[serde(default)]
    pub color: Option<String>,
}

/// A tag on a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    /// Tag name.
    pub name: String,
    /// Foreground colour.
    #[serde(default)]
    pub tag_fg: Option<String>,
    /// Background colour.
    #[serde(default)]
    pub tag_bg: Option<String>,
}

/// Minimal list reference embedded in a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskList {
    /// List ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// List name.
    #[serde(default)]
    pub name: Option<String>,
}

/// Minimal folder reference embedded in a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFolder {
    /// Folder ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Folder name.
    #[serde(default)]
    pub name: Option<String>,
}

/// Minimal space reference embedded in a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpace {
    /// Space ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
}

/// A custom field value on a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    /// Field ID.
    pub id: String,
    /// Field display name.
    pub name: String,
    /// Field type (e.g. "text", "number", "dropdown").
    #[serde(rename = "type")]
    pub field_type: String,
    /// Current value (shape varies by field type).
    #[serde(default)]
    pub value: Option<serde_json::Value>,
}

/// Response wrapper returned by `GET /list/{id}/task`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TasksResponse {
    /// Tasks on this page.
    pub tasks: Vec<Task>,
    /// Whether this is the last page of results.
    #[serde(default)]
    pub last_page: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_task_json() -> serde_json::Value {
        serde_json::json!({
            "id": "abc123",
            "custom_id": "PROJ-42",
            "name": "Implement login",
            "description": "Build the login page",
            "text_content": "Build the login page",
            "status": {
                "status": "in progress",
                "color": "#4194f6",
                "type": "custom"
            },
            "orderindex": "1.00",
            "date_created": "1710000000000",
            "date_updated": "1710100000000",
            "date_closed": null,
            "date_done": null,
            "creator": {
                "id": 1001,
                "username": "alice",
                "email": "alice@example.com",
                "color": "#ff0000",
                "profilePicture": null,
                "initials": "A"
            },
            "assignees": [
                {
                    "id": 1002,
                    "username": "bob",
                    "email": "bob@example.com",
                    "color": null,
                    "profilePicture": "https://example.com/bob.png",
                    "initials": "B"
                }
            ],
            "priority": {
                "id": "2",
                "priority": "high",
                "color": "#ffcc00"
            },
            "due_date": "1711000000000",
            "start_date": "1710000000000",
            "tags": [
                { "name": "frontend", "tag_fg": "#fff", "tag_bg": "#000" }
            ],
            "list": { "id": "list_1", "name": "Sprint 1" },
            "folder": { "id": "fold_1", "name": "Product" },
            "space": { "id": "space_1" },
            "url": "https://app.clickup.com/t/abc123",
            "markdown_description": "# Login\nBuild it.",
            "parent": null,
            "subtasks": [],
            "custom_fields": [
                {
                    "id": "cf_1",
                    "name": "Story Points",
                    "type": "number",
                    "value": 5
                }
            ]
        })
    }

    #[test]
    fn test_deserialize_task_full() {
        let task: Task = serde_json::from_value(sample_task_json()).expect("deserialize task");

        assert_eq!(task.id, "abc123");
        assert_eq!(task.custom_id.as_deref(), Some("PROJ-42"));
        assert_eq!(task.name, "Implement login");
        assert_eq!(task.status.status, "in progress");
        assert_eq!(task.status.status_type, "custom");
        assert_eq!(task.creator.username, "alice");
        assert_eq!(task.assignees.len(), 1);
        assert_eq!(task.assignees[0].username, "bob");

        let priority = task.priority.as_ref().unwrap();
        assert_eq!(priority.priority.as_deref(), Some("high"));

        assert_eq!(task.tags.len(), 1);
        assert_eq!(task.tags[0].name, "frontend");
        assert_eq!(task.list.id, "list_1");
        assert_eq!(task.folder.name.as_deref(), Some("Product"));
        assert_eq!(task.space.id, "space_1");
        assert_eq!(
            task.markdown_description.as_deref(),
            Some("# Login\nBuild it.")
        );

        let cf = task.custom_fields.as_ref().unwrap();
        assert_eq!(cf.len(), 1);
        assert_eq!(cf[0].name, "Story Points");
        assert_eq!(cf[0].field_type, "number");
    }

    #[test]
    fn test_deserialize_task_with_time_and_attachments() {
        let json = serde_json::json!({
            "id": "t2",
            "name": "Tracked task",
            "status": {
                "status": "open",
                "color": "#ccc",
                "type": "open"
            },
            "creator": {
                "id": 1,
                "username": "u",
                "email": "u@x.com"
            },
            "list": { "id": "l1" },
            "folder": { "id": "f1" },
            "space": { "id": "s1" },
            "time_estimate": 3600000,
            "time_spent": {"time": 1200000},
            "watchers": [
                {
                    "id": 1003,
                    "username": "watcher1",
                    "email": "w@x.com"
                }
            ],
            "attachments": [
                {
                    "id": "att_1",
                    "title": "design.png",
                    "url": "https://example.com/design.png",
                    "extension": "png",
                    "date": "1710000000000",
                    "thumbnail_small": "https://example.com/thumb.png"
                }
            ],
            "points": 8,
            "permission_level": "create"
        });

        let task: Task = serde_json::from_value(json).expect("deserialize task");
        assert_eq!(task.time_estimate, Some(3600000));
        assert_eq!(task.time_spent, Some(1200000));
        assert_eq!(task.watchers.len(), 1);
        assert_eq!(task.watchers[0].username, "watcher1");
        assert_eq!(task.attachments.len(), 1);
        assert_eq!(task.attachments[0].id, "att_1");
        assert_eq!(task.attachments[0].title.as_deref(), Some("design.png"));
        assert_eq!(task.points.unwrap(), serde_json::json!(8));
        assert_eq!(task.permission_level.as_deref(), Some("create"));
    }

    #[test]
    fn test_deserialize_task_minimal() {
        let json = serde_json::json!({
            "id": "t1",
            "name": "Minimal task",
            "status": { "status": "open", "color": "#ccc", "type": "open" },
            "creator": {
                "id": 1,
                "username": "u",
                "email": "u@x.com"
            },
            "list": { "id": "l1" },
            "folder": { "id": "f1" },
            "space": { "id": "s1" }
        });

        let task: Task = serde_json::from_value(json).expect("deserialize minimal task");
        assert_eq!(task.id, "t1");
        assert!(task.custom_id.is_none());
        assert!(task.assignees.is_empty());
        assert!(task.tags.is_empty());
        assert!(task.priority.is_none());
    }

    #[test]
    fn test_deserialize_task_priority_false() {
        let json = serde_json::json!({
            "id": "t2",
            "name": "No priority task",
            "status": { "status": "open", "color": "#ccc", "type": "open" },
            "creator": {
                "id": 1,
                "username": "u",
                "email": "u@x.com"
            },
            "priority": false,
            "list": { "id": "l1" },
            "folder": { "id": "f1" },
            "space": { "id": "s1" }
        });

        let task: Task = serde_json::from_value(json).expect("priority:false should deserialize");
        assert!(task.priority.is_none(), "priority false should map to None");
    }

    #[test]
    fn test_deserialize_tasks_response() {
        let json = serde_json::json!({
            "tasks": [sample_task_json()],
            "last_page": true
        });

        let resp: TasksResponse = serde_json::from_value(json).expect("deserialize response");
        assert_eq!(resp.tasks.len(), 1);
        assert_eq!(resp.last_page, Some(true));
    }

    #[test]
    fn test_deserialize_task_with_null_arrays() {
        let json = serde_json::json!({
            "id": "t_null",
            "name": "Task with null arrays",
            "status": {
                "status": "open",
                "color": "#ccc",
                "type": "open"
            },
            "creator": {
                "id": 1,
                "username": "u",
                "email": "u@x.com"
            },
            "list": { "id": "l1" },
            "folder": { "id": "f1" },
            "space": { "id": "s1" },
            "assignees": null,
            "tags": null,
            "checklists": null,
            "linked_tasks": null,
            "dependencies": null,
            "watchers": null,
            "attachments": null,
            "subtasks": null,
            "custom_fields": null
        });
        let task: Task =
            serde_json::from_value(json).expect("task with null arrays should deserialize");
        assert!(task.assignees.is_empty());
        assert!(task.tags.is_empty());
        assert!(task.checklists.is_empty());
        assert!(task.linked_tasks.is_empty());
        assert!(task.dependencies.is_empty());
        assert!(task.watchers.is_empty());
        assert!(task.attachments.is_empty());
    }
}
