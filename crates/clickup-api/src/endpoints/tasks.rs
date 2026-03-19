use crate::client::ClickUpClient;
use crate::error::{ClickUpError, Result};
use crate::models::Task;
use crate::pagination::PaginatedResponse;

impl ClickUpClient {
    /// Returns all tasks in a list, automatically paginating.
    ///
    /// Includes tasks added to this list from other lists (TIML).
    pub async fn get_tasks(&self, list_id: &str) -> Result<Vec<Task>> {
        tracing::debug!(%list_id, "fetching tasks");
        self.get_all_pages(
            &format!("/list/{list_id}/task"),
            &[("include_timl", "true")],
            tasks_extractor,
        )
        .await
    }

    /// Returns tasks in a list filtered by statuses, assignees, and whether
    /// to include closed tasks. Automatically paginates.
    ///
    /// Includes tasks added to this list from other lists (TIML).
    pub async fn get_tasks_with_filters(
        &self,
        list_id: &str,
        statuses: &[&str],
        assignees: &[&str],
        include_closed: bool,
    ) -> Result<Vec<Task>> {
        tracing::debug!(
            %list_id,
            ?statuses,
            ?assignees,
            include_closed,
            "fetching tasks with filters"
        );

        let mut params: Vec<(&str, &str)> = vec![("include_timl", "true")];
        for s in statuses {
            params.push(("statuses[]", s));
        }
        for a in assignees {
            params.push(("assignees[]", a));
        }
        let closed_str;
        if include_closed {
            closed_str = "true".to_string();
            params.push(("include_closed", &closed_str));
        }

        self.get_all_pages(&format!("/list/{list_id}/task"), &params, tasks_extractor)
            .await
    }

    /// Returns a single page of tasks in a list, with optional filters.
    ///
    /// Unlike [`get_tasks`] which auto-paginates and returns all tasks,
    /// this method fetches exactly one page for progressive loading.
    pub async fn get_tasks_page(
        &self,
        list_id: &str,
        page: usize,
        statuses: &[&str],
        assignees: &[&str],
        include_closed: bool,
    ) -> Result<PaginatedResponse<Task>> {
        tracing::debug!(%list_id, page, ?statuses, ?assignees, include_closed, "fetching task page");

        let page_str = page.to_string();
        let mut params: Vec<(&str, &str)> = vec![("include_timl", "true"), ("page", &page_str)];
        for s in statuses {
            params.push(("statuses[]", s));
        }
        for a in assignees {
            params.push(("assignees[]", a));
        }
        let closed_str;
        if include_closed {
            closed_str = "true".to_string();
            params.push(("include_closed", &closed_str));
        }

        let value: serde_json::Value = self
            .get_with_params(&format!("/list/{list_id}/task"), &params)
            .await?;
        tasks_extractor(value)
    }

    /// Returns a single task by ID with subtasks and markdown description.
    pub async fn get_task(&self, task_id: &str) -> Result<Task> {
        tracing::debug!(%task_id, "fetching task");
        self.get_with_params(
            &format!("/task/{task_id}"),
            &[
                ("include_subtasks", "true"),
                ("include_markdown_description", "true"),
            ],
        )
        .await
    }
}

/// Extracts a [`PaginatedResponse<Task>`] from the raw JSON returned by the
/// tasks endpoint.
fn tasks_extractor(value: serde_json::Value) -> Result<PaginatedResponse<Task>> {
    let last_page = value
        .get("last_page")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let tasks_value = value
        .get("tasks")
        .cloned()
        .unwrap_or(serde_json::Value::Array(vec![]));

    let data: Vec<Task> = serde_json::from_value(tasks_value).map_err(ClickUpError::deserialization)?;
    Ok(PaginatedResponse::new(data, last_page))
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use crate::client::ClickUpClient;

    fn task_json(id: &str, name: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id,
            "name": name,
            "status": { "status": "open", "color": "#ccc", "type": "open" },
            "orderindex": "0",
            "date_created": "1710000000000",
            "date_updated": "1710000000000",
            "creator": { "id": 1, "username": "u", "email": "u@x.com" },
            "list": { "id": "l1" },
            "folder": { "id": "f1" },
            "space": { "id": "s1" },
            "url": "https://app.clickup.com/t/abc"
        })
    }

    #[tokio::test]
    async fn test_get_tasks_single_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/list/l1/task"))
            .and(query_param("include_timl", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tasks": [task_json("t1", "Task 1"), task_json("t2", "Task 2")],
                "last_page": true
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let tasks = client.get_tasks("l1").await.unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, "t1");
        assert_eq!(tasks[1].name, "Task 2");
    }

    #[tokio::test]
    async fn test_get_tasks_pagination() {
        let server = MockServer::start().await;

        // Page 0
        Mock::given(method("GET"))
            .and(path("/api/v2/list/l1/task"))
            .and(query_param("page", "0"))
            .and(query_param("include_timl", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tasks": [task_json("t1", "Task 1")],
                "last_page": false
            })))
            .mount(&server)
            .await;

        // Page 1
        Mock::given(method("GET"))
            .and(path("/api/v2/list/l1/task"))
            .and(query_param("page", "1"))
            .and(query_param("include_timl", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "tasks": [task_json("t2", "Task 2")],
                "last_page": true
            })))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let tasks = client.get_tasks("l1").await.unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, "t1");
        assert_eq!(tasks[1].id, "t2");
    }

    #[tokio::test]
    async fn test_get_task() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v2/task/abc"))
            .and(query_param("include_subtasks", "true"))
            .and(query_param("include_markdown_description", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(task_json("abc", "My Task")))
            .mount(&server)
            .await;

        let client =
            ClickUpClient::with_base_url("pk_test".into(), format!("{}/api/v2", server.uri()));

        let task = client.get_task("abc").await.unwrap();
        assert_eq!(task.id, "abc");
        assert_eq!(task.name, "My Task");
    }
}
