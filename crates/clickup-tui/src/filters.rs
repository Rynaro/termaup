use std::collections::HashMap;

use clickup_api::filter_config::DueDateFilter;
use clickup_api::models::{Task, User};

/// Active task filters for the current list view.
///
/// Combines server-side filters (status, assignee) that trigger re-fetches
/// with client-side filters (priority, tags, due date) applied locally.
#[derive(Debug, Clone, Default)]
pub struct TaskFilters {
    /// Status names to include (empty = all).
    pub statuses: Vec<String>,
    /// Assignee user IDs to include (empty = all).
    pub assignees: Vec<String>,
    /// Priority labels to include (empty = all, e.g. "urgent", "high").
    pub priorities: Vec<String>,
    /// Tag names to include (empty = all).
    pub tags: Vec<String>,
    /// Due-date filter preset.
    pub due_date_filter: DueDateFilter,
    /// Whether "Me Mode" is active.
    pub me_mode: bool,
    /// Whether to include closed tasks (server-side).
    pub include_closed: bool,
}

impl TaskFilters {
    /// Returns `true` if any server-side filter is active.
    ///
    /// When server-side filters change, we need to re-fetch from the API.
    pub fn has_server_filters(&self) -> bool {
        !self.statuses.is_empty() || !self.assignees.is_empty() || self.include_closed
    }

    /// Returns `true` if any client-side filter is active.
    pub fn has_client_filters(&self) -> bool {
        !self.priorities.is_empty()
            || !self.tags.is_empty()
            || self.due_date_filter != DueDateFilter::All
    }

    /// Returns `true` if any filter is active (server or client).
    pub fn is_active(&self) -> bool {
        self.has_server_filters() || self.has_client_filters() || self.me_mode
    }

    /// Returns the server-side status filter strings for the API call.
    pub fn server_statuses(&self) -> Vec<&str> {
        self.statuses.iter().map(|s| s.as_str()).collect()
    }

    /// Returns the server-side assignee IDs for the API call.
    ///
    /// If Me Mode is active and the current user is provided, includes
    /// the current user's ID in the assignee filter.
    pub fn server_assignees(&self, current_user: Option<&User>) -> Vec<String> {
        let mut ids: Vec<String> = self.assignees.clone();
        if self.me_mode
            && let Some(user) = current_user
        {
            let uid = user.id.to_string();
            if !ids.contains(&uid) {
                ids.push(uid);
            }
        }
        ids
    }

    /// Filters a task list client-side, returning indices of matching tasks.
    ///
    /// Applies priority, tag, and due-date filters. Status and assignee
    /// filters are handled server-side and NOT applied here.
    pub fn apply_client_filters(&self, tasks: &[Task], now_ms: i64) -> Vec<usize> {
        tasks
            .iter()
            .enumerate()
            .filter(|(_, task)| {
                self.matches_priority(task)
                    && self.matches_tags(task)
                    && self.matches_due_date(task, now_ms)
            })
            .map(|(i, _)| i)
            .collect()
    }

    /// Full filter pipeline: apply client-side filters then text search.
    ///
    /// Returns indices into the original `tasks` slice.
    pub fn apply_all(&self, tasks: &[Task], text_query: &str, now_ms: i64) -> Vec<usize> {
        let mut indices = self.apply_client_filters(tasks, now_ms);

        // Apply text search on top.
        if !text_query.is_empty() {
            let q = text_query.to_lowercase();
            indices.retain(|&i| tasks[i].name.to_lowercase().contains(&q));
        }

        indices
    }

    /// Counts tasks per status from a set of filtered task indices.
    pub fn count_by_status<'a>(tasks: &'a [Task], indices: &[usize]) -> HashMap<&'a str, usize> {
        let mut counts = HashMap::new();
        for &i in indices {
            let status = tasks[i].status.status.as_str();
            *counts.entry(status).or_insert(0) += 1;
        }
        counts
    }

    fn matches_priority(&self, task: &Task) -> bool {
        if self.priorities.is_empty() {
            return true;
        }
        match &task.priority {
            Some(p) => match &p.priority {
                Some(label) => self
                    .priorities
                    .iter()
                    .any(|f| f.eq_ignore_ascii_case(label)),
                None => self
                    .priorities
                    .iter()
                    .any(|f| f.eq_ignore_ascii_case("none")),
            },
            None => self
                .priorities
                .iter()
                .any(|f| f.eq_ignore_ascii_case("none")),
        }
    }

    fn matches_tags(&self, task: &Task) -> bool {
        if self.tags.is_empty() {
            return true;
        }
        task.tags
            .iter()
            .any(|t| self.tags.iter().any(|f| f.eq_ignore_ascii_case(&t.name)))
    }

    fn matches_due_date(&self, task: &Task, now_ms: i64) -> bool {
        match self.due_date_filter {
            DueDateFilter::All => true,
            DueDateFilter::NoDueDate => task.due_date.is_none(),
            DueDateFilter::Overdue => task
                .due_date
                .as_ref()
                .and_then(|d| d.parse::<i64>().ok())
                .is_some_and(|due| due < now_ms),
            DueDateFilter::Today => task
                .due_date
                .as_ref()
                .and_then(|d| d.parse::<i64>().ok())
                .is_some_and(|due| {
                    let day_ms = 86_400_000;
                    let today_start = (now_ms / day_ms) * day_ms;
                    let today_end = today_start + day_ms;
                    due >= today_start && due < today_end
                }),
            DueDateFilter::ThisWeek => task
                .due_date
                .as_ref()
                .and_then(|d| d.parse::<i64>().ok())
                .is_some_and(|due| {
                    let week_ms = 7 * 86_400_000;
                    due >= now_ms && due < now_ms + week_ms
                }),
        }
    }
}

/// Converts persistent filter config to active task filters.
impl From<clickup_api::filter_config::ListFilterConfig> for TaskFilters {
    fn from(cfg: clickup_api::filter_config::ListFilterConfig) -> Self {
        Self {
            statuses: cfg.statuses,
            assignees: cfg.assignees,
            priorities: cfg.priorities,
            tags: cfg.tags,
            due_date_filter: cfg.due_date_filter,
            me_mode: cfg.me_mode,
            include_closed: cfg.include_closed,
        }
    }
}

/// Converts active task filters to persistent filter config for saving.
impl From<&TaskFilters> for clickup_api::filter_config::ListFilterConfig {
    fn from(filters: &TaskFilters) -> Self {
        Self {
            statuses: filters.statuses.clone(),
            assignees: filters.assignees.clone(),
            priorities: filters.priorities.clone(),
            tags: filters.tags.clone(),
            due_date_filter: filters.due_date_filter,
            me_mode: filters.me_mode,
            view_mode: clickup_api::filter_config::ViewModeConfig::default(),
            include_closed: filters.include_closed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clickup_api::models::User;
    use clickup_api::models::task::{
        Tag, TaskFolder, TaskList, TaskPriority, TaskSpace, TaskStatus,
    };

    fn make_task(name: &str, status: &str, priority: Option<&str>, due: Option<&str>) -> Task {
        Task {
            id: name.to_lowercase().replace(' ', "_"),
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
            date_created: "1710000000000".to_string(),
            date_updated: "1710000000000".to_string(),
            date_closed: None,
            date_done: None,
            creator: Some(User {
                id: 1,
                username: "test".to_string(),
                email: "test@test.com".to_string(),
                color: None,
                profile_picture: None,
                initials: None,
            }),
            assignees: vec![],
            priority: priority.map(|p| TaskPriority {
                id: Some("1".to_string()),
                priority: Some(p.to_string()),
                color: Some("#ccc".to_string()),
            }),
            due_date: due.map(|d| d.to_string()),
            start_date: None,
            tags: vec![],
            list: Some(TaskList {
                id: "l1".to_string(),
                name: None,
            }),
            folder: Some(TaskFolder {
                id: "f1".to_string(),
                name: None,
            }),
            space: Some(TaskSpace {
                id: "s1".to_string(),
            }),
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
        }
    }

    #[test]
    fn test_empty_filters_match_all() {
        let filters = TaskFilters::default();
        let tasks = vec![
            make_task("Task 1", "open", Some("high"), None),
            make_task("Task 2", "closed", Some("low"), None),
        ];
        let indices = filters.apply_client_filters(&tasks, 1710000000000);
        assert_eq!(indices, vec![0, 1]);
    }

    #[test]
    fn test_priority_filter() {
        let filters = TaskFilters {
            priorities: vec!["high".into()],
            ..Default::default()
        };
        let tasks = vec![
            make_task("Task 1", "open", Some("high"), None),
            make_task("Task 2", "open", Some("low"), None),
            make_task("Task 3", "open", None, None),
        ];
        let indices = filters.apply_client_filters(&tasks, 0);
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn test_priority_none_filter() {
        let filters = TaskFilters {
            priorities: vec!["none".into()],
            ..Default::default()
        };
        let tasks = vec![
            make_task("Task 1", "open", Some("high"), None),
            make_task("Task 2", "open", None, None),
        ];
        let indices = filters.apply_client_filters(&tasks, 0);
        assert_eq!(indices, vec![1]);
    }

    #[test]
    fn test_tag_filter() {
        let mut task1 = make_task("Task 1", "open", None, None);
        task1.tags = vec![Tag {
            name: "frontend".to_string(),
            tag_fg: None,
            tag_bg: None,
        }];
        let task2 = make_task("Task 2", "open", None, None);

        let filters = TaskFilters {
            tags: vec!["frontend".into()],
            ..Default::default()
        };
        let tasks = vec![task1, task2];
        let indices = filters.apply_client_filters(&tasks, 0);
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn test_due_date_overdue() {
        let now_ms = 1710100000000_i64;
        let filters = TaskFilters {
            due_date_filter: DueDateFilter::Overdue,
            ..Default::default()
        };
        let tasks = vec![
            make_task("Overdue", "open", None, Some("1710000000000")),
            make_task("Future", "open", None, Some("1711000000000")),
            make_task("No date", "open", None, None),
        ];
        let indices = filters.apply_client_filters(&tasks, now_ms);
        assert_eq!(indices, vec![0]);
    }

    #[test]
    fn test_due_date_no_date() {
        let filters = TaskFilters {
            due_date_filter: DueDateFilter::NoDueDate,
            ..Default::default()
        };
        let tasks = vec![
            make_task("With date", "open", None, Some("1710000000000")),
            make_task("No date", "open", None, None),
        ];
        let indices = filters.apply_client_filters(&tasks, 0);
        assert_eq!(indices, vec![1]);
    }

    #[test]
    fn test_apply_all_with_text_search() {
        let filters = TaskFilters {
            priorities: vec!["high".into()],
            ..Default::default()
        };
        let tasks = vec![
            make_task("Login page", "open", Some("high"), None),
            make_task("Dashboard", "open", Some("high"), None),
            make_task("Settings", "open", Some("low"), None),
        ];
        let indices = filters.apply_all(&tasks, "dash", 0);
        assert_eq!(indices, vec![1]);
    }

    #[test]
    fn test_server_assignees_with_me_mode() {
        let filters = TaskFilters {
            me_mode: true,
            assignees: vec!["999".into()],
            ..Default::default()
        };
        let user = User {
            id: 42,
            username: "me".into(),
            email: "me@test.com".into(),
            color: None,
            profile_picture: None,
            initials: None,
        };
        let assignees = filters.server_assignees(Some(&user));
        assert_eq!(assignees, vec!["999", "42"]);
    }

    #[test]
    fn test_server_assignees_me_mode_no_duplicate() {
        let filters = TaskFilters {
            me_mode: true,
            assignees: vec!["42".into()],
            ..Default::default()
        };
        let user = User {
            id: 42,
            username: "me".into(),
            email: "me@test.com".into(),
            color: None,
            profile_picture: None,
            initials: None,
        };
        let assignees = filters.server_assignees(Some(&user));
        assert_eq!(assignees, vec!["42"]);
    }

    #[test]
    fn test_has_server_filters() {
        assert!(!TaskFilters::default().has_server_filters());
        assert!(
            TaskFilters {
                statuses: vec!["open".into()],
                ..Default::default()
            }
            .has_server_filters()
        );
        assert!(
            TaskFilters {
                include_closed: true,
                ..Default::default()
            }
            .has_server_filters()
        );
    }

    #[test]
    fn test_count_by_status() {
        let tasks = vec![
            make_task("T1", "open", None, None),
            make_task("T2", "open", None, None),
            make_task("T3", "in progress", None, None),
        ];
        let indices = vec![0, 1, 2];
        let counts = TaskFilters::count_by_status(&tasks, &indices);
        assert_eq!(counts["open"], 2);
        assert_eq!(counts["in progress"], 1);
    }

    #[test]
    fn test_from_list_filter_config() {
        use clickup_api::filter_config::ListFilterConfig;
        let cfg = ListFilterConfig {
            statuses: vec!["open".into()],
            me_mode: true,
            ..Default::default()
        };
        let filters: TaskFilters = cfg.into();
        assert_eq!(filters.statuses, vec!["open"]);
        assert!(filters.me_mode);
    }
}
