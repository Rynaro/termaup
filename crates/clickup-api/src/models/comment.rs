use serde::{Deserialize, Serialize};

use super::user::User;
use super::workspace::WorkspaceMember;
use crate::serde_helpers::{deserialize_default_string_or_number, deserialize_string_or_number};

/// A user reference embedded in a comment content tag item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedUser {
    /// ClickUp user ID.
    pub id: i64,
    /// Display username (present on GET responses, omitted when sending).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    /// Email (present on GET responses, omitted when sending).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// A single segment in a ClickUp structured comment body.
///
/// ClickUp uses a `comment` array of heterogeneous items. The most common are
/// plain text and @mention tags. Unknown shapes (emoticons, attachments, etc.)
/// are captured by the `Unknown` variant to prevent deserialization failures.
///
/// The variants are ordered intentionally for `#[serde(untagged)]`:
/// `Tag` requires `user`, so it matches before `Text`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CommentContentItem {
    /// An @mention tag referencing a workspace user.
    Tag {
        /// Must be `"tag"` for this variant to match.
        #[serde(rename = "type")]
        content_type: String,
        /// The mentioned user (id required; username/email present on GET).
        user: TaggedUser,
        /// Plain-text representation of the mention (e.g. `"@alice"`), present
        /// on GET responses.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        text: Option<String>,
    },
    /// A plain-text segment, optionally with formatting attributes.
    Text {
        /// The text content.
        text: String,
        /// Formatting attributes (bold, italic, code, links, list, etc.).
        /// Captured as raw JSON to stay forward-compatible with new attribute
        /// types without breaking deserialization.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        attributes: Option<serde_json::Value>,
    },
    /// A fallback for any unrecognised item type (emoticons, attachments, etc.).
    Unknown(serde_json::Value),
}

/// Result returned by [`resolve_mentions`].
#[derive(Debug, Clone)]
pub struct MentionResolution {
    /// The fully-built structured comment content with tag items substituted
    /// for resolved mentions.
    pub comment_content: Vec<CommentContentItem>,
    /// Usernames that were successfully resolved to a user ID.
    pub resolved: Vec<String>,
    /// Usernames that appeared in the text but could not be matched to any
    /// workspace member.
    pub unresolved: Vec<String>,
}

/// Parses `text` for `@username` tokens at word boundaries and resolves them
/// against `members`, returning a [`MentionResolution`].
///
/// A word boundary is defined as: the `@` is at position 0 **or** is
/// immediately preceded by a whitespace character. This prevents
/// `user@example.com` from triggering a mention.
///
/// Segments between mentions are emitted as [`CommentContentItem::Text`] items.
/// Resolved mentions become [`CommentContentItem::Tag`] items. Unresolved
/// mentions are left as plain text.
pub fn resolve_mentions(text: &str, members: &[WorkspaceMember]) -> MentionResolution {
    let mut content: Vec<CommentContentItem> = Vec::new();
    let mut resolved: Vec<String> = Vec::new();
    let mut unresolved: Vec<String> = Vec::new();

    // Walk through the text character-by-character tracking runs.
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let mut segment_start = 0;

    while i < chars.len() {
        if chars[i] == '@' {
            // Check word-boundary: must be at start or preceded by whitespace.
            let at_word_boundary = i == 0 || chars[i - 1].is_whitespace();
            if at_word_boundary {
                // Collect the username token (alphanumeric + underscore + hyphen).
                let token_start = i + 1;
                let mut j = token_start;
                while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_' || chars[j] == '-') {
                    j += 1;
                }
                let username: String = chars[token_start..j].iter().collect();

                if !username.is_empty() {
                    // Flush any accumulated plain text before this mention.
                    let preceding: String = chars[segment_start..i].iter().collect();
                    if !preceding.is_empty() {
                        content.push(CommentContentItem::Text { text: preceding, attributes: None });
                    }

                    // Try to find a matching member (case-insensitive).
                    let member = members.iter().find(|m| {
                        m.user.username.eq_ignore_ascii_case(&username)
                    });

                    if let Some(m) = member {
                        content.push(CommentContentItem::Tag {
                            content_type: "tag".to_string(),
                            user: TaggedUser {
                                id: m.user.id,
                                username: None,
                                email: None,
                            },
                            text: None,
                        });
                        resolved.push(username);
                    } else {
                        // Unknown mention — emit as plain text.
                        content.push(CommentContentItem::Text {
                            text: format!("@{username}"),
                            attributes: None,
                        });
                        unresolved.push(username);
                    }

                    segment_start = j;
                    i = j;
                    continue;
                }
            }
        }
        i += 1;
    }

    // Flush remaining plain text.
    let tail: String = chars[segment_start..].iter().collect();
    if !tail.is_empty() {
        content.push(CommentContentItem::Text { text: tail, attributes: None });
    }

    MentionResolution { comment_content: content, resolved, unresolved }
}

/// A comment on a ClickUp task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment ID.
    #[serde(deserialize_with = "deserialize_string_or_number")]
    pub id: String,
    /// Plain-text comment body.
    #[serde(default)]
    pub comment_text: String,
    /// Structured comment content array (rich text with @mention tags).
    /// Present on GET responses when the comment uses rich formatting.
    #[serde(default)]
    pub comment: Vec<CommentContentItem>,
    /// The user who posted the comment.
    #[serde(default)]
    pub user: Option<User>,
    /// Timestamp (milliseconds).
    #[serde(default, deserialize_with = "deserialize_default_string_or_number")]
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
    /// Plain-text comment body (used when `comment` array is empty).
    pub comment_text: String,
    /// Structured comment content with @mention tags and rich formatting.
    /// When non-empty, ClickUp uses this array to send mention notifications.
    /// Omitted from the request body when empty.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comment: Vec<CommentContentItem>,
    /// Whether to notify all assignees.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notify_all: Option<bool>,
}

/// Request body for updating an existing comment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCommentRequest {
    /// Updated plain-text comment body.
    pub comment_text: String,
    /// User ID to assign the comment to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<i64>,
    /// Whether the comment is resolved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<bool>,
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
    fn test_deserialize_comment_integer_date() {
        let json = serde_json::json!({
            "id": "c20",
            "comment_text": "Just created",
            "date": 1773955968
        });

        let comment: Comment =
            serde_json::from_value(json).expect("deserialize comment with integer date");
        assert_eq!(comment.id, "c20");
        assert_eq!(comment.date, "1773955968");
    }

    #[test]
    fn test_serialize_create_comment_request() {
        let req = CreateCommentRequest {
            comment_text: "Hello".to_string(),
            comment: vec![],
            notify_all: Some(true),
        };
        let value = serde_json::to_value(&req).expect("serialize request");
        assert_eq!(value["comment_text"], "Hello");
        assert_eq!(value["notify_all"], true);

        let req_no_notify = CreateCommentRequest {
            comment_text: "Quiet".to_string(),
            comment: vec![],
            notify_all: None,
        };
        let value = serde_json::to_value(&req_no_notify).expect("serialize request without notify");
        assert_eq!(value["comment_text"], "Quiet");
        assert!(
            value.get("notify_all").is_none(),
            "notify_all should be omitted when None"
        );
    }

    #[test]
    fn test_serialize_update_comment_request_full() {
        let req = UpdateCommentRequest {
            comment_text: "Updated text".to_string(),
            assignee: Some(12345),
            resolved: Some(true),
        };
        let value = serde_json::to_value(&req).expect("serialize update request");
        assert_eq!(value["comment_text"], "Updated text");
        assert_eq!(value["assignee"], 12345);
        assert_eq!(value["resolved"], true);
    }

    #[test]
    fn test_serialize_update_comment_request_text_only() {
        let req = UpdateCommentRequest {
            comment_text: "Just editing text".to_string(),
            assignee: None,
            resolved: None,
        };
        let value = serde_json::to_value(&req).expect("serialize update request text only");
        assert_eq!(value["comment_text"], "Just editing text");
        assert!(
            value.get("assignee").is_none(),
            "assignee should be omitted when None"
        );
        assert!(
            value.get("resolved").is_none(),
            "resolved should be omitted when None"
        );
    }

    // --- CommentContentItem & resolve_mentions tests ---

    #[test]
    fn test_deserialize_comment_with_structured_content() {
        let json = serde_json::json!({
            "id": "c_rich",
            "comment_text": "Hi @alice check this",
            "comment": [
                { "text": "Hi " },
                { "type": "tag", "user": { "id": 123, "username": "alice" }, "text": "@alice" },
                { "text": " check this" }
            ]
        });

        let comment: Comment =
            serde_json::from_value(json).expect("deserialize structured comment");
        assert_eq!(comment.id, "c_rich");
        assert_eq!(comment.comment.len(), 3);

        match &comment.comment[0] {
            CommentContentItem::Text { text, .. } => assert_eq!(text, "Hi "),
            other => panic!("expected Text, got {other:?}"),
        }
        match &comment.comment[1] {
            CommentContentItem::Tag { content_type, user, text } => {
                assert_eq!(content_type, "tag");
                assert_eq!(user.id, 123);
                assert_eq!(user.username.as_deref(), Some("alice"));
                assert_eq!(text.as_deref(), Some("@alice"));
            }
            other => panic!("expected Tag, got {other:?}"),
        }
        match &comment.comment[2] {
            CommentContentItem::Text { text, .. } => assert_eq!(text, " check this"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn test_deserialize_comment_no_structured_content_defaults_to_empty() {
        let json = serde_json::json!({ "id": "c_plain", "comment_text": "plain text" });
        let comment: Comment = serde_json::from_value(json).expect("deserialize plain comment");
        assert!(comment.comment.is_empty(), "comment array should default to empty");
    }

    #[test]
    fn test_deserialize_unknown_item_type() {
        // An emoticon item has a "text" field so it matches the Text variant.
        // Unknown only fires for items with neither "user" nor "text".
        let json = serde_json::json!({
            "id": "c_emoji",
            "comment": [
                { "text": "U0001F60A", "type": "emoticon", "emoticon": { "code": "1f60a" } }
            ]
        });
        let comment: Comment = serde_json::from_value(json).expect("deserialize comment with emoticon");
        assert_eq!(comment.comment.len(), 1);
        // Emoticons have a "text" field → matched as Text; extra keys stored in attributes or ignored.
        assert!(
            matches!(&comment.comment[0], CommentContentItem::Text { .. }),
            "emoticon with text field should match Text variant"
        );
    }

    #[test]
    fn test_deserialize_truly_unknown_item_falls_back_to_unknown_variant() {
        // An item with no "text" and no "user" falls through to Unknown.
        let json = serde_json::json!({
            "id": "c_attachment",
            "comment": [
                { "type": "attachment", "attachment": { "url": "https://example.com/file.pdf" } }
            ]
        });
        let comment: Comment =
            serde_json::from_value(json).expect("deserialize comment with unknown item");
        assert_eq!(comment.comment.len(), 1);
        assert!(
            matches!(&comment.comment[0], CommentContentItem::Unknown(_)),
            "item with no text and no user should be Unknown"
        );
    }

    #[test]
    fn test_serialize_create_request_with_mentions_includes_comment_array() {
        use super::super::workspace::{WorkspaceMember};
        use super::super::user::User;

        let member = WorkspaceMember {
            user: User {
                id: 42,
                username: "alice".to_string(),
                email: "alice@example.com".to_string(),
                color: None,
                profile_picture: None,
                initials: None,
            },
        };
        let resolution = resolve_mentions("Hey @alice!", &[member]);
        let req = CreateCommentRequest {
            comment_text: "Hey @alice!".to_string(),
            comment: resolution.comment_content,
            notify_all: Some(true),
        };
        let value = serde_json::to_value(&req).expect("serialize");
        assert!(value.get("comment").is_some(), "comment key must be present");
        assert_eq!(value["comment"][1]["type"], "tag");
        assert_eq!(value["comment"][1]["user"]["id"], 42);
    }

    #[test]
    fn test_serialize_create_request_empty_comment_array_omitted() {
        let req = CreateCommentRequest {
            comment_text: "plain".to_string(),
            comment: vec![],
            notify_all: None,
        };
        let value = serde_json::to_value(&req).expect("serialize");
        assert!(value.get("comment").is_none(), "empty comment array must be omitted");
    }

    #[test]
    fn test_resolve_mentions_resolves_matching_members() {
        use super::super::workspace::WorkspaceMember;
        use super::super::user::User;

        let members = vec![
            WorkspaceMember {
                user: User {
                    id: 1,
                    username: "alice".to_string(),
                    email: "alice@example.com".to_string(),
                    color: None,
                    profile_picture: None,
                    initials: None,
                },
            },
            WorkspaceMember {
                user: User {
                    id: 2,
                    username: "bob".to_string(),
                    email: "bob@example.com".to_string(),
                    color: None,
                    profile_picture: None,
                    initials: None,
                },
            },
        ];

        let result = resolve_mentions("Please review @alice and @bob", &members);
        assert_eq!(result.resolved, vec!["alice", "bob"]);
        assert!(result.unresolved.is_empty());
        // 5 items: "Please review ", tag alice, " and ", tag bob, ""
        // (trailing empty text may or may not be emitted)
        let tags: Vec<_> = result.comment_content.iter().filter(|i| {
            matches!(i, CommentContentItem::Tag { .. })
        }).collect();
        assert_eq!(tags.len(), 2);
    }

    #[test]
    fn test_resolve_mentions_email_not_triggered() {
        use super::super::workspace::WorkspaceMember;
        use super::super::user::User;

        let member = WorkspaceMember {
            user: User {
                id: 1,
                username: "example".to_string(),
                email: "user@example.com".to_string(),
                color: None,
                profile_picture: None,
                initials: None,
            },
        };
        let result = resolve_mentions("Email user@example.com here", &[member]);
        assert!(result.resolved.is_empty(), "email @ should not trigger a mention");
        assert!(result.unresolved.is_empty());
    }

    #[test]
    fn test_resolve_mentions_unresolved_emits_plain_text() {
        let result = resolve_mentions("@unknown please help", &[]);
        assert!(result.resolved.is_empty());
        assert_eq!(result.unresolved, vec!["unknown"]);
        // Content should be plain text segments only
        assert!(result.comment_content.iter().all(|i| matches!(i, CommentContentItem::Text { .. })));
    }

    #[test]
    fn test_resolve_mentions_no_mentions_returns_single_text_segment() {
        let result = resolve_mentions("Just a plain message", &[]);
        assert!(result.resolved.is_empty());
        assert!(result.unresolved.is_empty());
        assert_eq!(result.comment_content.len(), 1);
        match &result.comment_content[0] {
            CommentContentItem::Text { text, .. } => assert_eq!(text, "Just a plain message"),
            other => panic!("expected Text, got {other:?}"),
        }
    }

    #[test]
    fn test_resolve_mentions_at_start_of_text() {
        use super::super::workspace::WorkspaceMember;
        use super::super::user::User;

        let member = WorkspaceMember {
            user: User {
                id: 7,
                username: "carol".to_string(),
                email: "carol@example.com".to_string(),
                color: None,
                profile_picture: None,
                initials: None,
            },
        };
        let result = resolve_mentions("@carol can you check?", &[member]);
        assert_eq!(result.resolved, vec!["carol"]);
        let first = &result.comment_content[0];
        assert!(matches!(first, CommentContentItem::Tag { .. }), "first item should be tag");
    }
}
