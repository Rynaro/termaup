use serde::{Deserialize, Serialize};

use super::user::User;
use crate::serde_helpers::deserialize_string_or_number;

/// A ClickUp workspace (team).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Workspace ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Workspace name.
    pub name: String,
    /// Hex colour.
    #[serde(default)]
    pub color: Option<String>,
    /// Avatar URL.
    #[serde(default)]
    pub avatar: Option<String>,
    /// Members of the workspace.
    #[serde(default)]
    pub members: Vec<WorkspaceMember>,
}

/// A member within a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    /// The user record.
    pub user: User,
}

/// Response wrapper returned by `GET /team`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacesResponse {
    /// List of workspaces the authenticated user belongs to.
    pub teams: Vec<Workspace>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_workspace_full() {
        let json = serde_json::json!({
            "id": "12345",
            "name": "My Workspace",
            "color": "#7B68EE",
            "avatar": "https://example.com/avatar.png",
            "members": [
                {
                    "user": {
                        "id": 1,
                        "username": "alice",
                        "email": "alice@example.com"
                    }
                }
            ]
        });

        let ws: Workspace = serde_json::from_value(json).expect("deserialize workspace");
        assert_eq!(ws.id, "12345");
        assert_eq!(ws.name, "My Workspace");
        assert_eq!(ws.color.as_deref(), Some("#7B68EE"));
        assert_eq!(ws.avatar.as_deref(), Some("https://example.com/avatar.png"));
        assert_eq!(ws.members.len(), 1);
        assert_eq!(ws.members[0].user.username, "alice");
    }

    #[test]
    fn test_deserialize_workspace_minimal() {
        let json = serde_json::json!({
            "id": "1",
            "name": "Bare"
        });

        let ws: Workspace = serde_json::from_value(json).expect("deserialize minimal workspace");
        assert_eq!(ws.id, "1");
        assert_eq!(ws.name, "Bare");
        assert!(ws.color.is_none());
        assert!(ws.avatar.is_none());
        assert!(ws.members.is_empty());
    }

    #[test]
    fn test_deserialize_workspace_numeric_id() {
        let json = serde_json::json!({
            "id": 99999,
            "name": "Numeric"
        });

        let ws: Workspace =
            serde_json::from_value(json).expect("deserialize workspace with numeric id");
        assert_eq!(ws.id, "99999");
    }

    #[test]
    fn test_deserialize_workspaces_response() {
        let json = serde_json::json!({
            "teams": [
                { "id": "1", "name": "First" },
                { "id": "2", "name": "Second" }
            ]
        });

        let resp: WorkspacesResponse =
            serde_json::from_value(json).expect("deserialize workspaces response");
        assert_eq!(resp.teams.len(), 2);
        assert_eq!(resp.teams[0].name, "First");
        assert_eq!(resp.teams[1].name, "Second");
    }
}
