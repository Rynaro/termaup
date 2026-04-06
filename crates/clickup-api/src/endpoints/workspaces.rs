use tracing::instrument;

use crate::client::ClickUpClient;
use crate::error::Result;
use crate::models::{Workspace, WorkspacesResponse};

impl ClickUpClient {
    /// Returns all workspaces (teams) the authenticated user belongs to.
    #[instrument(skip(self))]
    pub async fn get_workspaces(&self) -> Result<Vec<Workspace>> {
        tracing::debug!("fetching workspaces");
        let response: WorkspacesResponse = self.get("/team").await?;
        Ok(response.teams)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::client::ClickUpClient;

    #[tokio::test]
    async fn test_get_workspaces() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/team"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "teams": [
                    {
                        "id": "team_1",
                        "name": "Acme Corp",
                        "color": "#000",
                        "avatar": null,
                        "members": []
                    },
                    {
                        "id": "team_2",
                        "name": "Side Project",
                        "color": "#fff",
                        "avatar": null,
                        "members": []
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let workspaces = client.get_workspaces().await.unwrap();
        assert_eq!(workspaces.len(), 2);
        assert_eq!(workspaces[0].id, "team_1");
        assert_eq!(workspaces[0].name, "Acme Corp");
        assert_eq!(workspaces[1].id, "team_2");
    }
}
