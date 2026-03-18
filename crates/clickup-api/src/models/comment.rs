use serde::{Deserialize, Serialize};

use super::user::User;

/// A comment on a ClickUp task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment ID.
    pub id: String,
    /// Plain-text comment body.
    pub comment_text: String,
    /// The user who posted the comment.
    pub user: User,
    /// Timestamp (milliseconds).
    pub date: String,
}

/// Response wrapper returned by `GET /task/{id}/comment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsResponse {
    /// List of comments.
    pub comments: Vec<Comment>,
}
