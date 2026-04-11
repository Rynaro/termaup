# F22 — Bulk Task Operations

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 5 — Workspace Management
> **Complexity:** 7/12 | **Confidence:** 79%

---

## Problem Statement

When managing a sprint or triaging a backlog, users frequently need to perform the same action on multiple tasks: close 10 completed tasks, reassign 5 tasks to a new team member, or bulk-change status from "In Review" to "Done". Currently, each task requires individual navigation and action in the TUI, or separate CLI commands. This is tedious for batches larger than 2–3 tasks.

The ClickUp API v2 has **no bulk update endpoint** — each task must be updated individually via `PUT /task/{id}`. This means bulk operations require sequential (or concurrent) API calls, with proper error handling and progress feedback.

## Approach

1. **TUI multi-select:** On the TaskList screen, `Space` toggles task selection. Selected tasks are tracked in `selected_tasks: HashSet<String>` in the `App` struct. Visual indicator: checkbox `☑`/`☐` or highlight color on selected rows.

2. **Bulk action menu:** After selecting tasks, press `B` to open a bulk action popup menu with options: Change Status, Change Assignee, Delete. Each option opens the appropriate picker/dialog.

3. **Sequential execution with progress:** Bulk actions iterate over `selected_tasks`, calling `PUT /task/{id}` (or `DELETE /task/{id}`) for each. A progress indicator shows `"Updating 3/10..."` in the bottom bar. Failed tasks are collected and reported.

4. **CLI bulk command:** `clickup task bulk-update --tasks "ID1,ID2,ID3" --status "done"` or `--assignee USER_ID`. Also `--delete` for bulk deletion. Sequential execution with progress output.

### Rejected Alternatives

1. **Concurrent bulk API calls** — Sending all updates simultaneously would hit rate limits quickly (100 req/min on free plans). Sequential with rate-limit awareness is safer. Rejected for rate-limit risk.

2. **Local queue with retry** — Enqueue all changes locally and process with retries. Adds significant complexity (persistence, retry logic, conflict resolution). Rejected for scope.

3. **Select-all shortcut** — `Ctrl+A` to select all visible tasks. Dangerous for destructive operations. Can be added later with a confirmation step. Rejected for safety.

4. **Bulk via pipe** — `clickup task list --list X | clickup task bulk-update --status "done"`. Unix-philosophy approach but requires stable output format contracts. Deferred. Rejected for complexity.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| TUI `Space` to toggle task selection | Select-all (`Ctrl+A`) | Bulk move between lists |
| TUI `B` for bulk action menu | Concurrent API calls | Undo bulk operations |
| Bulk change status | Bulk custom field changes | Bulk tag operations |
| Bulk change assignee | Bulk due date changes | Pipe-based bulk operations |
| Bulk delete (with confirmation) | | Bulk priority changes |
| `selected_tasks: HashSet<String>` in App | | Bulk operations on search results |
| Progress indicator in bottom bar | | |
| CLI `clickup task bulk-update` command | | |
| Error collection and reporting | | |

## Stories

### S-1: TUI Multi-Select UI

**As a** TUI user on the TaskList screen, **I want** to press `Space` to toggle task selection and see which tasks are selected, **so that** I can prepare a batch of tasks for a bulk action.

**Timebox:** ≤2d | **Risk:** Low — state management + rendering change | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `selected_tasks: HashSet<String>` field | `crates/clickup-tui/src/app.rs` | Stores selected task IDs |
| 2 | Add `clear_selection(&mut self)` method | `crates/clickup-tui/src/app.rs` | Clears `selected_tasks`. Called on screen transition. |
| 3 | Add `toggle_task_selection(&mut self, task_id: &str)` method | `crates/clickup-tui/src/app.rs` | Insert if absent, remove if present |
| 4 | Add `Space` key handler on TaskList | `crates/clickup-tui/src/input.rs` | Get current task ID from `tasks[selected_index]`, call `toggle_task_selection()` |
| 5 | Update task rendering with selection indicator | `crates/clickup-tui/src/ui/task_list.rs` | Prepend `☑` (selected) or `☐` (unselected) to each task row when `selected_tasks` is non-empty. If no tasks are selected, render without checkboxes (normal mode). |
| 6 | Show selection count in bottom bar | `crates/clickup-tui/src/ui/mod.rs` | When `selected_tasks.len() > 0`: show `"{n} selected"` in the key hints area alongside `B Bulk action` |
| 7 | Clear selection on screen change | `crates/clickup-tui/src/input.rs` | Call `clear_selection()` when navigating away from TaskList (Enter to view detail, Esc to go back) |
| 8 | Update key hints | `crates/clickup-tui/src/ui/mod.rs` | Add `Space Select` to TaskList key hints. When tasks are selected, show `B Bulk action` and `Space Toggle` |

#### Acceptance Criteria

- **GIVEN** the user is on TaskList, **WHEN** they press `Space` on a task, **THEN** the task ID is added to `selected_tasks` and the task row shows `☑`.
- **GIVEN** a task is already selected, **WHEN** `Space` is pressed again, **THEN** the task is deselected (removed from set) and the row shows `☐`.
- **GIVEN** 3 tasks are selected, **WHEN** the bottom bar renders, **THEN** it shows `"3 selected"` alongside bulk action hints.
- **GIVEN** tasks are selected and the user presses `Enter` to view a task, **WHEN** they return to TaskList via Esc, **THEN** selection is preserved (not cleared by detail view).
- **GIVEN** tasks are selected and the user presses `Esc` to go back to SpaceContent, **WHEN** the screen changes, **THEN** selection is cleared.
- **GIVEN** no tasks are selected, **WHEN** TaskList renders, **THEN** no checkbox indicators are shown (normal rendering).

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-tasks`
- **Context:** `crates/clickup-tui/src/input.rs` (TaskList key handling lines 328–410), `crates/clickup-tui/src/ui/task_list.rs`, `crates/clickup-tui/src/app.rs`
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on `toggle_task_selection()`, `clear_selection()`

---

### S-2: Bulk Action Menu and Execution

**As a** TUI user with tasks selected, **I want** to press `B` to open a bulk action menu, choose an action, and have it applied to all selected tasks with progress feedback, **so that** I can efficiently perform batch operations.

**Timebox:** ≤3d | **Risk:** Medium — sequential API calls with error aggregation | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add bulk action state | `crates/clickup-tui/src/app.rs` | `bulk_action_menu_open: bool`, `bulk_action_selected: usize`, `bulk_progress: Option<(usize, usize)>` (current, total), `bulk_errors: Vec<(String, String)>` (task_id, error_msg) |
| 2 | Add `BulkAction` enum | `crates/clickup-tui/src/app.rs` | `pub enum BulkAction { ChangeStatus, ChangeAssignee, Delete }` |
| 3 | Add `B` key handler | `crates/clickup-tui/src/input.rs` | When `selected_tasks.len() > 0`: open bulk action menu |
| 4 | Handle menu navigation | `crates/clickup-tui/src/input.rs` | `↑/↓` navigate options, `Enter` selects action, `Esc` closes menu |
| 5 | Change Status flow | `crates/clickup-tui/src/input.rs` | On select: show status picker (reuse existing status list from task filters or fetch from space). On pick: call `spawn_bulk_update_status()` |
| 6 | Change Assignee flow | `crates/clickup-tui/src/input.rs` | On select: show member picker. On pick: call `spawn_bulk_update_assignee()` |
| 7 | Delete flow | `crates/clickup-tui/src/input.rs` | On select: show confirmation `"Delete {n} tasks? (y/n)"`. On confirm: call `spawn_bulk_delete()` |
| 8 | Create `spawn_bulk_update_status()` | `crates/clickup-tui/src/data.rs` | Iterates `selected_tasks`, calls `PUT /task/{id}` with `{"status": "..."}` for each. Sends progress events `DataPayload::BulkProgress(current, total)`. On completion: sends `DataPayload::BulkComplete(errors)`. |
| 9 | Create `spawn_bulk_update_assignee()` | `crates/clickup-tui/src/data.rs` | Same pattern as status but with `{"assignees": {"add": [user_id]}}` body |
| 10 | Create `spawn_bulk_delete()` | `crates/clickup-tui/src/data.rs` | Iterates `selected_tasks`, calls `DELETE /task/{id}` for each with progress events |
| 11 | Add `DataPayload::BulkProgress` variant | `crates/clickup-tui/src/app.rs` | Carries `(usize, usize)` — current index, total count |
| 12 | Add `DataPayload::BulkComplete` variant | `crates/clickup-tui/src/app.rs` | Carries `Vec<(String, String)>` — failed task IDs with error messages |
| 13 | Handle progress events | `crates/clickup-tui/src/app.rs` | Update `bulk_progress`. On complete: clear selection, refresh task list, show flash `"✓ Updated {n} tasks"` or `"✓ Updated {n} tasks, {m} failed"` |
| 14 | Render bulk action menu | `crates/clickup-tui/src/ui/mod.rs` | Centered popup with options list, highlight selected |
| 15 | Render progress in bottom bar | `crates/clickup-tui/src/ui/mod.rs` | When `bulk_progress` is set: show `"Updating {current}/{total}..."` with animated spinner |

#### Acceptance Criteria

- **GIVEN** 5 tasks are selected, **WHEN** `B` is pressed, **THEN** a popup menu shows "Change Status", "Change Assignee", "Delete".
- **GIVEN** "Change Status" is selected and status "Done" is picked, **WHEN** the bulk update runs, **THEN** each task is updated sequentially, the progress shows `"Updating 1/5..."`, `"Updating 2/5..."`, etc., and on completion `"✓ Updated 5 tasks"` appears.
- **GIVEN** 2 out of 5 tasks fail (e.g., permission error), **WHEN** bulk update completes, **THEN** the flash message shows `"✓ Updated 3 tasks, 2 failed"`.
- **GIVEN** "Delete" is selected, **WHEN** the confirmation prompt appears and user presses `y`, **THEN** all selected tasks are deleted and removed from the task list.
- **GIVEN** no tasks are selected, **WHEN** `B` is pressed, **THEN** nothing happens.
- **GIVEN** the bulk operation is in progress, **WHEN** the user presses any key, **THEN** the bulk operation is not interrupted (keys are blocked during bulk execution).

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-tasks`
- **Context:** `crates/clickup-tui/src/input.rs`, `crates/clickup-tui/src/data.rs` (existing spawn pattern), `crates/clickup-tui/src/app.rs`, `crates/clickup-tui/src/ui/mod.rs`
- **Note:** The `PUT /task/{id}` endpoint may not yet exist (depends on F06). If not available, this story should use a `put` call directly: `self.put::<Task, _>(&format!("/task/{task_id}"), &body)`. The body shape is `{"status": "done"}` for status updates and `{"assignees": {"add": [id]}}` for assignee updates.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on `BulkAction`, `spawn_bulk_update_status()`, etc.

---

### S-3: CLI Bulk Update Command

**As a** CLI user, **I want** `clickup task bulk-update --tasks "ID1,ID2,ID3" --status "done"` to update multiple tasks in one command, **so that** I can script batch operations.

**Timebox:** ≤2d | **Risk:** Low — sequential API calls with output | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `BulkUpdate` variant to `TaskCommands` | `crates/clickup-cli/src/commands/tasks.rs` | `BulkUpdate { #[arg(long, value_delimiter = ',')] tasks: Vec<String>, #[arg(long)] status: Option<String>, #[arg(long)] assignee: Option<String>, #[arg(long)] delete: bool, #[arg(long)] yes: bool }` |
| 2 | Validate inputs | `crates/clickup-cli/src/commands/tasks.rs` | At least one of `--status`, `--assignee`, or `--delete` must be provided. `--delete` requires `--yes` or interactive confirmation. |
| 3 | Implement sequential update | `crates/clickup-cli/src/commands/tasks.rs` | For each task ID: build update body, call `PUT /task/{id}` (or `DELETE`), print `"✓ {action} task {id}"` on success, `"✗ Failed: {id}: {error}"` on failure |
| 4 | Implement progress output | `crates/clickup-cli/src/commands/tasks.rs` | Print `"[{i}/{total}] Updating task {id}..."` before each call |
| 5 | Summary output | `crates/clickup-cli/src/commands/tasks.rs` | At end: `"✓ {success_count}/{total} tasks updated"`. If failures: `"✗ {fail_count} tasks failed"` followed by list of failed task IDs |
| 6 | Handle `--delete` + `--yes` | `crates/clickup-cli/src/commands/tasks.rs` | If `--delete` without `--yes`: prompt `"Delete {n} tasks? This cannot be undone. (y/n)"` |

#### Acceptance Criteria

- **GIVEN** 3 task IDs and `--status "done"`, **WHEN** running `clickup task bulk-update --tasks "abc,def,ghi" --status "done"`, **THEN** each task is updated sequentially with progress output and a summary at the end.
- **GIVEN** 1 out of 3 tasks fails, **WHEN** the command completes, **THEN** the summary shows `"✓ 2/3 tasks updated"` and lists the failed task ID.
- **GIVEN** `--delete` without `--yes`, **WHEN** the command runs, **THEN** a confirmation prompt appears.
- **GIVEN** `--delete --yes`, **WHEN** the command runs, **THEN** tasks are deleted without prompting.
- **GIVEN** no action flag provided (no `--status`, `--assignee`, or `--delete`), **WHEN** the command runs, **THEN** an error `"✗ Specify at least one action: --status, --assignee, or --delete"` is printed.

#### Agent Hints

- **Class:** builder
- **Skill:** `cli-data`
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (TaskCommands enum, handler pattern)
- **Note:** `PUT /task/{id}` may not be implemented yet (depends on F06). Use `client.put::<serde_json::Value, _>(path, &body)` directly if `update_task()` doesn't exist.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new subcommand

---

### S-4: Bulk Execution Tests and Error Handling

**As a** maintainer, **I want** tests covering bulk operation edge cases (partial failures, empty selection, rate limiting), **so that** bulk operations are robust.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-2, S-3

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Unit test: empty selection | `crates/clickup-tui/src/app.rs` | Verify `B` key does nothing when `selected_tasks` is empty |
| 2 | Unit test: toggle selection | `crates/clickup-tui/src/app.rs` | Test `toggle_task_selection()` add/remove behavior |
| 3 | Unit test: clear on screen change | `crates/clickup-tui/src/app.rs` | Test `clear_selection()` empties the set |
| 4 | Integration test: partial failure | `crates/clickup-api/tests/` | Mock 3 `PUT /task` calls: 2 succeed, 1 returns 404. Verify error collection. |
| 5 | Integration test: rate limit during bulk | `crates/clickup-api/tests/` | Mock `PUT /task` returning 429 on second call. Verify rate limiter pauses and retries. |

#### Acceptance Criteria

- **GIVEN** `selected_tasks` is empty, **WHEN** `toggle_task_selection("abc")` is called, **THEN** `selected_tasks` contains `"abc"`.
- **GIVEN** `selected_tasks` contains `"abc"`, **WHEN** `toggle_task_selection("abc")` is called, **THEN** `selected_tasks` is empty.
- **GIVEN** a bulk update of 3 tasks where the second returns 404, **WHEN** the operation completes, **THEN** errors list contains one entry for the failed task and the other 2 succeed.

#### Agent Hints

- **Class:** builder
- **Skill:** `docs-release`
- **Context:** `crates/clickup-tui/src/app.rs`, `crates/clickup-api/tests/`
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo clippy` clean

---

## Execution Sequence

```
S-1 (multi-select UI)  ──→ S-2 (bulk action menu + execution) ──→ S-4 (tests)
S-3 (CLI bulk command)                                          ──┘
```

S-1 must be done first (S-2 depends on it). S-3 is independent of S-1/S-2. S-4 depends on both S-2 and S-3.

## Assumptions

1. `PUT /task/{id}` is available as an API endpoint (either from F06 or via direct `client.put()` call). **Risk if wrong:** If F06 is not implemented yet, the story explicitly notes to use `client.put()` directly with a raw JSON body. This is a temporary workaround.
2. Sequential API calls are acceptable for bulk operations (no bulk API exists). **Risk if wrong:** None — this is a ClickUp API limitation.
3. The rate limiter in `ClickUpClient` automatically handles 429 responses during sequential calls. **Risk if wrong:** The rate limiter is already implemented and handles `X-RateLimit-Remaining` / `X-RateLimit-Reset` headers.
4. Bulk operations on 50+ tasks may take 30+ seconds. Progress feedback is essential. **Risk if wrong:** Progress display is part of the design.
5. The ClickUp API accepts partial task update bodies (`{"status": "done"}` without other fields). **Risk if wrong:** API docs confirm partial updates are supported.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Two crates (CLI, TUI), complex state management in TUI |
| Ambiguity | 2 | Bulk operation UX has open questions (progress display, error recovery) |
| Dependencies | 1 | May depend on F06 for `PUT /task/{id}`, but has fallback |
| Risk | 2 | Rate limiting during bulk ops, partial failure handling, concurrent state |

**Total: 7/12** → Standard-plus processing
