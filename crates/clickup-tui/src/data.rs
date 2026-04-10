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
                let _ = tx.send(AppEvent::DataLoaded(Box::new(DataPayload::Workspaces(ws))));
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
                let _ = tx.send(AppEvent::DataLoaded(Box::new(DataPayload::Spaces(spaces))));
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

        let _ = tx.send(AppEvent::DataLoaded(Box::new(
            DataPayload::FoldersAndLists(folders, folderless_lists),
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
                let _ = tx.send(AppEvent::DataLoaded(Box::new(DataPayload::Tasks(tasks))));
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
                let _ = tx.send(AppEvent::DataLoaded(Box::new(DataPayload::TaskDetail(
                    Box::new(task),
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
                let _ = tx.send(AppEvent::DataLoaded(Box::new(DataPayload::Comments(
                    comments,
                ))));
            }
            Err(e) => {
                tracing::warn!("failed to load comments: {e}");
            }
        }
    });
}

/// Creates a new comment on a task and sends the result.
pub fn spawn_create_comment(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    task_id: &str,
    comment_text: &str,
    comment_content: Vec<clickup_api::models::CommentContentItem>,
) {
    let client = client.clone();
    let tx = tx.clone();
    let task_id = task_id.to_string();
    let comment_text = comment_text.to_string();
    tokio::spawn(async move {
        tracing::debug!(%task_id, "creating comment");
        let request = clickup_api::models::CreateCommentRequest {
            // When the comment array is present it carries the full body;
            // sending comment_text alongside would cause ClickUp to render
            // the content twice.
            comment_text: if comment_content.is_empty() {
                comment_text.clone()
            } else {
                String::new()
            },
            comment: comment_content,
            notify_all: Some(true),
        };
        match client.create_task_comment(&task_id, &request).await {
            Ok(mut comment) => {
                // The POST response is sparse — backfill the text we submitted
                // when the API omits it.
                if comment.comment_text.is_empty() {
                    comment.comment_text = comment_text;
                }
                let _ = tx.send(AppEvent::DataLoaded(Box::new(DataPayload::CommentCreated(
                    Box::new(comment),
                ))));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Failed to create comment: {e}")));
            }
        }
    });
}

/// Loads threaded replies for a comment and sends the result.
pub fn spawn_load_comment_replies(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    comment_id: &str,
) {
    let client = client.clone();
    let tx = tx.clone();
    let comment_id = comment_id.to_string();
    tokio::spawn(async move {
        tracing::debug!(%comment_id, "loading comment replies");
        match client.get_comment_replies(&comment_id).await {
            Ok(replies) => {
                let _ = tx.send(AppEvent::DataLoaded(Box::new(
                    DataPayload::CommentReplies {
                        comment_id,
                        replies,
                    },
                )));
            }
            Err(e) => {
                tracing::warn!("failed to load replies for {comment_id}: {e}");
                let _ = tx.send(AppEvent::Error(format!("Failed to load replies: {e}")));
            }
        }
    });
}

/// Creates a threaded reply on a comment and sends the result.
pub fn spawn_create_comment_reply(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    comment_id: &str,
    comment_text: &str,
    comment_content: Vec<clickup_api::models::CommentContentItem>,
) {
    let client = client.clone();
    let tx = tx.clone();
    let comment_id = comment_id.to_string();
    let comment_text = comment_text.to_string();
    tokio::spawn(async move {
        tracing::debug!(%comment_id, "creating comment reply");
        let request = clickup_api::models::CreateCommentRequest {
            comment_text: if comment_content.is_empty() {
                comment_text.clone()
            } else {
                String::new()
            },
            comment: comment_content,
            notify_all: Some(true),
        };
        match client.create_comment_reply(&comment_id, &request).await {
            Ok(mut reply) => {
                if reply.comment_text.is_empty() {
                    reply.comment_text = comment_text;
                }
                let _ = tx.send(AppEvent::DataLoaded(Box::new(DataPayload::ReplyCreated {
                    parent_comment_id: comment_id,
                    reply: Box::new(reply),
                })));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Failed to create reply: {e}")));
            }
        }
    });
}

/// Updates an existing comment's text and sends the result.
///
/// For top-level comments (`parent_comment_id` is `None`), uses
/// `PUT /comment/{id}`.  For replies, the ClickUp API does not support
/// PUT on reply IDs, so we delete the old reply and create a new one on
/// the same thread.
pub fn spawn_update_comment(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    comment_id: &str,
    new_text: &str,
    parent_comment_id: Option<&str>,
) {
    let client = client.clone();
    let tx = tx.clone();
    let comment_id = comment_id.to_string();
    let new_text = new_text.to_string();
    let parent_comment_id = parent_comment_id.map(|s| s.to_string());
    tokio::spawn(async move {
        if let Some(parent_id) = parent_comment_id {
            // Reply edit: delete old reply, create new one on the same thread.
            tracing::debug!(%comment_id, %parent_id, "editing reply via delete+recreate");
            if let Err(e) = client.delete_comment(&comment_id).await {
                let _ = tx.send(AppEvent::Error(format!("Failed to edit reply: {e}")));
                return;
            }
            let request = clickup_api::models::CreateCommentRequest {
                comment_text: new_text.clone(),
                comment: vec![],
                notify_all: None,
            };
            match client.create_comment_reply(&parent_id, &request).await {
                Ok(mut reply) => {
                    if reply.comment_text.is_empty() {
                        reply.comment_text = new_text;
                    }
                    // Send delete of old + creation of new reply.
                    let _ = tx.send(AppEvent::DataLoaded(Box::new(
                        DataPayload::CommentDeleted {
                            comment_id,
                            parent_comment_id: Some(parent_id.clone()),
                        },
                    )));
                    let _ = tx.send(AppEvent::DataLoaded(Box::new(DataPayload::ReplyCreated {
                        parent_comment_id: parent_id,
                        reply: Box::new(reply),
                    })));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(format!("Failed to recreate reply: {e}")));
                }
            }
        } else {
            // Top-level comment: use PUT directly.
            tracing::debug!(%comment_id, "updating comment");
            let request = clickup_api::models::UpdateCommentRequest {
                comment_text: new_text.clone(),
                assignee: None,
                resolved: None,
            };
            match client.update_comment(&comment_id, &request).await {
                Ok(()) => {
                    let _ = tx.send(AppEvent::DataLoaded(Box::new(
                        DataPayload::CommentUpdated {
                            comment_id,
                            new_text,
                        },
                    )));
                }
                Err(e) => {
                    let _ = tx.send(AppEvent::Error(format!("Failed to update comment: {e}")));
                }
            }
        }
    });
}

/// Deletes a comment and sends the result.
pub fn spawn_delete_comment(
    client: &ClickUpClient,
    tx: &mpsc::UnboundedSender<AppEvent>,
    comment_id: &str,
    parent_comment_id: Option<&str>,
) {
    let client = client.clone();
    let tx = tx.clone();
    let comment_id = comment_id.to_string();
    let parent_comment_id = parent_comment_id.map(|s| s.to_string());
    tokio::spawn(async move {
        tracing::debug!(%comment_id, "deleting comment");
        match client.delete_comment(&comment_id).await {
            Ok(()) => {
                let _ = tx.send(AppEvent::DataLoaded(Box::new(
                    DataPayload::CommentDeleted {
                        comment_id,
                        parent_comment_id,
                    },
                )));
            }
            Err(e) => {
                let _ = tx.send(AppEvent::Error(format!("Failed to delete comment: {e}")));
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
                let _ = tx.send(AppEvent::DataLoaded(Box::new(DataPayload::TasksPage {
                    tasks: paginated.data,
                    page,
                    last_page: paginated.last_page,
                })));
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
                let _ = tx.send(AppEvent::DataLoaded(Box::new(DataPayload::CurrentUser(
                    user,
                ))));
            }
            Err(e) => {
                tracing::warn!("failed to load current user: {e}");
            }
        }
    });
}
