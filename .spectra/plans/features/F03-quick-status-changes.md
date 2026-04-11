# F03 — Quick Status Changes

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 1 — Critical MVP Gaps
> **Complexity:** 5/12 | **Confidence:** 88%

---

## Problem Statement

Users can view task statuses but cannot change them without opening the ClickUp web app. Status transitions are the most frequent task mutation in project management — moving a task from "to do" to "in progress" to "done." The `clickup-api` crate lacks a `PUT /task/{task_id}` endpoint entirely, which blocks not only status changes but all future task update features (priority changes, reassignment, due date edits). This story creates the foundational `update_task()` endpoint and applies it to the most common use case: status changes.

## Approach

Build a general-purpose `UpdateTaskRequest` model with all-optional fields using `#[serde(skip_serializing_if = "Option::is_none")]`, then expose a focused `task status` CLI command and a TUI status picker overlay. The `update_task()` API endpoint is deliberately general (it supports updating any task field) even though this feature only uses the `status` field — this ensures F06 (priority changes), F07 (reassignment), and F08 (due date edits) can reuse it without modification.

### Rejected Alternatives

1. **Status-only API method** (`update_task_status(task_id, status)`) — Would require a separate method for every field update (priority, assignee, etc.). The ClickUp API uses a single `PUT /task/{task_id}` for all updates. One general endpoint matches the API surface and avoids method proliferation. Rejected for future maintenance cost.

2. **Builder pattern for UpdateTaskRequest** — `UpdateTaskRequest::new().status("done").priority(2).build()` — Ergonomic but adds complexity for a struct that works perfectly well with struct initialization and `..Default::default()`. The `skip_serializing_if` approach already handles partial updates cleanly. Rejected for over-engineering.

3. **Inline status cycling in TUI** (press `s` to cycle through statuses in order) — Fast but error-prone: users can't see what they're selecting, and ClickUp spaces can have 10+ custom statuses with non-obvious ordering. A picker overlay with the full status list is safer. Rejected for poor UX.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `UpdateTaskRequest` model (all fields optional) | Custom status creation/deletion | Priority changes via `update_task()` (F06) |
| `update_task()` API endpoint (`PUT /task/{task_id}`) | Workflow automation / status triggers | Assignee changes via `update_task()` (F07) |
| CLI `task status TASK_ID --status "done"` command | Bulk status updates | Due date changes via `update_task()` (F08) |
| TUI status picker on TaskList and TaskDetail (`s` key) | Status transition validation | Status change history/audit log |
| Picker shows statuses from current space | | |
| Refresh task data after status change | | |

## Stories

### S-1: UpdateTaskRequest model and update_task() endpoint

**As a** developer using clickup-api, **I want** an `update_task()` method that PUTs to `/task/{task_id}`, **so that** CLI and TUI can modify task fields through a shared, tested interface.

**Timebox:** ≤2d | **Risk:** P0 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/src/models/task.rs` | Add `UpdateTaskRequest` struct deriving `Debug, Clone, Default, Serialize, Deserialize`. Fields (all `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]`): `name`, `description`, `markdown_description`, `status` (String), `priority` (integer — ClickUp uses `Option<u8>` where `None` means "no priority"), `assignees` (`Option<UpdateAssignees>`), `due_date` (ms timestamp string), `due_date_time` (bool), `start_date`, `start_date_time`, `time_estimate` (ms), `parent` (String — move to subtask), `archived` (bool) |
| 2 | Create | `crates/clickup-api/src/models/task.rs` | Add `UpdateAssignees` struct: `add: Vec<i64>`, `rem: Vec<i64>` — both with `#[serde(skip_serializing_if = "Vec::is_empty", default)]` — for additive/subtractive assignee updates |
| 3 | Create | `crates/clickup-api/src/endpoints/tasks.rs` | Add `pub async fn update_task(&self, task_id: &str, request: &UpdateTaskRequest) -> Result<Task>` — call `self.put(&format!("/task/{task_id}"), request)` with `tracing::debug!` logging |
| 4 | Test | `crates/clickup-api/src/models/task.rs` | Serialization tests: verify empty `UpdateTaskRequest::default()` serializes to `{}`, verify single-field updates serialize only that field, verify `UpdateAssignees` with both add and rem |
| 5 | Test | `crates/clickup-api/tests/` or inline | `wiremock` test: mock `PUT /task/abc123` → assert request body, assert response deserializes to `Task` |

#### Acceptance Criteria

- [ ] GIVEN an `UpdateTaskRequest` with only `status` set to `Some("done".to_string())` WHEN serialized THEN the JSON is `{"status": "done"}` with no other fields
- [ ] GIVEN an `UpdateTaskRequest::default()` WHEN serialized THEN the JSON is `{}`
- [ ] GIVEN an `UpdateTaskRequest` with `assignees` set to `UpdateAssignees { add: vec![123], rem: vec![456] }` WHEN serialized THEN the JSON contains `{"assignees": {"add": [123], "rem": [456]}}`
- [ ] GIVEN a valid task ID and request WHEN `update_task()` is called THEN it PUTs to `/task/{task_id}` and returns a deserialized `Task`
- [ ] GIVEN the API returns a 404 WHEN `update_task()` is called THEN it returns `ClickUpError::NotFound`

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/endpoints/comments.rs` (lines 39-47 `update_comment` as PUT pattern), `crates/clickup-api/src/models/comment.rs` (lines 225-234 `UpdateCommentRequest` as model pattern), `crates/clickup-api/src/client.rs` (`put` method signature)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code; no `panic!()` in library code
  - [ ] P0: Crate boundary respected — `clickup-api` has no CLI/TUI deps
  - [ ] P1: `cargo test -p clickup-api` passes with new wiremock tests
  - [ ] P1: New endpoint method has `wiremock` test
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` passes
  - [ ] P2: `///` doc comments on `update_task()`, `UpdateTaskRequest`, and `UpdateAssignees`

---

### S-2: CLI `task status` command

**As a** CLI user, **I want** to run `clickup task status TASK_ID --status "done"` to change a task's status, **so that** I can move tasks through my workflow without opening the web app.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify | `crates/clickup-cli/src/commands/tasks.rs` | Add `Status` variant to `TaskCommands` enum: `Status { task_id: String, #[arg(long)] status: String }` |
| 2 | Modify | `crates/clickup-cli/src/commands/tasks.rs` | Add match arm in `run()` for `Self::Status { task_id, status }` → call `update_status()` handler |
| 3 | Create | `crates/clickup-cli/src/commands/tasks.rs` | Implement `update_status()` handler: build `UpdateTaskRequest { status: Some(status), ..Default::default() }`, call `client.update_task(&task_id, &request)`, print success with green `✓` prefix showing old → new status, or format as json/markdown per `--format` flag |
| 4 | Test | `crates/clickup-cli/src/commands/tasks.rs` | Test that the command constructs the correct `UpdateTaskRequest` with only the status field set |

#### Acceptance Criteria

- [ ] GIVEN a valid task ID WHEN the user runs `clickup task status TASK_ID --status "in progress"` THEN the task status is updated and the output shows `✓ Task TASK_ID status changed to "in progress"` with green prefix
- [ ] GIVEN `--format json` WHEN the status is changed THEN the full updated task JSON is printed to stdout
- [ ] GIVEN an invalid task ID WHEN the user runs `clickup task status BAD_ID --status "done"` THEN a red `✗` error message is displayed on stderr
- [ ] GIVEN a status name that doesn't exist in the space WHEN the user attempts to change status THEN the ClickUp API returns an error and the CLI displays it clearly

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (lines 9-38 `TaskCommands` enum, `Get` and `View` variants as subcommand pattern)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo fmt --all -- --check` passes
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` passes

---

### S-3: TUI status picker overlay

**As a** TUI user viewing a task, **I want** to press `s` to see a floating list of available statuses and select one to change the task's status, **so that** I can manage task workflow interactively.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify | `crates/clickup-tui/src/app.rs` | Add status picker state to `App`: `status_picker_open: bool`, `status_picker_items: Vec<Status>`, `status_picker_selected: usize`, `status_picker_task_id: Option<String>`. Add `open_status_picker()` method that populates `status_picker_items` from `app.current_space.statuses` and sets the task ID. Add `close_status_picker()` reset method |
| 2 | Modify | `crates/clickup-tui/src/event.rs` | Add `DataPayload::TaskUpdated(Box<Task>)` variant for handling the async update response |
| 3 | Modify | `crates/clickup-tui/src/data.rs` | Add `pub fn spawn_update_task_status(client: &ClickUpClient, tx: &mpsc::UnboundedSender<AppEvent>, task_id: &str, status: &str)` — builds minimal `UpdateTaskRequest { status: Some(status.to_string()), ..Default::default() }`, calls `client.update_task()`, sends `TaskUpdated` on success |
| 4 | Modify | `crates/clickup-tui/src/input.rs` | Add early-return check for `status_picker_open` in `handle_key()` (before screen dispatch). Add `handle_status_picker()`: `j`/`Down` = next, `k`/`Up` = prev, `Enter` = confirm (spawn update, close picker), `Esc` = cancel. Bind `s` key in `handle_task_list()` and `handle_task_detail()` to open picker when sidebar is not open and no other modal is active |
| 5 | Create | `crates/clickup-tui/src/ui/status_picker.rs` | Render function: centered floating panel (40% width, min 5 rows + 2 border), title "Change Status", list of statuses with color-coded bullets (using status color from `Status.color`), highlight selected item, current status marked with `✓`, bottom hint: `[Enter] Select  [Esc] Cancel` |
| 6 | Modify | `crates/clickup-tui/src/ui/mod.rs` | Add `pub mod status_picker;` export, render picker overlay when `app.status_picker_open` is true |
| 7 | Modify | `crates/clickup-tui/src/main.rs` | Handle `DataPayload::TaskUpdated` event: update task in `app.tasks` list (or `app.current_task` if on TaskDetail), show brief success message, auto-close picker |

#### Acceptance Criteria

- [ ] GIVEN the user is on TaskList with a task selected WHEN they press `s` THEN a floating status picker appears showing all statuses from the current space, each with its color
- [ ] GIVEN the status picker is open WHEN the user navigates with `j`/`k` and presses `Enter` THEN the selected task's status is updated via the API and the picker closes
- [ ] GIVEN the status picker is open WHEN the user presses `Esc` THEN the picker closes without making any changes
- [ ] GIVEN the user is on TaskDetail WHEN they press `s` (with sidebar closed) THEN the status picker opens for the currently displayed task
- [ ] GIVEN a status update succeeds WHEN the response arrives THEN the task's status is visually updated in the list/detail view without a full data reload
- [ ] GIVEN the comment sidebar is open WHEN the user presses `s` THEN the status picker does NOT open (key is consumed by sidebar handler)
- [ ] GIVEN the status picker is open WHEN the user presses any key other than `j`/`k`/`Enter`/`Esc`/`Up`/`Down` THEN nothing happens (keys are not leaked to the screen behind)

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/help.rs` (overlay rendering pattern — centered floating panel with borders), `crates/clickup-tui/src/input.rs` (lines 22-31 `filter_panel_open`/`filter_active` early-return pattern for modal key capture), `crates/clickup-tui/src/app.rs` (lines 148-276 for state field patterns), `crates/clickup-tui/src/data.rs` (lines 107-128 `spawn_load_task_detail` as spawn pattern)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code; no `panic!()` in library code
  - [ ] P0: Main render thread is never blocked — status update happens via `tokio::spawn`
  - [ ] P0: Crate boundary respected — `clickup-api` has no CLI/TUI deps
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` passes
  - [ ] P2: All public items have `///` doc comments

## Execution Sequence

```
S-1 (API model + endpoint)
├── S-2 (CLI command)          [parallel]
└── S-3 (TUI status picker)    [parallel]
```

S-2 and S-3 are independent once S-1 provides the API endpoint. They can be implemented in parallel.

## Assumptions

1. Space statuses are available in `app.current_space.statuses` (the `Space` model captures statuses from the API) — Risk if wrong: need an additional `GET /space/{id}` call to fetch statuses, adding a round-trip (MEDIUM — verify Space model has statuses field)
2. `PUT /task/{task_id}` with `{"status": "done"}` returns the full updated `Task` object — Risk if wrong: need a follow-up `GET /task/{id}` to refresh data (LOW — ClickUp docs confirm full task response)
3. Status names are case-sensitive in the ClickUp API — Risk if wrong: case-insensitive matching may silently fail; document this for CLI users (LOW — test will verify)
4. The `s` key is not already bound on TaskList or TaskDetail screens — Risk if wrong: key conflict with existing functionality (LOW — verified from input.rs, `s` is unbound on both screens when sidebar is closed)

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Touches 3 crates, creates new model + endpoint + CLI command + TUI overlay; but each is small |
| Ambiguity | 1 | API endpoint is well-documented; status picker UI is a standard list selection |
| Dependencies | 1 | S-1 is the only blocker; `update_task()` creates value for future features (F06-F08) |
| Risk | 1 | All patterns have close exemplars; `put` method exists; overlay pattern proven in help.rs |
