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
                handle_data(&mut app, payload);
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
