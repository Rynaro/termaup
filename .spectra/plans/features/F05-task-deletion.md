# F05 — Task Deletion

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 1 — Critical MVP Gaps
> **Complexity:** 4/12 | **Confidence:** 92%

---

## Problem Statement

Users cannot delete tasks from the terminal. The ClickUp API supports `DELETE /task/{task_id}`, but the `clickup-api` crate has no `delete_task()` endpoint. Neither the CLI nor the TUI offers task deletion. While less frequent than status changes, task deletion is a core CRUD operation needed for workflow management — cleaning up duplicates, removing test tasks, and archiving completed work.

## Approach

Follow the exact pattern of the existing `delete_comment()` implementation across all three crates. API: `self.delete(&format!("/task/{task_id}"))`. CLI: `clickup task delete TASK_ID` with `--yes` to skip confirmation (using `dialoguer::Confirm`). TUI: `D` (shift+d) on TaskDetail triggers a confirmation modal, then deletes and navigates back to TaskList. The `D` key (uppercase) is chosen over `d` to prevent accidental deletion — it requires holding Shift.

### Rejected Alternatives

1. **Soft-delete / archive instead of delete** — ClickUp supports archiving (`PUT /task/{id}` with `archived: true`) which is reversible, unlike deletion. However, users who ask for delete expect permanent removal. Archive could be offered as a separate command but should not replace delete. Rejected for not matching user intent; archive can be added as a separate `task archive` command later using the `update_task()` endpoint from F03.

2. **Delete from TaskList screen** — Allowing deletion directly from the list (without viewing the task first) increases the risk of deleting the wrong task. Requiring the user to be on TaskDetail ensures they've seen what they're deleting. Rejected for safety. The `D` key is intentionally NOT bound on TaskList.

3. **Multi-select deletion** — Selecting multiple tasks with checkboxes and batch-deleting is powerful but requires a selection mode, multi-select state, and batch API calls. Over-scoped for MVP. Deferred.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `delete_task()` API endpoint | Batch/multi-select deletion | Undo/restore after deletion |
| CLI `task delete TASK_ID [--yes]` with confirmation | Task archiving (separate feature) | Delete from TaskList screen |
| TUI `D` on TaskDetail with confirmation modal | Subtask cascade behavior (API handles this) | Recycle bin / trash view |
| Navigate back to TaskList + refresh after deletion | | |
| `wiremock` test for API endpoint | | |

## Stories

### S-1: delete_task() API endpoint

**As a** developer using clickup-api, **I want** a `delete_task()` method that DELETEs `/task/{task_id}`, **so that** CLI and TUI can delete tasks through a shared, tested interface.

**Timebox:** ≤1d | **Risk:** P0 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify | `crates/clickup-api/src/endpoints/tasks.rs` | Add `pub async fn delete_task(&self, task_id: &str) -> Result<()>` method. Implementation: `tracing::debug!(%task_id, "deleting task"); self.delete(&format!("/task/{task_id}")).await`. This is a 3-line method following the exact pattern of `delete_comment()` in `endpoints/comments.rs` (lines 53-57) |
| 2 | Test | `crates/clickup-api/src/endpoints/tasks.rs` or `crates/clickup-api/tests/` | `wiremock` test: mock `DELETE /task/abc123` → return 200 empty body, assert `delete_task("abc123")` returns `Ok(())`. Test error case: mock 404 → assert `ClickUpError::NotFound` |

#### Acceptance Criteria

- [ ] GIVEN a valid task ID WHEN `delete_task("abc123")` is called THEN it sends `DELETE /api/v2/task/abc123` and returns `Ok(())`
- [ ] GIVEN the API returns 404 WHEN `delete_task("nonexistent")` is called THEN it returns `Err(ClickUpError::NotFound)`
- [ ] GIVEN the API returns 401 WHEN `delete_task()` is called THEN it returns `Err(ClickUpError::AuthError)`
- [ ] GIVEN the API returns 429 (rate limited) WHEN `delete_task()` is called THEN the rate limiter handles the backoff transparently

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/endpoints/comments.rs` (lines 53-57 `delete_comment` — exact pattern to follow), `crates/clickup-api/src/client.rs` (`delete` method signature: `pub async fn delete(&self, path: &str) -> Result<()>`)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code; no `panic!()` in library code
  - [ ] P0: Crate boundary respected — `clickup-api` has no CLI/TUI deps
  - [ ] P1: `cargo test -p clickup-api` passes with new wiremock test
  - [ ] P1: New endpoint method has `wiremock` test
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` passes
  - [ ] P2: `///` doc comment on `delete_task()`

---

### S-2: CLI `task delete` command

**As a** CLI user, **I want** to run `clickup task delete TASK_ID` with a confirmation prompt (skippable with `--yes`), **so that** I can remove tasks from my terminal with appropriate safety guards.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify | `crates/clickup-cli/src/commands/tasks.rs` | Add `Delete` variant to `TaskCommands` enum: `Delete { /// The task ID to delete. task_id: String, /// Skip confirmation prompt. #[arg(long, default_value_t = false)] yes: bool }` |
| 2 | Modify | `crates/clickup-cli/src/commands/tasks.rs` | Add match arm in `run()` for `Self::Delete { task_id, yes }` → call `delete_task_cmd()` handler |
| 3 | Create | `crates/clickup-cli/src/commands/tasks.rs` | Implement `delete_task_cmd()` handler: if `!yes`, first call `client.get_task(&task_id)` to show the task name, then prompt with `dialoguer::Confirm::new().with_prompt(format!("Delete task '{}'?", task.name)).default(false).interact()?`. On confirm (or if `--yes`), call `client.delete_task(&task_id)`, print green `✓ Task {task_id} deleted`. On decline, print blue `ℹ Deletion cancelled` |
| 4 | Test | `crates/clickup-cli/src/commands/tasks.rs` | Test that `--yes` bypasses confirmation. Test output message format |

#### Acceptance Criteria

- [ ] GIVEN a valid task ID WHEN the user runs `clickup task delete TASK_ID` without `--yes` THEN the task name is displayed and a confirmation prompt asks `Delete task 'Task Name'? (y/n)` defaulting to no
- [ ] GIVEN the user confirms deletion WHEN the API call succeeds THEN the output shows `✓ Task TASK_ID deleted` with green prefix
- [ ] GIVEN the user declines deletion WHEN they respond `n` THEN the output shows `ℹ Deletion cancelled` with blue prefix and no API call is made
- [ ] GIVEN `--yes` flag is provided WHEN the user runs `clickup task delete TASK_ID --yes` THEN the task is deleted without any confirmation prompt
- [ ] GIVEN an invalid task ID WHEN the user runs `clickup task delete BAD_ID --yes` THEN a red `✗` error message is displayed on stderr

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (lines 9-38 `TaskCommands` enum), `crates/clickup-cli/src/commands/comments.rs` (lines 55-64 `Delete` variant as pattern, and the `delete_comment()` handler function)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo fmt --all -- --check` passes
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` passes

---

### S-3: TUI task deletion with confirmation modal

**As a** TUI user viewing a task, **I want** to press `D` (Shift+D) to trigger a confirmation dialog and delete the task, **so that** I can manage tasks entirely from the TUI with a safety guard against accidental deletion.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify | `crates/clickup-tui/src/app.rs` | Add `task_delete_confirm: bool` field to `App` struct. Add `task_delete_target_id: Option<String>` and `task_delete_target_name: Option<String>` for the confirmation modal to display. Add `open_task_delete_confirm(task_id: &str, task_name: &str)` and `close_task_delete_confirm()` helper methods |
| 2 | Modify | `crates/clickup-tui/src/data.rs` | Add `pub fn spawn_delete_task(client: &ClickUpClient, tx: &mpsc::UnboundedSender<AppEvent>, task_id: &str)` — calls `client.delete_task()`, sends a new `DataPayload::TaskDeleted(task_id)` on success |
| 3 | Modify | `crates/clickup-tui/src/event.rs` | Add `DataPayload::TaskDeleted(String)` variant (the String is the deleted task ID) |
| 4 | Modify | `crates/clickup-tui/src/input.rs` | Add early-return check for `app.task_delete_confirm` in `handle_key()` before screen dispatch: `y`/`Enter` = confirm (spawn delete, close confirm), `n`/`Esc` = cancel (close confirm). This follows the existing `delete_confirm_target` pattern used for comment deletion (input.rs lines 545-549). Add `D` (uppercase, `KeyCode::Char('D')`) key binding in `handle_task_detail()` when sidebar is closed: call `app.open_task_delete_confirm()` with the current task's ID and name |
| 5 | Modify | `crates/clickup-tui/src/ui/task_detail.rs` | Add confirmation modal rendering when `app.task_delete_confirm` is true: centered overlay box with text `Delete task '{name}'? (y/n)`, red-tinted border to signal destructive action. Follow the comment delete confirmation pattern |
| 6 | Modify | `crates/clickup-tui/src/main.rs` | Handle `DataPayload::TaskDeleted` event: remove the task from `app.tasks` list (if present), set `app.current_task = None`, navigate to `Screen::TaskList`, clear any task detail stack (from F04), pop breadcrumb back to list level, refresh task list via `spawn_load_tasks()` |

#### Acceptance Criteria

- [ ] GIVEN the user is on TaskDetail WHEN they press `D` (Shift+D) THEN a confirmation modal appears with the text `Delete task 'Task Name'? (y/n)` with a red-tinted border
- [ ] GIVEN the confirmation modal is open WHEN the user presses `y` or `Enter` THEN `spawn_delete_task()` is called and the modal closes
- [ ] GIVEN the confirmation modal is open WHEN the user presses `n` or `Esc` THEN the modal closes without deleting
- [ ] GIVEN the task is successfully deleted WHEN the response arrives THEN the app navigates back to TaskList, the task list refreshes, and the deleted task is no longer visible
- [ ] GIVEN the user is on TaskDetail with the comment sidebar open WHEN they press `D` THEN nothing happens (key is consumed by sidebar handler, not task deletion)
- [ ] GIVEN the confirmation modal is open WHEN any key other than `y`/`n`/`Enter`/`Esc` is pressed THEN nothing happens (keys are not leaked)
- [ ] GIVEN the user has drilled into a subtask (F04 stack) WHEN they delete the subtask THEN the stack is cleared and navigation returns to the TaskList (not the parent task, which may have stale subtask data)

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/input.rs` (lines 545-549 delete confirmation pattern for comments, lines 650-677 `d` key for comment deletion), `crates/clickup-tui/src/data.rs` (lines 107-128 `spawn_load_task_detail` as spawn pattern), `crates/clickup-tui/src/app.rs` (`delete_confirm_target` field as state pattern)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code; no `panic!()` in library code
  - [ ] P0: Main render thread is never blocked — deletion happens via `tokio::spawn`
  - [ ] P0: `D` key requires Shift (uppercase) to prevent accidental deletion
  - [ ] P0: Confirmation modal blocks all other key handlers
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` passes
  - [ ] P2: All new public items have `///` doc comments

## Execution Sequence

```
S-1 (API endpoint)
├── S-2 (CLI command)              [parallel]
└── S-3 (TUI confirmation + action) [parallel]
```

S-2 and S-3 are independent once S-1 provides the `delete_task()` method. They can be implemented in parallel.

## Assumptions

1. `DELETE /task/{task_id}` returns an empty body with HTTP 200 on success — Risk if wrong: the `self.delete()` method may need to handle a non-empty response body (LOW — `delete_comment()` already uses the same `self.delete()` method with empty response, and ClickUp docs confirm empty response)
2. Deleting a task also deletes its subtasks (server-side cascade) — Risk if wrong: orphaned subtasks remain visible until refresh (NEGLIGIBLE — ClickUp handles cascade server-side)
3. The `D` key (Shift+D) is not already bound on TaskDetail — Risk if wrong: key conflict (LOW — verified from input.rs, uppercase D is not bound; lowercase `d` is used for comment deletion in sidebar)
4. The rate limiter and error handling in `ClickUpClient` handle DELETE requests the same as other HTTP methods — Risk if wrong: need method-specific error handling (NEGLIGIBLE — `self.delete()` is a proven method used by `delete_comment()`)

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 1 | Small endpoint, simple CLI command, straightforward confirmation modal |
| Ambiguity | 1 | Exact pattern exists in `delete_comment()` for all three layers |
| Dependencies | 1 | No cross-feature dependencies; `delete` HTTP method already exists |
| Risk | 1 | Destructive operation but mitigated by confirmation in both CLI and TUI; follows proven pattern |
