use serde::{Deserialize, Serialize};

use crate::serde_helpers::{deserialize_i32_or_string, deserialize_option_string_or_number};

/// Status of a task or list in ClickUp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Status {
    /// Status ID (may be absent on some responses).
    #[serde(default, deserialize_with = "deserialize_option_string_or_number")]
    pub id: Option<String>,
    /// Status name (e.g. "Open", "In Progress").
    #[serde(default)]
    pub status: String,
    /// Hex colour for the status.
    #[serde(default)]
    pub color: String,
    /// Status type (e.g. "open", "closed", "custom").
    #[serde(default, rename = "type")]
    pub status_type: String,
    /// Display order index.
    #[serde(default, deserialize_with = "deserialize_i32_or_string")]
    pub orderindex: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_status_full() {
        let json = serde_json::json!({
            "id": "s1",
            "status": "In Progress",
            "color": "#4194f6",
            "type": "custom",
            "orderindex": 2
        });

        let status: Status = serde_json::from_value(json).expect("deserialize status");
        assert_eq!(status.id.as_deref(), Some("s1"));
        assert_eq!(status.status, "In Progress");
        assert_eq!(status.color, "#4194f6");
        assert_eq!(status.status_type, "custom");
        assert_eq!(status.orderindex, 2);
    }

    #[test]
    fn test_deserialize_status_minimal() {
        let json = serde_json::json!({});

        let status: Status = serde_json::from_value(json).expect("deserialize minimal status");
        assert!(status.id.is_none());
        assert_eq!(status.status, "");
        assert_eq!(status.color, "");
        assert_eq!(status.status_type, "");
        assert_eq!(status.orderindex, 0);
    }

    #[test]
    fn test_deserialize_status_numeric_id() {
        let json = serde_json::json!({
            "id": 42,
            "status": "Open",
            "type": "open",
            "orderindex": "1"
        });

        let status: Status =
            serde_json::from_value(json).expect("deserialize status with numeric id");
        assert_eq!(status.id.as_deref(), Some("42"));
        assert_eq!(status.status, "Open");
        assert_eq!(status.orderindex, 1);
    }
}
