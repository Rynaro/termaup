mod app;
mod data;
mod event;
mod filters;
mod input;
mod theme;
mod ui;
mod widgets;

use std::io;
use std::panic;

use anyhow::{Context, Result};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use tracing_subscriber::EnvFilter;

use clickup_api::auth::TokenStorage;
use clickup_api::client::ClickUpClient;
use clickup_api::config::Config;

use app::App;
use event::{AppEvent, DataPayload, EventHandler};

#[tokio::main]
async fn main() -> Result<()> {
    // --- Tracing to file ---
    let log_dir = Config::config_dir();
    std::fs::create_dir_all(&log_dir).ok();
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("tui.log"))
        .context("failed to open tui.log")?;

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_env("CLICKUP_LOG").unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_writer(log_file)
        .with_ansi(false)
        .init();

    // --- Load token and create client ---
    let token =
        TokenStorage::get_token().context("not authenticated — run `clickup auth login` first")?;
    let config = Config::load().context("failed to load configuration")?;
    let client = ClickUpClient::with_base_url(token, config.api_base_url.clone());

    // --- Install panic hook that restores the terminal ---
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        original_hook(info);
    }));

    // --- Terminal setup ---
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // --- Run the app ---
    let result = run_app(&mut terminal, client).await;

    // --- Terminal teardown ---
    restore_terminal()?;

    result
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    client: ClickUpClient,
) -> Result<()> {
    let mut app = App::new();
    let (mut events, event_tx) = EventHandler::new();

    // Kick off initial data load.
    app.loading = true;
    data::spawn_load_workspaces(&client, &event_tx);
    data::spawn_load_current_user(&client, &event_tx);

    loop {
        terminal.draw(|frame| ui::render(&app, frame))?;

        match events.next().await {
            AppEvent::Key(key) => {
                input::handle_key(&mut app, key, &client, &event_tx);
            }
            AppEvent::Mouse(mouse) => {
                // content_y_offset = top bar (1) + border (1)
                input::handle_mouse(&mut app, mouse, &client, &event_tx, 2);
            }
            AppEvent::DataLoaded(payload) => {
                handle_data(&mut app, *payload);
            }
            AppEvent::Error(msg) => {
                app.loading = false;
                app.set_error(msg);
            }
            AppEvent::Tick => {
                app.tick();
            }
            AppEvent::Resize(_, _) => {}
        }

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn handle_data(app: &mut App, payload: DataPayload) {
    app.loading = false;
    app.last_updated = Some(std::time::Instant::now());
    match payload {
        DataPayload::Workspaces(ws) => {
            app.workspaces = ws;
            app.screen = app::Screen::WorkspaceSelect;
            app.reset_selection();
        }
        DataPayload::Spaces(spaces) => {
            app.spaces = spaces;
            app.reset_selection();
        }
        DataPayload::FoldersAndLists(folders, folderless_lists) => {
            app.folders = folders;
            app.lists = folderless_lists;
            app.rebuild_space_content();
            app.reset_selection();
        }
        DataPayload::Tasks(tasks) => {
            app.tasks = tasks;
            app.reset_selection();
            app.rebuild_status_groups();
        }
        DataPayload::TaskDetail(task) => {
            app.current_task = Some(*task);
        }
        DataPayload::Comments(comments) => {
            app.comments = comments;
        }
        DataPayload::CommentCreated(mut comment) => {
            // The POST response is sparse — backfill the current user
            // when the API omits it.
            if comment.user.is_none() {
                comment.user = app.current_user.clone();
            }
            app.comments.insert(0, *comment);
            app.comment_input_mode = app::CommentInputMode::Browse;
            app.comment_input_text.clear();
            app.comment_scroll_offset = 0;
            app.selected_comment_index = 0;
        }
        DataPayload::CommentReplies {
            comment_id,
            replies,
        } => {
            app.comment_replies.insert(comment_id, replies);
        }
        DataPayload::ReplyCreated {
            parent_comment_id,
            mut reply,
        } => {
            // Backfill the current user when the API omits it.
            if reply.user.is_none() {
                reply.user = app.current_user.clone();
            }
            // Insert into the replies cache.
            app.comment_replies
                .entry(parent_comment_id.clone())
                .or_default()
                .push(*reply);
            // Increment reply_count on the parent comment.
            if let Some(parent) = app.comments.iter_mut().find(|c| c.id == parent_comment_id) {
                parent.reply_count += 1;
            }
            // Auto-expand the thread so the new reply is visible.
            app.expanded_comments.insert(parent_comment_id);
            app.comment_input_mode = app::CommentInputMode::Browse;
            app.comment_input_text.clear();
            app.reply_target_id = None;
        }
        DataPayload::CommentUpdated {
            comment_id,
            new_text,
        } => {
            // Update the comment in top-level comments or in replies.
            if let Some(existing) = app.comments.iter_mut().find(|c| c.id == comment_id) {
                existing.comment_text = new_text.clone();
            } else {
                // Search in reply caches.
                for replies in app.comment_replies.values_mut() {
                    if let Some(reply) = replies.iter_mut().find(|r| r.id == comment_id) {
                        reply.comment_text = new_text.clone();
                        break;
                    }
                }
            }
            app.comment_input_mode = app::CommentInputMode::Browse;
            app.comment_input_text.clear();
            app.editing_comment_id = None;
            app.editing_parent_id = None;
        }
        DataPayload::CommentDeleted {
            comment_id,
            parent_comment_id,
        } => {
            if let Some(parent_id) = parent_comment_id {
                // It was a reply — remove from the reply cache.
                if let Some(replies) = app.comment_replies.get_mut(&parent_id) {
                    replies.retain(|r| r.id != comment_id);
                }
                // Decrement reply_count on the parent comment.
                if let Some(parent) = app.comments.iter_mut().find(|c| c.id == parent_id) {
                    parent.reply_count = parent.reply_count.saturating_sub(1);
                }
            } else {
                // It was a top-level comment — remove from main list.
                app.comments.retain(|c| c.id != comment_id);
                // Also remove cached replies for this comment.
                app.comment_replies.remove(&comment_id);
                app.expanded_comments.remove(&comment_id);
            }
            // Clamp selection index.
            let visible_count = app.visible_comment_items().len();
            if visible_count == 0 {
                app.selected_comment_index = 0;
            } else if app.selected_comment_index >= visible_count {
                app.selected_comment_index = visible_count - 1;
            }
            app.delete_confirm_target = None;
            app.delete_confirm_parent = None;
        }
        DataPayload::TasksPage {
            tasks,
            page,
            last_page,
        } => {
            if page == 0 {
                app.tasks = tasks;
                app.reset_selection();
            } else {
                app.tasks.extend(tasks);
            }
            app.current_page = page;
            app.has_more_pages = !last_page;
            app.loading_more = false;
            app.rebuild_status_groups();
        }
        DataPayload::CurrentUser(user) => {
            tracing::info!(user_id = user.id, username = %user.username, "authenticated user loaded");
            app.current_user = Some(user);
        }
    }
}
