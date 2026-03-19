use crate::client::ClickUpClient;
use crate::error::Result;
use crate::models::{Comment, CommentsResponse, CreateCommentRequest};

impl ClickUpClient {
    /// Returns all comments on a task.
    pub async fn get_task_comments(&self, task_id: &str) -> Result<Vec<Comment>> {
        tracing::debug!(%task_id, "fetching task comments");
        let response: CommentsResponse = self.get(&format!("/task/{task_id}/comment")).await?;
        Ok(response.comments)
    }

    /// Creates a new comment on a task.
    pub async fn create_task_comment(&self, task_id: &str, comment_text: &str) -> Result<Comment> {
        tracing::debug!(%task_id, "creating task comment");
        let body = CreateCommentRequest {
            comment_text: comment_text.to_string(),
            notify_all: Some(true),
        };
        self.post(&format!("/task/{task_id}/comment"), &body).await
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

        let comment = client
            .create_task_comment("abc", "Great work!")
            .await
            .unwrap();
        assert_eq!(comment.id, "90170193899");
        assert_eq!(comment.comment_text, "Great work!");
        assert_eq!(comment.user.as_ref().unwrap().username, "alice");
        assert_eq!(comment.reply_count, 0);
        assert_eq!(comment.resolved, Some(false));
    }
}
