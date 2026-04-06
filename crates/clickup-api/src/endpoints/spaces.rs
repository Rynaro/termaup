use tracing::instrument;

use crate::client::ClickUpClient;
use crate::error::Result;
use crate::models::{Space, SpacesResponse};

impl ClickUpClient {
    /// Returns all spaces in a workspace.
    #[instrument(skip(self), fields(%team_id))]
    pub async fn get_spaces(&self, team_id: &str) -> Result<Vec<Space>> {
        tracing::debug!(%team_id, "fetching spaces");
        let response: SpacesResponse = self.get(&format!("/team/{team_id}/space")).await?;
        Ok(response.spaces)
    }

    /// Returns details for a single space.
    #[instrument(skip(self), fields(%space_id))]
    pub async fn get_space(&self, space_id: &str) -> Result<Space> {
        tracing::debug!(%space_id, "fetching space");
        self.get(&format!("/space/{space_id}")).await
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::client::ClickUpClient;

    #[tokio::test]
    async fn test_get_spaces() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/team/t1/space"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "spaces": [
                    {
                        "id": "s1",
                        "name": "Engineering",
                        "private": false,
                        "statuses": [],
                        "multiple_assignees": true
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let spaces = client.get_spaces("t1").await.unwrap();
        assert_eq!(spaces.len(), 1);
        assert_eq!(spaces[0].name, "Engineering");
    }

    #[tokio::test]
    async fn test_get_space() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/space/s1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "s1",
                "name": "Design",
                "private": true,
                "statuses": [],
                "multiple_assignees": false
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let space = client.get_space("s1").await.unwrap();
        assert_eq!(space.id, "s1");
        assert_eq!(space.name, "Design");
        assert!(space.private);
    }
}
