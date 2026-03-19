use serde::{Deserialize, Serialize};

use super::user::User;
use crate::serde_helpers::deserialize_string_or_number;

/// A comment on a ClickUp task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Plain-text comment body.
    #[serde(default)]
    pub comment_text: String,
    /// The user who posted the comment.
    #[serde(default)]
    pub user: Option<User>,
    /// Timestamp (milliseconds).
    #[serde(default)]
    pub date: String,
    /// Number of replies to this comment.
    #[serde(default)]
    pub reply_count: u64,
    /// Whether the comment has been resolved.
    #[serde(default)]
    pub resolved: Option<bool>,
    /// Parent comment ID when this comment is a reply.
    #[serde(default)]
    pub parent: Option<String>,
    /// User the comment is assigned to.
    #[serde(default)]
    pub assignee: Option<User>,
}

/// Response wrapper returned by `GET /task/{id}/comment`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentsResponse {
    /// List of comments.
    pub comments: Vec<Comment>,
}

/// Request body for creating a new comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    /// Plain-text comment body.
    pub comment_text: String,
    /// Whether to notify all assignees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_all: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_comment_full() {
        let json = serde_json::json!({
            "id": "c1",
            "comment_text": "Looks good!",
            "user": {
                "id": 123,
                "username": "alice",
                "email": "alice@example.com",
                "color": "#ff0000",
                "profilePicture": "https://example.com/avatar.png",
                "initials": "A"
            },
            "date": "1710000000000"
        });

        let comment: Comment = serde_json::from_value(json).expect("deserialize comment");
        assert_eq!(comment.id, "c1");
        assert_eq!(comment.comment_text, "Looks good!");
        assert_eq!(comment.date, "1710000000000");
        let user = comment.user.expect("user should be present");
        assert_eq!(user.id, 123);
        assert_eq!(user.username, "alice");
    }

    #[test]
    fn test_deserialize_comment_minimal() {
        let json = serde_json::json!({
            "id": "c2"
        });

        let comment: Comment = serde_json::from_value(json).expect("deserialize minimal comment");
        assert_eq!(comment.id, "c2");
        assert_eq!(comment.comment_text, "");
        assert!(comment.user.is_none());
        assert_eq!(comment.date, "");
        assert_eq!(comment.reply_count, 0);
        assert!(comment.resolved.is_none());
        assert!(comment.parent.is_none());
        assert!(comment.assignee.is_none());
    }

    #[test]
    fn test_deserialize_comment_null_user() {
        let json = serde_json::json!({
            "id": "c3",
            "comment_text": "Automated message",
            "user": null,
            "date": "1710000000000"
        });

        let comment: Comment =
            serde_json::from_value(json).expect("deserialize comment with null user");
        assert_eq!(comment.id, "c3");
        assert_eq!(comment.comment_text, "Automated message");
        assert!(comment.user.is_none());
    }

    #[test]
    fn test_deserialize_comments_response() {
        let json = serde_json::json!({
            "comments": [
                { "id": "c1", "comment_text": "First" },
                { "id": "c2", "comment_text": "Second" }
            ]
        });

        let resp: CommentsResponse =
            serde_json::from_value(json).expect("deserialize comments response");
        assert_eq!(resp.comments.len(), 2);
        assert_eq!(resp.comments[0].id, "c1");
        assert_eq!(resp.comments[1].comment_text, "Second");
    }

    #[test]
    fn test_deserialize_comment_with_replies() {
        let json = serde_json::json!({
            "id": "c10",
            "comment_text": "Thread starter",
            "reply_count": 3
        });

        let comment: Comment =
            serde_json::from_value(json).expect("deserialize comment with replies");
        assert_eq!(comment.id, "c10");
        assert_eq!(comment.reply_count, 3);
    }

    #[test]
    fn test_deserialize_comment_resolved() {
        let json = serde_json::json!({
            "id": "c11",
            "comment_text": "Fixed now",
            "resolved": true
        });

        let comment: Comment = serde_json::from_value(json).expect("deserialize resolved comment");
        assert_eq!(comment.id, "c11");
        assert_eq!(comment.resolved, Some(true));
    }

    #[test]
    fn test_deserialize_comment_as_reply() {
        let json = serde_json::json!({
            "id": "c12",
            "comment_text": "Reply text",
            "parent": "c_parent_id"
        });

        let comment: Comment = serde_json::from_value(json).expect("deserialize reply comment");
        assert_eq!(comment.id, "c12");
        assert_eq!(comment.parent.as_deref(), Some("c_parent_id"));
    }

    #[test]
    fn test_deserialize_comment_integer_id() {
        let json = serde_json::json!({
            "id": 90170193899_u64,
            "comment_text": "Created via API",
            "date": "1710000000000"
        });

        let comment: Comment =
            serde_json::from_value(json).expect("deserialize comment with integer id");
        assert_eq!(comment.id, "90170193899");
        assert_eq!(comment.comment_text, "Created via API");
    }

    #[test]
    fn test_serialize_create_comment_request() {
        let req = CreateCommentRequest {
            comment_text: "Hello".to_string(),
            notify_all: Some(true),
        };
        let value = serde_json::to_value(&req).expect("serialize request");
        assert_eq!(value["comment_text"], "Hello");
        assert_eq!(value["notify_all"], true);

        let req_no_notify = CreateCommentRequest {
            comment_text: "Quiet".to_string(),
            notify_all: None,
        };
        let value = serde_json::to_value(&req_no_notify).expect("serialize request without notify");
        assert_eq!(value["comment_text"], "Quiet");
        assert!(
            value.get("notify_all").is_none(),
            "notify_all should be omitted when None"
        );
    }
}
