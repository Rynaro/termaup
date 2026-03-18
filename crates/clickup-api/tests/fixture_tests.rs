//! Integration tests that load JSON fixtures and verify deserialization.

use std::fs;

use clickup_api::models::{
    AuthenticatedUser, CommentsResponse, FoldersResponse, ListsResponse, SpacesResponse, Task,
    TasksResponse, WorkspacesResponse,
};

fn load_fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to load fixture {name}: {e}"))
}

#[test]
fn test_deserialize_user_fixture() {
    let json = load_fixture("user.json");
    let user: AuthenticatedUser = serde_json::from_str(&json).unwrap();

    assert_eq!(user.user.id, 12345678_i64);
    assert_eq!(user.user.username, "John Doe");
    assert_eq!(user.user.email, "john@example.com");
    assert_eq!(user.user.color.as_deref(), Some("#7B68EE"));
    assert_eq!(user.user.initials.as_deref(), Some("JD"));
    assert!(
        user.user.profile_picture.is_some(),
        "profilePicture should deserialize via serde alias"
    );
}

#[test]
fn test_deserialize_workspaces_fixture() {
    let json = load_fixture("workspaces.json");
    let resp: WorkspacesResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(resp.teams.len(), 1);
    let ws = &resp.teams[0];
    assert_eq!(ws.id, "team_001");
    assert_eq!(ws.name, "Acme Corp");
    assert_eq!(ws.members.len(), 2);
    assert_eq!(ws.members[0].user.username, "John Doe");
    assert_eq!(ws.members[1].user.username, "Jane Smith");
}

#[test]
fn test_deserialize_spaces_fixture() {
    let json = load_fixture("spaces.json");
    let resp: SpacesResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(resp.spaces.len(), 2);

    let eng = &resp.spaces[0];
    assert_eq!(eng.name, "Engineering");
    assert!(!eng.private);
    assert_eq!(eng.statuses.len(), 3);
    assert!(eng.multiple_assignees);
    assert!(eng.features.is_some());

    let design = &resp.spaces[1];
    assert_eq!(design.name, "Design");
    assert!(design.private);
    assert_eq!(design.statuses.len(), 2);
    assert_eq!(
        design.statuses[0].status_type, "open",
        "status type should be deserialized via #[serde(rename)]"
    );
}

#[test]
fn test_deserialize_folders_fixture() {
    let json = load_fixture("folders.json");
    let resp: FoldersResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(resp.folders.len(), 1);
    let folder = &resp.folders[0];
    assert_eq!(folder.name, "Backend");
    assert!(!folder.hidden);
    assert_eq!(folder.task_count.as_deref(), Some("15"));
    assert_eq!(folder.lists.len(), 1);
    assert_eq!(folder.lists[0].name, "Sprint 42");
}

#[test]
fn test_deserialize_lists_fixture() {
    let json = load_fixture("lists.json");
    let resp: ListsResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(resp.lists.len(), 2);
    assert_eq!(resp.lists[0].name, "Backlog");
    assert_eq!(resp.lists[0].task_count.as_deref(), Some("23"));
    assert_eq!(resp.lists[1].name, "Bug Fixes");
    assert_eq!(resp.lists[1].content.as_deref(), Some("Tracked bugs"));
}

#[test]
fn test_deserialize_tasks_fixture() {
    let json = load_fixture("tasks.json");
    let resp: TasksResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(resp.tasks.len(), 2);
    assert_eq!(resp.last_page, Some(true));

    let task1 = &resp.tasks[0];
    assert_eq!(task1.id, "task_abc123");
    assert_eq!(task1.name, "Implement user authentication");
    assert_eq!(task1.status.status, "In Progress");
    assert_eq!(task1.status.color, "#4194f6");
    assert_eq!(task1.assignees.len(), 1);
    assert_eq!(
        task1.priority.as_ref().unwrap().priority.as_deref(),
        Some("high")
    );
    assert!(task1.due_date.is_some());
    assert_eq!(task1.tags.len(), 1);
    assert_eq!(task1.tags[0].name, "backend");
    assert!(task1.markdown_description.is_some());
    assert!(task1.custom_fields.is_some());
    assert_eq!(task1.custom_fields.as_ref().unwrap().len(), 1);

    let task2 = &resp.tasks[1];
    assert_eq!(task2.custom_id.as_deref(), Some("ENG-42"));
    assert!(task2.assignees.is_empty());
    assert!(task2.priority.is_none());
    assert!(task2.due_date.is_none());
}

#[test]
fn test_deserialize_task_detail_fixture() {
    let json = load_fixture("task_detail.json");
    let task: Task = serde_json::from_str(&json).unwrap();

    assert_eq!(task.id, "task_abc123");
    assert_eq!(task.tags.len(), 2);

    // Subtasks
    let subtasks = task.subtasks.as_ref().unwrap();
    assert_eq!(subtasks.len(), 1);
    assert_eq!(subtasks[0].name, "Design auth schema");
    assert_eq!(subtasks[0].status.status, "Done");
    assert_eq!(subtasks[0].parent.as_deref(), Some("task_abc123"));

    // Custom fields
    let fields = task.custom_fields.as_ref().unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "Story Points");
    assert_eq!(fields[0].field_type, "number");
    assert_eq!(fields[1].name, "Component");
}

#[test]
fn test_deserialize_comments_fixture() {
    let json = load_fixture("comments.json");
    let resp: CommentsResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(resp.comments.len(), 2);
    assert_eq!(resp.comments[0].user.username, "John Doe");
    assert!(resp.comments[0].comment_text.contains("RS256"));
    assert_eq!(resp.comments[1].user.username, "Jane Smith");
}
