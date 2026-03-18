use crate::client::ClickUpClient;
use crate::error::Result;
use crate::models::{Folder, FoldersResponse};

impl ClickUpClient {
    /// Returns all folders in a space.
    pub async fn get_folders(&self, space_id: &str) -> Result<Vec<Folder>> {
        tracing::debug!(%space_id, "fetching folders");
        let response: FoldersResponse = self.get(&format!("/space/{space_id}/folder")).await?;
        Ok(response.folders)
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::client::ClickUpClient;

    #[tokio::test]
    async fn test_get_folders() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/space/s1/folder"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "folders": [
                    {
                        "id": "f1",
                        "name": "Sprint 1",
                        "orderindex": 0,
                        "hidden": false,
                        "space": { "id": "s1", "name": "Eng" },
                        "task_count": "12",
                        "lists": []
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let folders = client.get_folders("s1").await.unwrap();
        assert_eq!(folders.len(), 1);
        assert_eq!(folders[0].id, "f1");
        assert_eq!(folders[0].name, "Sprint 1");
        assert_eq!(folders[0].space.id, "s1");
    }
}
