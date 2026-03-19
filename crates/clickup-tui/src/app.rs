use std::collections::HashMap;
use std::time::Instant;

use clickup_api::models::{Comment, Folder, List, Space, Status, Task, User, Workspace};

use crate::filters::TaskFilters;
use crate::ui::filter_panel::FilterPanelState;

/// The current screen of the TUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Screen {
    /// Initial loading state.
    Loading,
    /// Select a workspace.
    WorkspaceSelect,
    /// Browse spaces in a workspace.
    SpaceList,
    /// Browse folders and lists in a space.
    SpaceContent,
    /// Browse tasks in a single list.
    TaskList,
    /// View a single task's details.
    TaskDetail,
}

/// View mode for the task list screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    /// Standard flat task list.
    #[default]
    List,
    /// Vertical sections grouped by status.
    VisionSections,
    /// Horizontal kanban board grouped by status.
    VisionBoard,
}

impl ViewMode {
    /// Cycles to the next view mode.
    pub fn next(self) -> Self {
        match self {
            Self::List => Self::VisionSections,
            Self::VisionSections => Self::VisionBoard,
            Self::VisionBoard => Self::List,
        }
    }

    /// Human-readable label for the current view mode.
    pub fn label(self) -> &'static str {
        match self {
            Self::List => "List",
            Self::VisionSections => "Vision (Sections)",
            Self::VisionBoard => "Vision (Board)",
        }
    }
}

impl From<ViewMode> for clickup_api::filter_config::ViewModeConfig {
    fn from(mode: ViewMode) -> Self {
        match mode {
            ViewMode::List => Self::List,
            ViewMode::VisionSections => Self::VisionSections,
            ViewMode::VisionBoard => Self::VisionBoard,
        }
    }
}

impl From<clickup_api::filter_config::ViewModeConfig> for ViewMode {
    fn from(mode: clickup_api::filter_config::ViewModeConfig) -> Self {
        match mode {
            clickup_api::filter_config::ViewModeConfig::List => Self::List,
            clickup_api::filter_config::ViewModeConfig::VisionSections => Self::VisionSections,
            clickup_api::filter_config::ViewModeConfig::VisionBoard => Self::VisionBoard,
        }
    }
}

/// An item displayed in the SpaceContent screen.
#[derive(Debug, Clone)]
pub enum SpaceContentItem {
    /// A folder that can be expanded to show its lists.
    FolderItem {
        /// The folder.
        folder: Folder,
        /// Whether the folder's lists are visible.
        expanded: bool,
    },
    /// A list inside an expanded folder.
    ListInFolder {
        /// The list.
        list: List,
        /// Name of the parent folder (used for context).
        #[allow(dead_code)]
        folder_name: String,
    },
    /// A folderless list at the space root level.
    FolderlessList {
        /// The list.
        list: List,
    },
}

impl SpaceContentItem {
    /// Returns the display name for this item.
    pub fn display_name(&self) -> &str {
        match self {
            Self::FolderItem { folder, .. } => &folder.name,
            Self::ListInFolder { list, .. } | Self::FolderlessList { list } => &list.name,
        }
    }
}

/// Spinner characters for the loading animation.
const SPINNER_FRAMES: &[char] = &['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];

/// Top-level application state.
pub struct App {
    /// Current screen being displayed.
    pub screen: Screen,
    /// Whether the application should exit.
    pub should_quit: bool,

    // --- Data ---
    /// Loaded workspaces.
    pub workspaces: Vec<Workspace>,
    /// Loaded spaces for the current workspace.
    pub spaces: Vec<Space>,
    /// Currently selected workspace.
    pub current_workspace: Option<Workspace>,
    /// Currently selected space.
    pub current_space: Option<Space>,
    /// Folders within the current space.
    pub folders: Vec<Folder>,
    /// Lists within the current space (all combined).
    pub lists: Vec<List>,
    /// Flat list of items for the SpaceContent screen.
    pub space_content: Vec<SpaceContentItem>,
    /// Tasks in the current list.
    pub tasks: Vec<Task>,
    /// Currently viewed task.
    pub current_task: Option<Task>,
    /// Currently selected list.
    pub current_list: Option<List>,
    /// Comments on the current task.
    pub comments: Vec<Comment>,
    /// The authenticated user (loaded on startup for Me Mode).
    pub current_user: Option<User>,

    // --- Navigation ---
    /// Index of the highlighted item in the current list.
    pub selected_index: usize,
    /// Saved task list selection index (preserved on back-nav).
    pub saved_task_index: usize,
    /// Saved space content selection index (preserved on back-nav).
    pub saved_content_index: usize,
    /// Breadcrumb trail for navigation context.
    pub breadcrumb: Vec<String>,

    // --- Filter / search ---
    /// Whether the search bar is active.
    pub filter_active: bool,
    /// Current filter text.
    pub filter_text: String,
    /// Active structured filters for the current list.
    pub task_filters: TaskFilters,
    /// Whether the filter panel overlay is visible.
    pub filter_panel_open: bool,
    /// Filter panel navigation state.
    pub filter_panel_state: FilterPanelState,

    // --- View mode ---
    /// Current view mode for the task list.
    pub view_mode: ViewMode,
    /// Tasks grouped by status name (for Vision views).
    /// Maps status name → vec of indices into `self.tasks`.
    pub status_groups: Vec<StatusGroup>,
    /// In Vision views, which status group is focused.
    pub focused_group: usize,
    /// In Vision views, which item within the focused group is selected.
    pub group_selected_index: usize,

    // --- Pagination ---
    /// Current page of task results (0-indexed).
    pub current_page: usize,
    /// Whether more pages are available.
    pub has_more_pages: bool,
    /// Whether a background page load is in progress.
    pub loading_more: bool,

    // --- UI state ---
    /// Error message to display (auto-dismisses).
    pub error_message: Option<String>,
    /// When the current error message was set.
    pub error_set_at: Option<Instant>,
    /// Whether a background data load is in progress.
    pub loading: bool,
    /// Vertical scroll offset for long content.
    pub scroll_offset: u16,
    /// Whether the help overlay is visible.
    pub show_help: bool,
    /// Tick counter for spinner animation.
    pub tick_count: usize,
    /// When the current screen's data was last refreshed.
    pub last_updated: Option<Instant>,
}

/// A group of tasks sharing the same status, used for Vision views.
#[derive(Debug, Clone)]
pub struct StatusGroup {
    /// Status name.
    pub name: String,
    /// Status hex colour.
    pub color: String,
    /// Status type ("open", "closed", "custom").
    pub status_type: String,
    /// Indices into `App::tasks` for tasks in this group.
    pub task_indices: Vec<usize>,
}

impl App {
    /// Creates a new `App` with default state.
    pub fn new() -> Self {
        Self {
            screen: Screen::Loading,
            should_quit: false,
            workspaces: Vec::new(),
            spaces: Vec::new(),
            current_workspace: None,
            current_space: None,
            folders: Vec::new(),
            lists: Vec::new(),
            space_content: Vec::new(),
            tasks: Vec::new(),
            current_task: None,
            current_list: None,
            comments: Vec::new(),
            current_user: None,
            selected_index: 0,
            saved_task_index: 0,
            saved_content_index: 0,
            breadcrumb: vec!["ClickUp".to_string()],
            filter_active: false,
            filter_text: String::new(),
            task_filters: TaskFilters::default(),
            filter_panel_open: false,
            filter_panel_state: FilterPanelState::default(),
            view_mode: ViewMode::default(),
            status_groups: Vec::new(),
            focused_group: 0,
            group_selected_index: 0,
            current_page: 0,
            has_more_pages: false,
            loading_more: false,
            error_message: None,
            error_set_at: None,
            loading: false,
            scroll_offset: 0,
            show_help: false,
            tick_count: 0,
            last_updated: None,
        }
    }

    /// Pushes a label onto the breadcrumb trail.
    pub fn push_breadcrumb(&mut self, label: &str) {
        self.breadcrumb.push(label.to_string());
    }

    /// Pops the last breadcrumb label.
    pub fn pop_breadcrumb(&mut self) {
        if self.breadcrumb.len() > 1 {
            self.breadcrumb.pop();
        }
    }

    /// Moves the selection to the next item, wrapping around.
    pub fn select_next(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        self.selected_index = (self.selected_index + 1) % max;
    }

    /// Moves the selection to the previous item, wrapping around.
    pub fn select_previous(&mut self, max: usize) {
        if max == 0 {
            return;
        }
        if self.selected_index == 0 {
            self.selected_index = max - 1;
        } else {
            self.selected_index -= 1;
        }
    }

    /// Resets the selection index and scroll offset.
    pub fn reset_selection(&mut self) {
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    /// Sets an error message with a timestamp for auto-dismiss.
    pub fn set_error(&mut self, msg: String) {
        // Add helpful hints for common errors.
        let enhanced = if msg.contains("401") || msg.contains("auth") || msg.contains("Auth") {
            format!("{msg} — run `clickup auth login` to re-authenticate")
        } else if msg.contains("timed out") || msg.contains("connection") || msg.contains("network")
        {
            format!("{msg} — check your connection and token")
        } else {
            msg
        };
        self.error_message = Some(enhanced);
        self.error_set_at = Some(Instant::now());
    }

    /// Clears all task data and associated status groups.
    pub fn clear_tasks(&mut self) {
        self.tasks.clear();
        self.status_groups.clear();
    }

    /// Clears the error message.
    pub fn clear_error(&mut self) {
        self.error_message = None;
        self.error_set_at = None;
    }

    /// Auto-dismisses the error after 5 seconds. Call on each tick.
    pub fn maybe_dismiss_error(&mut self) {
        if let Some(set_at) = self.error_set_at
            && set_at.elapsed().as_secs() >= 5
        {
            self.clear_error();
        }
    }

    /// Advances the tick counter.
    pub fn tick(&mut self) {
        self.tick_count = self.tick_count.wrapping_add(1);
        self.maybe_dismiss_error();
    }

    /// Returns the current spinner character.
    pub fn spinner_char(&self) -> char {
        SPINNER_FRAMES[self.tick_count % SPINNER_FRAMES.len()]
    }

    /// Activates the filter/search bar.
    pub fn start_filter(&mut self) {
        self.filter_active = true;
        self.filter_text.clear();
    }

    /// Cancels the filter, restoring the full list.
    pub fn cancel_filter(&mut self) {
        self.filter_active = false;
        self.filter_text.clear();
        self.selected_index = 0;
    }

    /// Confirms the filter (stays filtered).
    pub fn confirm_filter(&mut self) {
        self.filter_active = false;
        self.selected_index = 0;
    }

    /// Returns filtered workspace indices matching the current filter.
    pub fn filtered_workspace_indices(&self) -> Vec<usize> {
        filter_indices(&self.workspaces, |ws| &ws.name, &self.filter_text)
    }

    /// Returns filtered space indices matching the current filter.
    pub fn filtered_space_indices(&self) -> Vec<usize> {
        filter_indices(&self.spaces, |s| &s.name, &self.filter_text)
    }

    /// Returns filtered task indices matching the current filter.
    pub fn filtered_task_indices(&self) -> Vec<usize> {
        filter_indices(&self.tasks, |t| &t.name, &self.filter_text)
    }

    /// Returns filtered space content indices matching the current filter.
    pub fn filtered_content_indices(&self) -> Vec<usize> {
        filter_indices(
            &self.space_content,
            |item| item.display_name(),
            &self.filter_text,
        )
    }

    /// Rebuilds the flat `space_content` list from folders and folderless lists.
    pub fn rebuild_space_content(&mut self) {
        let mut items = Vec::new();

        // Preserve expansion state from previous content.
        let expanded_folders: std::collections::HashSet<String> = self
            .space_content
            .iter()
            .filter_map(|item| match item {
                SpaceContentItem::FolderItem {
                    folder, expanded, ..
                } if *expanded => Some(folder.id.clone()),
                _ => None,
            })
            .collect();

        for folder in &self.folders {
            let expanded = expanded_folders.contains(&folder.id);
            items.push(SpaceContentItem::FolderItem {
                folder: folder.clone(),
                expanded,
            });
            if expanded {
                for list in &folder.lists {
                    items.push(SpaceContentItem::ListInFolder {
                        list: list.clone(),
                        folder_name: folder.name.clone(),
                    });
                }
            }
        }

        for list in &self.lists {
            items.push(SpaceContentItem::FolderlessList { list: list.clone() });
        }

        self.space_content = items;
    }

    /// Toggles a folder's expanded state and rebuilds the content list.
    pub fn toggle_folder(&mut self, content_index: usize) {
        if let Some(SpaceContentItem::FolderItem { expanded, .. }) =
            self.space_content.get_mut(content_index)
        {
            *expanded = !*expanded;
        }
        // Rebuild from scratch preserving the toggled state.
        let mut items = Vec::new();
        for item in &self.space_content {
            match item {
                SpaceContentItem::FolderItem { folder, expanded } => {
                    items.push(SpaceContentItem::FolderItem {
                        folder: folder.clone(),
                        expanded: *expanded,
                    });
                    if *expanded {
                        for list in &folder.lists {
                            items.push(SpaceContentItem::ListInFolder {
                                list: list.clone(),
                                folder_name: folder.name.clone(),
                            });
                        }
                    }
                }
                SpaceContentItem::FolderlessList { list } => {
                    items.push(SpaceContentItem::FolderlessList { list: list.clone() });
                }
                SpaceContentItem::ListInFolder { .. } => {
                    // Skip — will be re-generated from expanded folders.
                }
            }
        }
        self.space_content = items;
    }

    /// Human-readable "last updated" string.
    pub fn last_updated_label(&self) -> String {
        match self.last_updated {
            Some(at) => {
                let secs = at.elapsed().as_secs();
                if secs < 60 {
                    "just now".to_string()
                } else {
                    let mins = secs / 60;
                    if mins == 1 {
                        "1 min ago".to_string()
                    } else {
                        format!("{mins} min ago")
                    }
                }
            }
            None => String::new(),
        }
    }

    /// Rebuilds status groups from the current tasks, using space statuses
    /// for ordering.
    pub fn rebuild_status_groups(&mut self) {
        let statuses: &[Status] = self
            .current_space
            .as_ref()
            .map(|s| s.statuses.as_slice())
            .unwrap_or(&[]);

        // Build a map of status name (lowercase) → tasks.
        let mut group_map: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, task) in self.tasks.iter().enumerate() {
            group_map
                .entry(task.status.status.to_lowercase())
                .or_default()
                .push(i);
        }

        // Build groups in status order. Statuses with no tasks get empty groups.
        let mut groups = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for status in statuses {
            let key = status.status.to_lowercase();
            seen.insert(key.clone());
            groups.push(StatusGroup {
                name: status.status.clone(),
                color: status.color.clone(),
                status_type: status.status_type.clone(),
                task_indices: group_map.remove(&key).unwrap_or_default(),
            });
        }

        // Append any statuses not in the space definition (shouldn't happen but be safe).
        for (key, indices) in group_map {
            if !seen.contains(&key) {
                groups.push(StatusGroup {
                    name: key,
                    color: "#808080".to_string(),
                    status_type: "custom".to_string(),
                    task_indices: indices,
                });
            }
        }

        self.status_groups = groups;
    }

    /// Resets pagination state for a fresh task load.
    pub fn reset_pagination(&mut self) {
        self.current_page = 0;
        self.has_more_pages = false;
        self.loading_more = false;
    }

    /// Resets view mode navigation state.
    pub fn reset_vision_selection(&mut self) {
        self.focused_group = 0;
        self.group_selected_index = 0;
    }

    /// Total number of visible tasks across all status groups.
    pub fn total_grouped_tasks(&self) -> usize {
        self.status_groups
            .iter()
            .map(|g| g.task_indices.len())
            .sum()
    }
}

/// Generic case-insensitive filter returning matching indices.
fn filter_indices<T, F>(items: &[T], name_fn: F, query: &str) -> Vec<usize>
where
    F: Fn(&T) -> &str,
{
    if query.is_empty() {
        return (0..items.len()).collect();
    }
    let q = query.to_lowercase();
    items
        .iter()
        .enumerate()
        .filter(|(_, item)| name_fn(item).to_lowercase().contains(&q))
        .map(|(i, _)| i)
        .collect()
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_next_wraps() {
        let mut app = App::new();
        app.select_next(3);
        assert_eq!(app.selected_index, 1);
        app.select_next(3);
        assert_eq!(app.selected_index, 2);
        app.select_next(3);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_select_previous_wraps() {
        let mut app = App::new();
        app.select_previous(3);
        assert_eq!(app.selected_index, 2);
        app.select_previous(3);
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_select_with_zero_max() {
        let mut app = App::new();
        app.select_next(0);
        assert_eq!(app.selected_index, 0);
        app.select_previous(0);
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_breadcrumb_push_pop() {
        let mut app = App::new();
        assert_eq!(app.breadcrumb, vec!["ClickUp"]);
        app.push_breadcrumb("Workspace");
        assert_eq!(app.breadcrumb, vec!["ClickUp", "Workspace"]);
        app.pop_breadcrumb();
        assert_eq!(app.breadcrumb, vec!["ClickUp"]);
        // Cannot pop the root.
        app.pop_breadcrumb();
        assert_eq!(app.breadcrumb, vec!["ClickUp"]);
    }

    #[test]
    fn test_reset_selection() {
        let mut app = App::new();
        app.selected_index = 5;
        app.scroll_offset = 10;
        app.reset_selection();
        assert_eq!(app.selected_index, 0);
        assert_eq!(app.scroll_offset, 0);
    }

    #[test]
    fn test_spinner_cycles() {
        let mut app = App::new();
        let first = app.spinner_char();
        app.tick();
        let second = app.spinner_char();
        assert_ne!(first, second);
    }

    #[test]
    fn test_set_and_clear_error() {
        let mut app = App::new();
        app.set_error("oops".to_string());
        assert!(app.error_message.is_some());
        assert!(app.error_set_at.is_some());
        app.clear_error();
        assert!(app.error_message.is_none());
        assert!(app.error_set_at.is_none());
    }

    #[test]
    fn test_view_mode_cycles() {
        let mode = ViewMode::List;
        assert_eq!(mode.next(), ViewMode::VisionSections);
        assert_eq!(mode.next().next(), ViewMode::VisionBoard);
        assert_eq!(mode.next().next().next(), ViewMode::List);
    }

    #[test]
    fn test_view_mode_labels() {
        assert_eq!(ViewMode::List.label(), "List");
        assert_eq!(ViewMode::VisionSections.label(), "Vision (Sections)");
        assert_eq!(ViewMode::VisionBoard.label(), "Vision (Board)");
    }

    #[test]
    fn test_reset_pagination() {
        let mut app = App::new();
        app.current_page = 3;
        app.has_more_pages = true;
        app.loading_more = true;
        app.reset_pagination();
        assert_eq!(app.current_page, 0);
        assert!(!app.has_more_pages);
        assert!(!app.loading_more);
    }

    #[test]
    fn test_reset_vision_selection() {
        let mut app = App::new();
        app.focused_group = 2;
        app.group_selected_index = 5;
        app.reset_vision_selection();
        assert_eq!(app.focused_group, 0);
        assert_eq!(app.group_selected_index, 0);
    }

    #[test]
    fn test_rebuild_status_groups_empty() {
        let mut app = App::new();
        app.rebuild_status_groups();
        assert!(app.status_groups.is_empty());
    }

    #[test]
    fn test_rebuild_status_groups_with_space_statuses() {
        use clickup_api::models::status::Status;
        use clickup_api::models::task::{TaskFolder, TaskList, TaskSpace, TaskStatus};
        use clickup_api::models::{Space, Task, User};

        let mut app = App::new();
        app.current_space = Some(Space {
            id: "s1".to_string(),
            name: "Test".to_string(),
            private: false,
            color: None,
            avatar: None,
            statuses: vec![
                Status {
                    id: None,
                    status: "Open".to_string(),
                    color: "#ccc".to_string(),
                    status_type: "open".to_string(),
                    orderindex: 0,
                },
                Status {
                    id: None,
                    status: "Done".to_string(),
                    color: "#0f0".to_string(),
                    status_type: "closed".to_string(),
                    orderindex: 1,
                },
            ],
            multiple_assignees: false,
            features: None,
        });

        let make_task = |name: &str, status: &str| Task {
            id: name.to_lowercase(),
            custom_id: None,
            name: name.to_string(),
            description: None,
            text_content: None,
            status: TaskStatus {
                status: status.to_string(),
                color: "#ccc".to_string(),
                status_type: "custom".to_string(),
            },
            orderindex: "0".to_string(),
            date_created: "0".to_string(),
            date_updated: "0".to_string(),
            date_closed: None,
            date_done: None,
            creator: User {
                id: 1,
                username: "u".to_string(),
                email: "u@t.com".to_string(),
                color: None,
                profile_picture: None,
                initials: None,
            },
            assignees: vec![],
            priority: None,
            due_date: None,
            start_date: None,
            tags: vec![],
            list: TaskList {
                id: "l".to_string(),
                name: None,
            },
            folder: TaskFolder {
                id: "f".to_string(),
                name: None,
            },
            space: TaskSpace {
                id: "s".to_string(),
            },
            url: String::new(),
            markdown_description: None,
            parent: None,
            subtasks: None,
            custom_fields: None,
            checklists: vec![],
            linked_tasks: vec![],
            dependencies: vec![],
            time_estimate: None,
            time_spent: None,
            watchers: vec![],
            attachments: vec![],
            points: None,
            permission_level: None,
        };

        app.tasks = vec![
            make_task("Task 1", "Open"),
            make_task("Task 2", "Done"),
            make_task("Task 3", "Open"),
        ];

        app.rebuild_status_groups();

        assert_eq!(app.status_groups.len(), 2);
        assert_eq!(app.status_groups[0].name, "Open");
        assert_eq!(app.status_groups[0].task_indices, vec![0, 2]);
        assert_eq!(app.status_groups[1].name, "Done");
        assert_eq!(app.status_groups[1].task_indices, vec![1]);
    }

    #[test]
    fn test_view_mode_conversion_roundtrip() {
        use clickup_api::filter_config::ViewModeConfig;

        let modes = [
            ViewMode::List,
            ViewMode::VisionSections,
            ViewMode::VisionBoard,
        ];
        for mode in modes {
            let config: ViewModeConfig = mode.into();
            let back: ViewMode = config.into();
            assert_eq!(mode, back);
        }
    }
}
