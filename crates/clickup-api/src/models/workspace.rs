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
