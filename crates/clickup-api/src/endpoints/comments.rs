use crate::client::ClickUpClient;
use crate::error::Result;
use crate::models::{Comment, CommentsResponse, CreateCommentRequest, UpdateCommentRequest};

impl ClickUpClient {
    /// Returns all comments on a task.
    pub async fn get_task_comments(&self, task_id: &str) -> Result<Vec<Comment>> {
        tracing::debug!(%task_id, "fetching task comments");
        let response: CommentsResponse = self.get(&format!("/task/{task_id}/comment")).await?;
        Ok(response.comments)
    }

    /// Returns threaded replies for a comment.
    pub async fn get_comment_replies(&self, comment_id: &str) -> Result<Vec<Comment>> {
        tracing::debug!(%comment_id, "fetching comment replies");
        let response: CommentsResponse = self.get(&format!("/comment/{comment_id}/reply")).await?;
        Ok(response.comments)
    }

    /// Creates a new comment on a task.
    pub async fn create_task_comment(
        &self,
        task_id: &str,
        request: &CreateCommentRequest,
    ) -> Result<Comment> {
        tracing::debug!(%task_id, "creating task comment");
        self.post(&format!("/task/{task_id}/comment"), request)
            .await
    }

    /// Creates a threaded reply on an existing comment.
    pub async fn create_comment_reply(
        &self,
        comment_id: &str,
        request: &CreateCommentRequest,
    ) -> Result<Comment> {
        tracing::debug!(%comment_id, "creating comment reply");
        self.post(&format!("/comment/{comment_id}/reply"), request)
            .await
    }

    /// Updates an existing comment's text, assignee, or resolved status.
    pub async fn update_comment(
        &self,
        comment_id: &str,
        request: &UpdateCommentRequest,
    ) -> Result<()> {
        tracing::debug!(%comment_id, "updating comment");
        self.put_no_body(&format!("/comment/{comment_id}"), request)
            .await
    }

    /// Deletes a comment permanently.
    pub async fn delete_comment(&self, comment_id: &str) -> Result<()> {
        tracing::debug!(%comment_id, "deleting comment");
        self.delete(&format!("/comment/{comment_id}")).await
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::client::ClickUpClient;

    #[tokio::test]
    async fn test_get_task_comments() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/task/abc/comment"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "comments": [
                    {
                        "id": "c1",
                        "comment_text": "Looks good!",
                        "user": {
                            "id": 1,
                            "username": "alice",
                            "email": "alice@example.com"
                        },
                        "date": "1710000000000"
                    },
                    {
                        "id": "c2",
                        "comment_text": "Needs revision",
                        "user": {
                            "id": 2,
                            "username": "bob",
                            "email": "bob@example.com"
                        },
                        "date": "1710100000000"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let comments = client.get_task_comments("abc").await.unwrap();
        assert_eq!(comments.len(), 2);
        assert_eq!(comments[0].id, "c1");
        assert_eq!(comments[0].comment_text, "Looks good!");
        assert_eq!(comments[0].user.as_ref().unwrap().username, "alice");
        assert_eq!(comments[1].id, "c2");
    }

    #[tokio::test]
    async fn test_create_task_comment() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v2/task/abc/comment"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": 90170193899_u64,
                "comment_text": "Great work!",
                "user": {
                    "id": 1,
                    "username": "alice",
                    "email": "alice@example.com"
                },
                "date": "1710000000000",
                "reply_count": 0,
                "resolved": false
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let request = crate::models::CreateCommentRequest {
            comment_text: "Great work!".to_string(),
            notify_all: Some(true),
        };
        let comment = client.create_task_comment("abc", &request).await.unwrap();
        assert_eq!(comment.id, "90170193899");
        assert_eq!(comment.comment_text, "Great work!");
        assert_eq!(comment.user.as_ref().unwrap().username, "alice");
        assert_eq!(comment.reply_count, 0);
        assert_eq!(comment.resolved, Some(false));
    }

    #[tokio::test]
    async fn test_get_comment_replies() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/comment/c1/reply"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "comments": [
                    {
                        "id": "r1",
                        "comment_text": "Thanks!",
                        "user": {
                            "id": 2,
                            "username": "bob",
                            "email": "bob@example.com"
                        },
                        "date": "1710200000000",
                        "parent": "c1"
                    },
                    {
                        "id": "r2",
                        "comment_text": "Agreed",
                        "user": {
                            "id": 3,
                            "username": "charlie",
                            "email": "charlie@example.com"
                        },
                        "date": 1710300000000_u64,
                        "parent": "c1"
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let replies = client.get_comment_replies("c1").await.unwrap();
        assert_eq!(replies.len(), 2);
        assert_eq!(replies[0].id, "r1");
        assert_eq!(replies[0].comment_text, "Thanks!");
        assert_eq!(replies[0].parent.as_deref(), Some("c1"));
        assert_eq!(replies[1].id, "r2");
        assert_eq!(replies[1].date, "1710300000000");
    }

    #[tokio::test]
    async fn test_create_comment_reply() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v2/comment/c1/reply"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "r10",
                "comment_text": "Replying here",
                "user": {
                    "id": 5,
                    "username": "eve",
                    "email": "eve@example.com"
                },
                "date": "1710400000000",
                "parent": "c1"
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let request = crate::models::CreateCommentRequest {
            comment_text: "Replying here".to_string(),
            notify_all: None,
        };
        let reply = client.create_comment_reply("c1", &request).await.unwrap();
        assert_eq!(reply.id, "r10");
        assert_eq!(reply.comment_text, "Replying here");
        assert_eq!(reply.parent.as_deref(), Some("c1"));
        assert_eq!(reply.user.as_ref().unwrap().username, "eve");
    }

    #[tokio::test]
    async fn test_update_comment() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/api/v2/comment/c1"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let request = crate::models::UpdateCommentRequest {
            comment_text: "Updated text".to_string(),
            assignee: None,
            resolved: None,
        };
        client.update_comment("c1", &request).await.unwrap();
    }

    #[tokio::test]
    async fn test_update_comment_with_resolved() {
        let server = MockServer::start().await;
        Mock::given(method("PUT"))
            .and(path("/api/v2/comment/c5"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let request = crate::models::UpdateCommentRequest {
            comment_text: "Resolved now".to_string(),
            assignee: None,
            resolved: Some(true),
        };
        client.update_comment("c5", &request).await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_comment() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v2/comment/c1"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        client.delete_comment("c1").await.unwrap();
    }

    #[tokio::test]
    async fn test_delete_comment_not_found() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v2/comment/nonexistent"))
            .respond_with(ResponseTemplate::new(404).set_body_json(serde_json::json!({
                "err": "Comment not found",
                "ECODE": "COMMENT_015"
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let err = client.delete_comment("nonexistent").await.unwrap_err();
        match err {
            crate::error::ClickUpError::NotFound(msg) => {
                assert_eq!(msg, "Comment not found");
            }
            other => panic!("expected NotFound, got: {other:?}"),
        }
    }
}
