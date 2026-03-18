use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use tokio::sync::mpsc;

use clickup_api::models::{Comment, Folder, List, Space, Task, User, Workspace};

/// Events processed by the TUI main loop.
pub enum AppEvent {
    /// A keyboard event.
    Key(KeyEvent),
    /// A mouse event.
    Mouse(MouseEvent),
    /// Periodic tick for animations / spinners.
    Tick,
    /// Terminal resize.
    #[allow(dead_code)]
    Resize(u16, u16),
    /// Data loaded from the API.
    DataLoaded(DataPayload),
    /// An error occurred during data loading.
    Error(String),
}

/// Payloads delivered by background data-loading tasks.
pub enum DataPayload {
    /// Workspace list loaded.
    Workspaces(Vec<Workspace>),
    /// Space list loaded.
    Spaces(Vec<Space>),
    /// Folders and folderless lists loaded for a space.
    FoldersAndLists(Vec<Folder>, Vec<List>),
    /// Tasks loaded for a list.
    Tasks(Vec<Task>),
    /// Single task detail loaded.
    TaskDetail(Box<Task>),
    /// Comments loaded for a task.
    Comments(Vec<Comment>),
    /// A single page of tasks loaded (for progressive pagination).
    TasksPage {
        /// Tasks on this page.
        tasks: Vec<Task>,
        /// Page number (0-indexed).
        page: usize,
        /// Whether this is the last page.
        last_page: bool,
    },
    /// The authenticated user loaded.
    CurrentUser(User),
}

const TICK_RATE: Duration = Duration::from_millis(250);

/// Polls terminal events and forwards them through a channel.
pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<AppEvent>,
}

impl EventHandler {
    /// Creates a new `EventHandler`, spawning background polling tasks.
    ///
    /// Returns the handler and the sender that background data-loading
    /// tasks can use to push [`AppEvent`] values.
    pub fn new() -> (Self, mpsc::UnboundedSender<AppEvent>) {
        let (tx, rx) = mpsc::unbounded_channel();

        let event_tx = tx.clone();
        tokio::spawn(async move {
            loop {
                // Poll crossterm for terminal events.
                let has_event =
                    tokio::task::spawn_blocking(|| event::poll(TICK_RATE).unwrap_or(false))
                        .await
                        .unwrap_or(false);

                if has_event {
                    if let Ok(ev) = crossterm::event::read() {
                        let app_event = match ev {
                            Event::Key(key) => AppEvent::Key(key),
                            Event::Mouse(mouse) => AppEvent::Mouse(mouse),
                            Event::Resize(w, h) => AppEvent::Resize(w, h),
                            _ => AppEvent::Tick,
                        };
                        if event_tx.send(app_event).is_err() {
                            break;
                        }
                    }
                } else {
                    // No terminal event within TICK_RATE — send a tick.
                    if event_tx.send(AppEvent::Tick).is_err() {
                        break;
                    }
                }
            }
        });

        (Self { rx }, tx)
    }

    /// Waits for and returns the next event.
    pub async fn next(&mut self) -> AppEvent {
        self.rx.recv().await.unwrap_or(AppEvent::Tick)
    }
}
