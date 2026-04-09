use anyhow::{Context, Result};
use clap::Subcommand;
use owo_colors::OwoColorize;

use clickup_api::models::{
    comment::{resolve_mentions, CommentContentItem, MentionResolution},
    CreateCommentRequest, UpdateCommentRequest,
};

use crate::client_factory::create_client;
use crate::output;

/// Comment subcommands.
#[derive(Subcommand)]
pub enum CommentCommands {
    /// List comments on a task with threaded replies.
    List {
        /// Task ID to show comments for.
        #[arg(long)]
        task: String,
    },
    /// Create a new comment on a task.
    Create {
        /// Task ID to comment on.
        #[arg(long)]
        task: String,
        /// Comment text.
        #[arg(short, long)]
        message: String,
        /// Notify all task watchers.
        #[arg(long, default_value_t = false)]
        notify_all: bool,
    },
    /// Reply to an existing comment (threaded).
    Reply {
        /// Comment ID to reply to.
        #[arg(long)]
        comment: String,
        /// Reply text.
        #[arg(short, long)]
        message: String,
        /// Notify all task watchers.
        #[arg(long, default_value_t = false)]
        notify_all: bool,
    },
    /// Edit an existing comment's text.
    Edit {
        /// Comment ID to edit.
        #[arg(long)]
        comment: String,
        /// New comment text.
        #[arg(short, long)]
        message: String,
    },
    /// Delete a comment permanently.
    Delete {
        /// Comment ID to delete.
        #[arg(long)]
        comment: String,
        /// Skip confirmation prompt.
        #[arg(long, default_value_t = false)]
        yes: bool,
    },
}

impl CommentCommands {
    /// Dispatches to the appropriate comment handler.
    pub async fn run(self, format: &str, workspace_override: Option<&str>) -> Result<()> {
        match self {
            Self::List { task } => list_comments(format, workspace_override, &task).await,
            Self::Create {
                task,
                message,
                notify_all,
            } => create_comment(format, workspace_override, &task, &message, notify_all).await,
            Self::Reply {
                comment,
                message,
                notify_all,
            } => reply_comment(format, workspace_override, &comment, &message, notify_all).await,
            Self::Edit { comment, message } => {
                edit_comment(workspace_override, &comment, &message).await
            }
            Self::Delete { comment, yes } => {
                delete_comment(workspace_override, &comment, yes).await
            }
        }
    }
}

async fn list_comments(
    format: &str,
    workspace_override: Option<&str>,
    task_id: &str,
) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;

    let comments = client
        .get_task_comments(task_id)
        .await
        .context("failed to fetch comments")?;

    if comments.is_empty() {
        output::info(&format!("No comments on task {task_id}"));
        return Ok(());
    }

    if format == "json" {
        let mut threaded = Vec::new();
        for comment in &comments {
            let mut entry = serde_json::json!({
                "id": comment.id,
                "author": comment.user.as_ref().map(|u| u.username.as_str()).unwrap_or("Unknown"),
                "text": comment_body_text(&comment.comment, &comment.comment_text),
                "date": comment.date,
                "reply_count": comment.reply_count,
            });
            if comment.reply_count > 0 {
                match client.get_comment_replies(&comment.id).await {
                    Ok(replies) => {
                        let reply_json: Vec<serde_json::Value> = replies
                            .iter()
                            .map(|r| {
                                serde_json::json!({
                                    "id": r.id,
                                    "author": r.user.as_ref().map(|u| u.username.as_str()).unwrap_or("Unknown"),
                                    "text": comment_body_text(&r.comment, &r.comment_text),
                                    "date": r.date,
                                })
                            })
                            .collect();
                        entry["replies"] = serde_json::Value::Array(reply_json);
                    }
                    Err(e) => {
                        tracing::warn!("failed to load replies for {}: {e}", comment.id);
                    }
                }
            }
            threaded.push(entry);
        }
        output::print_json(&threaded);
        return Ok(());
    }

    output::info(&format!(
        "Comments on task {} ({} comment{})",
        task_id,
        comments.len(),
        if comments.len() == 1 { "" } else { "s" }
    ));

    let mut rows: Vec<Vec<String>> = Vec::new();

    for comment in &comments {
        let date = output::format_date(Some(&comment.date));
        rows.push(vec![
            comment.id.clone(),
            comment.user.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| "Unknown".to_string()),
            truncate_text(&comment_body_text(&comment.comment, &comment.comment_text), 50),
            date,
        ]);

        if comment.reply_count > 0 {
            match client.get_comment_replies(&comment.id).await {
                Ok(replies) => {
                    for (i, reply) in replies.iter().enumerate() {
                        let is_last = i == replies.len() - 1;
                        let connector = if is_last { "└" } else { "├" };
                        rows.push(vec![
                            format!("{connector} {}", reply.id),
                            reply.user.as_ref().map(|u| u.username.clone()).unwrap_or_else(|| "Unknown".to_string()),
                            truncate_text(&comment_body_text(&reply.comment, &reply.comment_text), 50),
                            output::format_date(Some(&reply.date)),
                        ]);
                    }
                }
                Err(e) => {
                    tracing::warn!("failed to load replies for {}: {e}", comment.id);
                }
            }
        }
    }

    output::print_table(&["ID", "Author", "Comment", "Date"], rows);

    Ok(())
}

async fn create_comment(
    _format: &str,
    workspace_override: Option<&str>,
    task_id: &str,
    message: &str,
    notify_all: bool,
) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;

    let (comment_content, resolved, unresolved) =
        resolve_mentions_for_message(&client, message).await?;
    print_mention_summary(&resolved, &unresolved);

    let request = CreateCommentRequest {
        comment_text: message.to_string(),
        notify_all: if notify_all { Some(true) } else { None },
        comment: comment_content,
    };

    let comment = client
        .create_task_comment(task_id, &request)
        .await
        .context("failed to create comment")?;

    output::success(&format!("Comment created (ID: {})", comment.id));
    Ok(())
}

async fn reply_comment(
    _format: &str,
    workspace_override: Option<&str>,
    comment_id: &str,
    message: &str,
    notify_all: bool,
) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;

    let (comment_content, resolved, unresolved) =
        resolve_mentions_for_message(&client, message).await?;
    print_mention_summary(&resolved, &unresolved);

    let request = CreateCommentRequest {
        comment_text: message.to_string(),
        notify_all: if notify_all { Some(true) } else { None },
        comment: comment_content,
    };

    let reply = client
        .create_comment_reply(comment_id, &request)
        .await
        .context("failed to create reply")?;

    output::success(&format!("Reply created (ID: {})", reply.id));
    Ok(())
}

async fn edit_comment(
    workspace_override: Option<&str>,
    comment_id: &str,
    message: &str,
) -> Result<()> {
    let (client, _config) = create_client(workspace_override)?;

    let request = UpdateCommentRequest {
        comment_text: message.to_string(),
        assignee: None,
        resolved: None,
    };

    client
        .update_comment(comment_id, &request)
        .await
        .context("failed to edit comment")?;

    output::success(&format!("Comment updated (ID: {comment_id})"));
    Ok(())
}

async fn delete_comment(
    workspace_override: Option<&str>,
    comment_id: &str,
    skip_confirm: bool,
) -> Result<()> {
    if !skip_confirm {
        let confirmed = dialoguer::Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete comment {comment_id}? This cannot be undone"
            ))
            .default(false)
            .interact()
            .context("failed to read confirmation")?;

        if !confirmed {
            output::info("Delete cancelled");
            return Ok(());
        }
    }

    let (client, _config) = create_client(workspace_override)?;

    client
        .delete_comment(comment_id)
        .await
        .context("failed to delete comment")?;

    output::success(&format!("Comment deleted (ID: {comment_id})"));
    Ok(())
}

/// Resolves `@username` tokens in `message` against workspace members.
///
/// Only fetches workspaces when the message contains `@`. Returns the
/// structured comment content (empty vec if no mentions) along with
/// a [`MentionResolution`] for reporting purposes.
async fn resolve_mentions_for_message(
    client: &clickup_api::client::ClickUpClient,
    message: &str,
) -> Result<(Vec<CommentContentItem>, Vec<String>, Vec<String>)> {
    if !message.contains('@') {
        return Ok((vec![], vec![], vec![]));
    }

    let workspaces = client
        .get_workspaces()
        .await
        .context("failed to fetch workspace members for mention resolution")?;

    // Collect all unique members across workspaces.
    let mut all_members: Vec<clickup_api::models::workspace::WorkspaceMember> = Vec::new();
    for ws in &workspaces {
        for member in &ws.members {
            if !all_members.iter().any(|m| m.user.id == member.user.id) {
                all_members.push(member.clone());
            }
        }
    }

    let resolution: MentionResolution = resolve_mentions(message, &all_members);
    Ok((
        resolution.comment_content,
        resolution.resolved,
        resolution.unresolved,
    ))
}

/// Prints a summary of resolved and unresolved @mention usernames.
fn print_mention_summary(resolved: &[String], unresolved: &[String]) {
    if !resolved.is_empty() {
        let names: Vec<String> = resolved.iter().map(|u| format!("@{u}")).collect();
        output::info(&format!("Mentioned: {}", names.join(", ")));
    }
    for u in unresolved {
        eprintln!(
            "{} Could not resolve @{u} — sent as plain text",
            "⚠".yellow()
        );
    }
}

/// Extracts plain-text from a structured comment array, rendering `Tag` items
/// as `@username`.
fn comment_body_text(items: &[CommentContentItem], fallback: &str) -> String {
    if items.is_empty() {
        return fallback.to_string();
    }
    let mut out = String::new();
    for item in items {
        match item {
            CommentContentItem::Tag { user, text, .. } => {
                if let Some(t) = text {
                    out.push_str(t);
                } else if let Some(ref uname) = user.username {
                    out.push('@');
                    out.push_str(uname);
                }
            }
            CommentContentItem::Text { text, .. } => out.push_str(text),
            CommentContentItem::Unknown(_) => {}
        }
    }
    if out.is_empty() {
        fallback.to_string()
    } else {
        out
    }
}
    let first_line = text.lines().next().unwrap_or(text);
    if first_line.chars().count() > max_len {
        let truncated: String = first_line.chars().take(max_len).collect();
        format!("{truncated}…")
    } else if text.lines().count() > 1 {
        format!("{first_line}…")
    } else {
        first_line.to_string()
    }
}
