use serde::{Deserialize, Serialize};

/// A ClickUp user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Numeric user ID (may be `-1` for system/unknown users on linked tasks).
    pub id: i64,
    /// Display name.
    pub username: String,
    /// Email address.
    pub email: String,
    /// Hex colour associated with the user.
    #[serde(default)]
    pub color: Option<String>,
    /// URL of the user's profile picture.
    #[serde(default, alias = "profilePicture", alias = "profilepicture")]
    pub profile_picture: Option<String>,
    /// User initials (used as avatar fallback).
    #[serde(default)]
    pub initials: Option<String>,
}

/// Response wrapper returned by `GET /user`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticatedUser {
    /// The authenticated user.
    pub user: User,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_user_with_negative_id() {
        let json = serde_json::json!({
            "id": -1,
            "username": "system",
            "email": "system@clickup.internal"
        });

        let user: User = serde_json::from_value(json).expect("should handle id: -1");
        assert_eq!(user.id, -1_i64);
        assert_eq!(user.username, "system");
    }
}
