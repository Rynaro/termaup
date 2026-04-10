use serde::{Deserialize, Serialize};

use super::user::User;
use crate::serde_helpers::deserialize_option_string_or_number;

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
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_null_as_default"
    )]
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
    /// The user who created the task (absent on some task variants).
    #[serde(default)]
    pub creator: Option<User>,
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
    /// Parent list reference (absent on TIML tasks and some search results).
    #[serde(default)]
    pub list: Option<TaskList>,
    /// Parent folder reference (absent on TIML tasks and some search results).
    #[serde(default)]
    pub folder: Option<TaskFolder>,
    /// Parent space reference (absent on TIML tasks and some search results).
    #[serde(default)]
    pub space: Option<TaskSpace>,
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

impl Default for TaskStatus {
    fn default() -> Self {
        Self {
            status: "unknown".to_string(),
            color: "#808080".to_string(),
            status_type: "custom".to_string(),
        }
    }
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskList {
    /// List ID.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_default_string_or_number"
    )]
    pub id: String,
    /// List name.
    #[serde(default)]
    pub name: Option<String>,
}

/// Minimal folder reference embedded in a task.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskFolder {
    /// Folder ID.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_default_string_or_number"
    )]
    pub id: String,
    /// Folder name.
    #[serde(default)]
    pub name: Option<String>,
}

/// Minimal space reference embedded in a task.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TaskSpace {
    /// Space ID.
    #[serde(
        default,
        deserialize_with = "crate::serde_helpers::deserialize_default_string_or_number"
    )]
    pub id: String,
}

/// A custom field value on a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomField {
    /// Field ID.
    pub id: String,
    /// Field display name.
    pub name: String,
    /// Field type (e.g. "text", "number", "drop_down", "labels").
    #[serde(rename = "type")]
    pub field_type: String,
    /// Field configuration (options list for drop_down/labels, precision for
    /// currency, code_point for emoji, etc.). Preserved for value resolution.
    #[serde(default)]
    pub type_config: Option<serde_json::Value>,
    /// Current value (shape varies by field type).
    #[serde(default)]
    pub value: Option<serde_json::Value>,
}

impl CustomField {
    /// Returns a human-readable display string for this field's current value.
    ///
    /// Resolves option IDs to display names for `drop_down` and `labels` types
    /// using `type_config.options`. Falls back to the raw value string when
    /// `type_config` is absent or the option ID is not found.
    pub fn display_value(&self) -> String {
        let Some(val) = &self.value else {
            return "—".to_string();
        };
        match self.field_type.as_str() {
            "number" => format_number(val),
            "currency" => {
                let symbol = self
                    .type_config
                    .as_ref()
                    .and_then(|tc| tc.get("currency_type"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("$");
                format!("{} {}", symbol, format_number(val))
            }
            "checkbox" => {
                if val.as_bool().unwrap_or(false) {
                    "✅".to_string()
                } else {
                    "☐".to_string()
                }
            }
            "date" => {
                let ms_str;
                let ts = if let Some(s) = val.as_str() {
                    s
                } else if let Some(n) = val.as_i64() {
                    ms_str = n.to_string();
                    &ms_str
                } else {
                    return val.to_string();
                };
                if let Ok(ms) = ts.parse::<i64>() {
                    chrono::DateTime::from_timestamp_millis(ms)
                        .map(|dt| dt.format("%b %d, %Y").to_string())
                        .unwrap_or_else(|| ts.to_string())
                } else {
                    ts.to_string()
                }
            }
            "drop_down" => {
                let option_id = if let Some(s) = val.as_str() {
                    s.to_string()
                } else if let Some(n) = val.as_i64() {
                    // Some endpoints return the orderindex integer instead of UUID
                    return self
                        .type_config
                        .as_ref()
                        .and_then(|tc| tc.get("options"))
                        .and_then(|opts| opts.as_array())
                        .and_then(|arr| {
                            arr.iter().find(|o| {
                                o.get("orderindex")
                                    .and_then(|i| i.as_i64())
                                    .map(|i| i == n)
                                    .unwrap_or(false)
                            })
                        })
                        .and_then(|o| o.get("name").and_then(|n| n.as_str()))
                        .unwrap_or(&n.to_string())
                        .to_string();
                } else {
                    return val.to_string();
                };
                resolve_option_name(&self.type_config, &option_id, "name")
                    .unwrap_or(option_id)
            }
            "labels" => {
                let ids: Vec<String> = if let Some(arr) = val.as_array() {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                } else {
                    return val.to_string();
                };
                if ids.is_empty() {
                    return "—".to_string();
                }
                ids.iter()
                    .map(|id| {
                        // Labels use "label" key, not "name"
                        resolve_option_name(&self.type_config, id, "label")
                            .or_else(|| resolve_option_name(&self.type_config, id, "name"))
                            .unwrap_or_else(|| id.clone())
                    })
                    .collect::<Vec<_>>()
                    .join(", ")
            }
            "emoji" => {
                let count = val.as_u64().unwrap_or(0) as usize;
                if count == 0 {
                    return "—".to_string();
                }
                let emoji_char = self
                    .type_config
                    .as_ref()
                    .and_then(|tc| tc.get("code_point"))
                    .and_then(|v| v.as_str())
                    .and_then(|cp| u32::from_str_radix(cp, 16).ok())
                    .and_then(char::from_u32)
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "★".to_string());
                emoji_char.repeat(count)
            }
            "users" => {
                let users: Vec<&str> = if let Some(arr) = val.as_array() {
                    arr.iter()
                        .filter_map(|u| {
                            u.get("username")
                                .or_else(|| u.get("email"))
                                .and_then(|v| v.as_str())
                        })
                        .collect()
                } else {
                    return val.to_string();
                };
                if users.is_empty() {
                    "—".to_string()
                } else {
                    users.join(", ")
                }
            }
            "tasks" => {
                let ids: Vec<&str> = if let Some(arr) = val.as_array() {
                    arr.iter()
                        .filter_map(|t| t.as_str().or_else(|| t.get("id").and_then(|v| v.as_str())))
                        .collect()
                } else {
                    return val.to_string();
                };
                if ids.is_empty() {
                    "—".to_string()
                } else {
                    ids.join(", ")
                }
            }
            "manual_progress" => {
                let current = val
                    .get("current")
                    .and_then(|v| v.as_f64())
                    .unwrap_or_else(|| val.as_f64().unwrap_or(0.0));
                let start = self
                    .type_config
                    .as_ref()
                    .and_then(|tc| tc.get("start"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let end = self
                    .type_config
                    .as_ref()
                    .and_then(|tc| tc.get("end"))
                    .and_then(|v| v.as_f64())
                    .unwrap_or(100.0);
                let range = end - start;
                if range <= 0.0 {
                    return format!("{}", current as i64);
                }
                let pct = ((current - start) / range * 100.0).round() as i64;
                format!("{pct}%")
            }
            "automatic_progress" => {
                // Read-only; value is a percentage integer or object
                if let Some(n) = val.as_i64() {
                    format!("{n}%")
                } else if let Some(pct) = val.get("percent").and_then(|v| v.as_i64()) {
                    format!("{pct}%")
                } else {
                    val.to_string()
                }
            }
            "location" => val
                .get("formatted_address")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| val.to_string()),
            "url" | "email" | "phone" | "short_text" | "text" => {
                val.as_str().unwrap_or("—").to_string()
            }
            _ => val
                .as_str()
                .map(String::from)
                .unwrap_or_else(|| val.to_string()),
        }
    }
}

/// Formats a JSON number value as an integer string if whole, otherwise 2 d.p.
fn format_number(val: &serde_json::Value) -> String {
    if let Some(n) = val.as_f64() {
        if n.fract() == 0.0 {
            format!("{}", n as i64)
        } else {
            format!("{n:.2}")
        }
    } else {
        val.to_string()
    }
}

/// Looks up an option by `id` in `type_config.options` and returns the value
/// of `name_key` (typically `"name"` for dropdowns or `"label"` for labels).
fn resolve_option_name(
    type_config: &Option<serde_json::Value>,
    option_id: &str,
    name_key: &str,
) -> Option<String> {
    type_config
        .as_ref()?
        .get("options")?
        .as_array()?
        .iter()
        .find(|opt| {
            opt.get("id")
                .and_then(|v| v.as_str())
                .map(|id| id == option_id)
                .unwrap_or(false)
        })?
        .get(name_key)?
        .as_str()
        .map(String::from)
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
                    "type_config": {},
                    "value": 5,
                    "required": false
                },
                {
                    "id": "cf_2",
                    "name": "Priority Level",
                    "type": "drop_down",
                    "type_config": {
                        "options": [
                            { "id": "opt_a", "name": "High", "color": "#ff0000", "orderindex": 0 },
                            { "id": "opt_b", "name": "Medium", "color": "#ffff00", "orderindex": 1 },
                            { "id": "opt_c", "name": "Low", "color": "#00ff00", "orderindex": 2 }
                        ]
                    },
                    "value": "opt_a"
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
        assert_eq!(task.creator.as_ref().unwrap().username, "alice");
        assert_eq!(task.assignees.len(), 1);
        assert_eq!(task.assignees[0].username, "bob");

        let priority = task.priority.as_ref().unwrap();
        assert_eq!(priority.priority.as_deref(), Some("high"));

        assert_eq!(task.tags.len(), 1);
        assert_eq!(task.tags[0].name, "frontend");
        assert_eq!(task.list.as_ref().unwrap().id, "list_1");
        assert_eq!(
            task.folder.as_ref().unwrap().name.as_deref(),
            Some("Product")
        );
        assert_eq!(task.space.as_ref().unwrap().id, "space_1");
        assert_eq!(
            task.markdown_description.as_deref(),
            Some("# Login\nBuild it.")
        );

        let cf = task.custom_fields.as_ref().unwrap();
        assert_eq!(cf.len(), 2);
        assert_eq!(cf[0].name, "Story Points");
        assert_eq!(cf[0].field_type, "number");
        assert!(cf[0].type_config.is_some(), "type_config should be captured");
        assert_eq!(cf[1].name, "Priority Level");
        assert_eq!(cf[1].field_type, "drop_down");
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

    #[test]
    fn test_deserialize_task_bare_minimum() {
        let json = serde_json::json!({
            "id": "t_bare",
            "name": "Bare minimum task"
        });

        let task: Task =
            serde_json::from_value(json).expect("bare minimum task should deserialize");
        assert_eq!(task.id, "t_bare");
        assert_eq!(task.name, "Bare minimum task");
        assert_eq!(task.status.status, "unknown");
        assert!(task.creator.is_none());
        assert!(task.list.is_none());
        assert!(task.folder.is_none());
        assert!(task.space.is_none());
        assert!(task.assignees.is_empty());
        assert!(task.tags.is_empty());
        assert!(task.priority.is_none());
    }

    // ── display_value() tests ──────────────────────────────────────────────

    fn make_field(
        field_type: &str,
        type_config: serde_json::Value,
        value: serde_json::Value,
    ) -> CustomField {
        CustomField {
            id: "cf_test".to_string(),
            name: "Test Field".to_string(),
            field_type: field_type.to_string(),
            type_config: Some(type_config),
            value: Some(value),
        }
    }

    fn make_field_no_config(field_type: &str, value: serde_json::Value) -> CustomField {
        CustomField {
            id: "cf_test".to_string(),
            name: "Test Field".to_string(),
            field_type: field_type.to_string(),
            type_config: None,
            value: Some(value),
        }
    }

    #[test]
    fn test_display_value_none_returns_dash() {
        let field = CustomField {
            id: "cf".to_string(),
            name: "F".to_string(),
            field_type: "text".to_string(),
            type_config: None,
            value: None,
        };
        assert_eq!(field.display_value(), "—");
    }

    #[test]
    fn test_display_value_number_integer() {
        let f = make_field_no_config("number", serde_json::json!(5));
        assert_eq!(f.display_value(), "5");
    }

    #[test]
    fn test_display_value_number_float() {
        let f = make_field_no_config("number", serde_json::json!(3.14));
        assert_eq!(f.display_value(), "3.14");
    }

    #[test]
    fn test_display_value_currency_with_type() {
        let f = make_field(
            "currency",
            serde_json::json!({ "currency_type": "USD", "precision": 2 }),
            serde_json::json!(99.99),
        );
        assert_eq!(f.display_value(), "USD 99.99");
    }

    #[test]
    fn test_display_value_currency_no_config_fallback() {
        let f = make_field_no_config("currency", serde_json::json!(100));
        assert_eq!(f.display_value(), "$ 100");
    }

    #[test]
    fn test_display_value_checkbox_true() {
        let f = make_field_no_config("checkbox", serde_json::json!(true));
        assert_eq!(f.display_value(), "✅");
    }

    #[test]
    fn test_display_value_checkbox_false() {
        let f = make_field_no_config("checkbox", serde_json::json!(false));
        assert_eq!(f.display_value(), "☐");
    }

    #[test]
    fn test_display_value_date_from_ms_string() {
        // 2024-03-10 00:00:00 UTC
        let f = make_field_no_config("date", serde_json::json!("1710028800000"));
        let result = f.display_value();
        assert!(
            result.starts_with("Mar"),
            "date should start with Mar, got: {result}"
        );
        assert!(result.contains("2024"), "date should contain 2024, got: {result}");
    }

    #[test]
    fn test_display_value_date_from_integer() {
        let f = make_field_no_config("date", serde_json::json!(1710028800000_i64));
        let result = f.display_value();
        assert!(result.contains("2024"), "date should contain 2024, got: {result}");
    }

    #[test]
    fn test_display_value_dropdown_resolves_uuid() {
        let f = make_field(
            "drop_down",
            serde_json::json!({
                "options": [
                    { "id": "opt_a", "name": "High", "orderindex": 0 },
                    { "id": "opt_b", "name": "Medium", "orderindex": 1 }
                ]
            }),
            serde_json::json!("opt_a"),
        );
        assert_eq!(f.display_value(), "High");
    }

    #[test]
    fn test_display_value_dropdown_unknown_uuid_falls_back_to_raw() {
        let f = make_field(
            "drop_down",
            serde_json::json!({ "options": [{ "id": "opt_a", "name": "High" }] }),
            serde_json::json!("unknown-uuid"),
        );
        assert_eq!(f.display_value(), "unknown-uuid");
    }

    #[test]
    fn test_display_value_dropdown_no_type_config_falls_back() {
        let f = make_field_no_config("drop_down", serde_json::json!("opt_a"));
        assert_eq!(f.display_value(), "opt_a");
    }

    #[test]
    fn test_display_value_dropdown_orderindex_integer() {
        let f = make_field(
            "drop_down",
            serde_json::json!({
                "options": [
                    { "id": "opt_a", "name": "High", "orderindex": 0 },
                    { "id": "opt_b", "name": "Medium", "orderindex": 1 }
                ]
            }),
            serde_json::json!(1_i64),
        );
        assert_eq!(f.display_value(), "Medium");
    }

    #[test]
    fn test_display_value_labels_resolves_uuids() {
        let f = make_field(
            "labels",
            serde_json::json!({
                "options": [
                    { "id": "lbl_1", "label": "Bug", "color": "#e74c3c" },
                    { "id": "lbl_2", "label": "Feature", "color": "#27ae60" }
                ]
            }),
            serde_json::json!(["lbl_1", "lbl_2"]),
        );
        assert_eq!(f.display_value(), "Bug, Feature");
    }

    #[test]
    fn test_display_value_labels_unknown_id_falls_back() {
        let f = make_field(
            "labels",
            serde_json::json!({ "options": [{ "id": "lbl_1", "label": "Bug" }] }),
            serde_json::json!(["lbl_1", "unknown-id"]),
        );
        assert_eq!(f.display_value(), "Bug, unknown-id");
    }

    #[test]
    fn test_display_value_labels_empty_array() {
        let f = make_field(
            "labels",
            serde_json::json!({ "options": [] }),
            serde_json::json!([]),
        );
        assert_eq!(f.display_value(), "—");
    }

    #[test]
    fn test_display_value_emoji_renders_repeated() {
        let f = make_field(
            "emoji",
            serde_json::json!({ "code_point": "2b50", "count": 5 }),
            serde_json::json!(3),
        );
        assert_eq!(f.display_value(), "⭐⭐⭐");
    }

    #[test]
    fn test_display_value_emoji_zero() {
        let f = make_field(
            "emoji",
            serde_json::json!({ "code_point": "2b50", "count": 5 }),
            serde_json::json!(0),
        );
        assert_eq!(f.display_value(), "—");
    }

    #[test]
    fn test_display_value_emoji_no_config_fallback_star() {
        let f = make_field_no_config("emoji", serde_json::json!(2));
        assert_eq!(f.display_value(), "★★");
    }

    #[test]
    fn test_display_value_users_extracts_usernames() {
        let f = make_field_no_config(
            "users",
            serde_json::json!([
                { "id": 1, "username": "alice", "email": "alice@x.com" },
                { "id": 2, "username": "bob", "email": "bob@x.com" }
            ]),
        );
        assert_eq!(f.display_value(), "alice, bob");
    }

    #[test]
    fn test_display_value_users_empty_array() {
        let f = make_field_no_config("users", serde_json::json!([]));
        assert_eq!(f.display_value(), "—");
    }

    #[test]
    fn test_display_value_manual_progress_percentage() {
        let f = make_field(
            "manual_progress",
            serde_json::json!({ "start": 0, "end": 100 }),
            serde_json::json!({ "current": 75 }),
        );
        assert_eq!(f.display_value(), "75%");
    }

    #[test]
    fn test_display_value_manual_progress_custom_range() {
        let f = make_field(
            "manual_progress",
            serde_json::json!({ "start": 10, "end": 30 }),
            serde_json::json!({ "current": 20 }),
        );
        assert_eq!(f.display_value(), "50%");
    }

    #[test]
    fn test_display_value_location_formatted_address() {
        let f = make_field_no_config(
            "location",
            serde_json::json!({
                "location": { "lat": -28.016667, "lng": 153.4 },
                "formatted_address": "Gold Coast QLD, Australia"
            }),
        );
        assert_eq!(f.display_value(), "Gold Coast QLD, Australia");
    }

    #[test]
    fn test_display_value_text_types() {
        for field_type in &["text", "short_text", "email", "url", "phone"] {
            let f = make_field_no_config(field_type, serde_json::json!("hello world"));
            assert_eq!(f.display_value(), "hello world", "field_type={field_type}");
        }
    }

    #[test]
    fn test_display_value_unknown_type_string_value() {
        let f = make_field_no_config("future_field_type", serde_json::json!("some value"));
        assert_eq!(f.display_value(), "some value");
    }

    #[test]
    fn test_display_value_unknown_type_object_fallback() {
        let f = make_field_no_config("future_field_type", serde_json::json!({ "x": 1 }));
        // Should not panic; returns JSON representation
        let result = f.display_value();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_custom_field_type_config_absent_deserializes() {
        let json = serde_json::json!({
            "id": "cf_no_config",
            "name": "No Config Field",
            "type": "number",
            "value": 42
        });
        let cf: CustomField = serde_json::from_value(json).expect("type_config absent should deserialize");
        assert!(cf.type_config.is_none());
        assert_eq!(cf.display_value(), "42");
    }

    #[test]
    fn test_custom_field_type_config_null_deserializes() {
        let json = serde_json::json!({
            "id": "cf_null_config",
            "name": "Null Config Field",
            "type": "drop_down",
            "type_config": null,
            "value": "opt_x"
        });
        let cf: CustomField = serde_json::from_value(json).expect("type_config null should deserialize");
        assert!(cf.type_config.is_none());
        // Falls back to raw value
        assert_eq!(cf.display_value(), "opt_x");
    }

    #[test]
    fn test_custom_field_extra_api_fields_ignored() {
        let json = serde_json::json!({
            "id": "cf_extra",
            "name": "Extra Fields",
            "type": "text",
            "type_config": {},
            "value": "hello",
            "required": true,
            "date_created": "1710000000000",
            "hide_from_guests": false
        });
        let cf: CustomField = serde_json::from_value(json)
            .expect("extra API fields should be silently ignored");
        assert_eq!(cf.display_value(), "hello");
    }
    #[test]
    fn test_deserialize_timl_task_missing_structural_fields() {
        let json = serde_json::json!({
            "id": "t_timl",
            "name": "TIML task from another list",
            "status": {
                "status": "in progress",
                "color": "#4194f6",
                "type": "custom"
            },
            "creator": {
                "id": 1,
                "username": "u",
                "email": "u@x.com"
            },
            "assignees": [],
            "tags": []
        });

        let task: Task = serde_json::from_value(json).expect("TIML task without list/folder/space");
        assert_eq!(task.id, "t_timl");
        assert!(task.list.is_none(), "TIML task should have no list");
        assert!(task.folder.is_none(), "TIML task should have no folder");
        assert!(task.space.is_none(), "TIML task should have no space");
        assert!(task.creator.is_some());
    }

    #[test]
    fn test_deserialize_task_null_structural_fields() {
        let json = serde_json::json!({
            "id": "t_null_struct",
            "name": "Task with null structures",
            "status": null,
            "creator": null,
            "list": null,
            "folder": null,
            "space": null
        });

        let task: Task =
            serde_json::from_value(json).expect("null structural fields should deserialize");
        assert_eq!(task.status.status, "unknown", "null status defaults");
        assert!(task.creator.is_none());
        assert!(task.list.is_none());
        assert!(task.folder.is_none());
        assert!(task.space.is_none());
    }
}
