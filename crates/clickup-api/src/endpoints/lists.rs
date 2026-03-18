use crate::client::ClickUpClient;
use crate::error::Result;
use crate::models::{List, ListsResponse};

impl ClickUpClient {
    /// Returns all lists inside a folder.
    pub async fn get_lists_in_folder(&self, folder_id: &str) -> Result<Vec<List>> {
        tracing::debug!(%folder_id, "fetching lists in folder");
        let response: ListsResponse = self.get(&format!("/folder/{folder_id}/list")).await?;
        Ok(response.lists)
    }

    /// Returns folderless lists in a space.
    pub async fn get_folderless_lists(&self, space_id: &str) -> Result<Vec<List>> {
        tracing::debug!(%space_id, "fetching folderless lists");
        let response: ListsResponse = self.get(&format!("/space/{space_id}/list")).await?;
        Ok(response.lists)
    }

    /// Returns details for a single list.
    pub async fn get_list(&self, list_id: &str) -> Result<List> {
        tracing::debug!(%list_id, "fetching list");
        self.get(&format!("/list/{list_id}")).await
    }
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::client::ClickUpClient;

    fn list_json(id: &str, name: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "name": name,
            "orderindex": 0,
            "status": null,
            "content": null,
            "task_count": "5",
            "space": { "id": "s1" },
            "folder": { "id": "f1", "name": "Folder", "hidden": false }
        })
    }

    #[tokio::test]
    async fn test_get_lists_in_folder() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/folder/f1/list"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "lists": [list_json("l1", "Backlog")] })),
            )
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let lists = client.get_lists_in_folder("f1").await.unwrap();
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].id, "l1");
        assert_eq!(lists[0].name, "Backlog");
    }

    #[tokio::test]
    async fn test_get_folderless_lists() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/space/s1/list"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(
                    serde_json::json!({ "lists": [list_json("l2", "Quick Tasks")] }),
                ),
            )
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let lists = client.get_folderless_lists("s1").await.unwrap();
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].name, "Quick Tasks");
    }

    #[tokio::test]
    async fn test_get_list() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/list/l1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(list_json("l1", "Backlog")))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let list = client.get_list("l1").await.unwrap();
        assert_eq!(list.id, "l1");
        assert_eq!(list.name, "Backlog");
    }
}
