use serde::{Deserialize, Serialize};

use super::status::Status;
use crate::serde_helpers::deserialize_string_or_number;

/// A ClickUp space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Space {
    /// Space ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Space name.
    pub name: String,
    /// Whether the space is private.
    #[serde(default)]
    pub private: bool,
    /// Hex colour.
    #[serde(default)]
    pub color: Option<String>,
    /// Avatar URL.
    #[serde(default)]
    pub avatar: Option<String>,
    /// Available statuses in this space.
    #[serde(default)]
    pub statuses: Vec<Status>,
    /// Whether multiple assignees are enabled.
    #[serde(default)]
    pub multiple_assignees: bool,
    /// Enabled feature flags (kept flexible for now).
    #[serde(default)]
    pub features: Option<serde_json::Value>,
}

/// Response wrapper returned by `GET /team/{id}/space`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacesResponse {
    /// List of spaces.
    pub spaces: Vec<Space>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_space() {
        let json = serde_json::json!({
            "id": "790",
            "name": "Engineering",
            "private": false,
            "color": "#7B68EE",
            "avatar": null,
            "statuses": [
                {
                    "id": "s1",
                    "status": "Open",
                    "color": "#d3d3d3",
                    "type": "open",
                    "orderindex": 0
                },
                {
                    "id": "s2",
                    "status": "Closed",
                    "color": "#6bc950",
                    "type": "closed",
                    "orderindex": 1
                }
            ],
            "multiple_assignees": true,
            "features": { "due_dates": { "enabled": true } }
        });

        let space: Space = serde_json::from_value(json).expect("deserialize space");
        assert_eq!(space.id, "790");
        assert_eq!(space.name, "Engineering");
        assert!(!space.private);
        assert_eq!(space.statuses.len(), 2);
        assert_eq!(space.statuses[0].status, "Open");
        assert_eq!(space.statuses[0].status_type, "open");
        assert!(space.multiple_assignees);
        assert!(space.features.is_some());
    }

    #[test]
    fn test_deserialize_space_minimal() {
        let json = serde_json::json!({
            "id": "1",
            "name": "Minimal"
        });

        let space: Space = serde_json::from_value(json).expect("deserialize minimal space");
        assert_eq!(space.id, "1");
        assert!(!space.private);
        assert!(space.statuses.is_empty());
    }
}
