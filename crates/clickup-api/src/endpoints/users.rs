use tracing::instrument;

use crate::client::ClickUpClient;
use crate::error::Result;
use crate::models::{AuthenticatedUser, User};

impl ClickUpClient {
    /// Returns the authenticated user. Also serves as token validation.
    #[instrument(skip(self))]
    pub async fn get_authenticated_user(&self) -> Result<User> {
        tracing::debug!("fetching authenticated user");
        let response: AuthenticatedUser = self.get("/user").await?;
        Ok(response.user)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::client::ClickUpClient;

    #[tokio::test]
    async fn test_get_authenticated_user() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/user"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "user": {
                    "id": 1001,
                    "username": "alice",
                    "email": "alice@example.com",
                    "color": "#ff0000",
                    "profilePicture": null,
                    "initials": "A"
                }
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let user = client.get_authenticated_user().await.unwrap();
        assert_eq!(user.id, 1001_i64);
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");
    }
}
