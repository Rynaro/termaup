use tokio::sync::mpsc;

use clickup_api::client::ClickUpClient;

use crate::event::{AppEvent, DataPayload};

/// Loads workspaces and sends the result through the event channel.
pub fn spawn_load_workspaces(client: &ClickUpClient, tx: &mpsc::UnboundedSender<AppEvent>) {
    let client = client.clone();
    let tx = tx.clone();
    tokio::spawn(async move {
        tracing::debug!("loading workspaces");
        match client.get_workspaces().await {
            Ok(ws) => {
                let _ = tx.send(AppEvent::DataLoaded(DataPayload::Workspaces(ws)));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Failed to load workspaces: {e}")));
            }
        }
    });
}

/// Loads spaces for a workspace and sends the result.
pub fn spawn_load_spaces(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    team_id: &str,
) {
    let client = client.clone();
    let tx = tx.clone();
    let team_id = team_id.to_string();
    tokio::spawn(async move {
        tracing::debug!(%team_id, "loading spaces");
        match client.get_spaces(&team_id).await {
            Ok(spaces) => {
                let _ = tx.send(AppEvent::DataLoaded(DataPayload::Spaces(spaces)));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Failed to load spaces: {e}")));
            }
        }
    });
}

/// Loads folders and folderless lists for a space and sends the result.
///
/// Does NOT eagerly load tasks — task loading happens when the user
/// selects a specific list.
pub fn spawn_load_folders_and_lists(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    space_id: &str,
) {
    let client = client.clone();
    let tx = tx.clone();
    let space_id = space_id.to_string();
    tokio::spawn(async move {
        tracing::debug!(%space_id, "loading folders and lists");
        let mut folderless_lists = Vec::new();

        let folders = match client.get_folders(&space_id).await {
            Ok(f) => f,
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Failed to load folders: {e}")));
                return;
            }
        };

        match client.get_folderless_lists(&space_id).await {
            Ok(lists) => folderless_lists = lists,
            Err(e) => {
                tracing::warn!("failed to load folderless lists: {e}");
            }
        }

        let _ = tx.send(AppEvent::DataLoaded(DataPayload::FoldersAndLists(
            folders,
            folderless_lists,
        )));
    });
}

/// Loads tasks for a list and sends the result.
pub fn spawn_load_tasks(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    list_id: &str,
) {
    let client = client.clone();
    let tx = tx.clone();
    let list_id = list_id.to_string();
    tokio::spawn(async move {
        tracing::debug!(%list_id, "loading tasks");
        match client.get_tasks(&list_id).await {
            Ok(tasks) => {
                let _ = tx.send(AppEvent::DataLoaded(DataPayload::Tasks(tasks)));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Failed to load tasks: {e}")));
            }
        }
    });
}

/// Loads a single task with detail and sends the result.
pub fn spawn_load_task_detail(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    task_id: &str,
) {
    let client = client.clone();
    let tx = tx.clone();
    let task_id = task_id.to_string();
    tokio::spawn(async move {
        tracing::debug!(%task_id, "loading task detail");
        match client.get_task(&task_id).await {
            Ok(task) => {
                let _ = tx.send(AppEvent::DataLoaded(DataPayload::TaskDetail(Box::new(
                    task,
                ))));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Failed to load task detail: {e}")));
            }
        }
    });
}

/// Loads comments for a task and sends the result.
pub fn spawn_load_comments(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    task_id: &str,
) {
    let client = client.clone();
    let tx = tx.clone();
    let task_id = task_id.to_string();
    tokio::spawn(async move {
        tracing::debug!(%task_id, "loading comments");
        match client.get_task_comments(&task_id).await {
            Ok(comments) => {
                let _ = tx.send(AppEvent::DataLoaded(DataPayload::Comments(comments)));
            }
            Err(e) => {
                tracing::warn!("failed to load comments: {e}");
            }
        }
    });
}

/// Loads a single page of tasks with optional filters.
pub fn spawn_load_tasks_page(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    list_id: &str,
    page: usize,
    statuses: &[String],
    assignees: &[String],
    include_closed: bool,
) {
    let client = client.clone();
    let tx = tx.clone();
    let list_id = list_id.to_string();
    let statuses: Vec<String> = statuses.to_vec();
    let assignees: Vec<String> = assignees.to_vec();
    tokio::spawn(async move {
        tracing::debug!(%list_id, page, "loading task page");
        let status_refs: Vec<&str> = statuses.iter().map(|s| s.as_str()).collect();
        let assignee_refs: Vec<&str> = assignees.iter().map(|s| s.as_str()).collect();
        match client
            .get_tasks_page(&list_id, page, &status_refs, &assignee_refs, include_closed)
            .await
        {
            Ok(paginated) => {
                let _ = tx.send(AppEvent::DataLoaded(DataPayload::TasksPage {
                    tasks: paginated.data,
                    page,
                    last_page: paginated.last_page,
                }));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Failed to load tasks: {e}")));
            }
        }
    });
}

/// Loads the authenticated user and sends the result.
pub fn spawn_load_current_user(client: &ClickUpClient, tx: &mpsc::UnboundedSender<AppEvent>) {
    let client = client.clone();
    let tx = tx.clone();
    tokio::spawn(async move {
        tracing::debug!("loading current user");
        match client.get_authenticated_user().await {
            Ok(user) => {
                let _ = tx.send(AppEvent::DataLoaded(DataPayload::CurrentUser(user)));
            }
            Err(e) => {
                tracing::warn!("failed to load current user: {e}");
            }
        }
    });
}
