# F18 — Quick-Jump Navigation

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 4 — Navigation & Content
> **Complexity:** 5/12 | **Confidence:** 88%

---

## Problem Statement

TUI users who know a task ID (from a Slack message, Git branch name, or ClickUp notification) must navigate through the full workspace → space → folder → list hierarchy to find and view the task. This is unnecessarily slow for direct access. Similarly, there's no way to jump directly to a known list by ID.

Additionally, users frequently revisit recent tasks but must re-navigate the hierarchy each time. There is no "recent tasks" history to enable quick access to previously viewed items.

## Approach

Three complementary navigation shortcuts:

1. **Go-to-task dialog (`g`):** Press `g` on any screen → modal text input → type task ID → `GET /task/{task_id}` → jump directly to TaskDetail. Uses the existing `get_task()` endpoint.

2. **Go-to-list dialog (`G`):** Press `G` on any screen → modal text input → type list ID → `GET /list/{list_id}` + load tasks → jump to TaskList screen. Uses the existing `get_list()` and `get_tasks()` endpoints.

3. **Recent tasks picker (`Ctrl+R`):** Maintain a `recent_tasks: Vec<(String, String)>` (id, name) in `App` — last 10 visited tasks. Press `Ctrl+R` → popup list of recent tasks → select → jump to TaskDetail. No API call needed if the task is already in memory.

### Rejected Alternatives

1. **URL bar input** — A persistent "address bar" at the top for entering ClickUp URLs. Over-engineered for the use case. `g` dialog is simpler. Rejected for complexity.

2. **Bookmark system** — Persistent bookmarks saved to config file. Interesting but larger scope. Deferred to a future feature. Rejected for scope.

3. **Merge go-to and search into one dialog** — Combining task ID input with name search in one overlay. Conflates two use cases with different UX requirements. F17 handles search. Rejected for UX clarity.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `g` → go-to-task dialog (task ID input) | Go-to by ClickUp URL | Persistent bookmarks |
| `G` → go-to-list dialog (list ID input) | Go-to by task custom_id | Configurable max history size |
| `Ctrl+R` → recent tasks picker | Go-to space/folder by ID | Recent lists history |
| `recent_tasks: Vec<(String, String)>` (last 10) | Search by task name (that's F17) | Persistent history across sessions |
| Breadcrumb update on jump | | |

## Stories

### S-1: Go-To-Task Dialog

**As a** TUI user, **I want** to press `g` to open a "Go to task" dialog, type a task ID, and jump directly to its TaskDetail, **so that** I can access any task instantly when I know its ID.

**Timebox:** ≤2d | **Risk:** Low — simple dialog + existing API call | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add dialog state fields | `crates/clickup-tui/src/app.rs` | `goto_dialog_open: bool`, `goto_dialog_kind: GotoKind` (enum: `Task`, `List`), `goto_input: String` |
| 2 | Add `GotoKind` enum | `crates/clickup-tui/src/app.rs` | `pub enum GotoKind { Task, List }` |
| 3 | Add `g` key handler | `crates/clickup-tui/src/input.rs` | Global (not in filter/compose mode): set `goto_dialog_open = true`, `goto_dialog_kind = GotoKind::Task`, clear `goto_input` |
| 4 | Add dialog input handling | `crates/clickup-tui/src/input.rs` | When dialog is open: `Char(c)` appends to `goto_input`, `Backspace` removes, `Enter` triggers task load, `Esc` closes dialog |
| 5 | On Enter: load task | `crates/clickup-tui/src/input.rs` | Call `spawn_load_task_detail(goto_input.trim())`, close dialog, set `loading = true` |
| 6 | Handle loaded task | `crates/clickup-tui/src/app.rs` | On `DataPayload::TaskDetail` arrival while goto was active: set screen to `TaskDetail`, push breadcrumb `"ClickUp › {task.name}"` (simplified breadcrumb since we're jumping) |
| 7 | Handle task not found | `crates/clickup-tui/src/app.rs` | On `DataPayload::Error` with 404: show flash message `"✗ Task not found: {id}"`, stay on current screen |
| 8 | Render go-to dialog | `crates/clickup-tui/src/ui/mod.rs` or `crates/clickup-tui/src/ui/goto.rs` | Centered small popup: title "Go to Task", text input field showing current input, hint "Enter task ID" |
| 9 | Update key hints | `crates/clickup-tui/src/ui/mod.rs` | Add `g Go to task` hint on all main screens |

#### Acceptance Criteria

- **GIVEN** the user is on any screen (not in filter/compose mode), **WHEN** they press `g`, **THEN** a "Go to Task" dialog appears with a text input.
- **GIVEN** the dialog is open and the user types `abc123` and presses Enter, **WHEN** the task exists, **THEN** the dialog closes, loading indicator appears, and TaskDetail for task `abc123` loads with breadcrumb `"ClickUp › {task name}"`.
- **GIVEN** the dialog is open and the user types a non-existent task ID, **WHEN** the API returns 404, **THEN** a flash message `"✗ Task not found: {id}"` appears and the user stays on the current screen.
- **GIVEN** the dialog is open, **WHEN** the user presses Esc, **THEN** the dialog closes without action.
- **GIVEN** the comment sidebar is open or user is composing a comment, **WHEN** `g` is pressed, **THEN** nothing happens (key is not intercepted).

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-polish`
- **Context:** `crates/clickup-tui/src/input.rs` (global key handling lines 44–87), `crates/clickup-tui/src/app.rs` (App struct), `crates/clickup-tui/src/data.rs` (`spawn_load_task_detail` line 107)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on `GotoKind`

---

### S-2: Recent Tasks History and Picker

**As a** TUI user, **I want** to press `Ctrl+R` to see my 10 most recently viewed tasks and select one to jump back to, **so that** I can quickly revisit tasks without re-navigating.

**Timebox:** ≤2d | **Risk:** Low — simple state tracking + popup list | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `recent_tasks: Vec<(String, String)>` field | `crates/clickup-tui/src/app.rs` | Stores `(task_id, task_name)`, most recent first, max 10 entries |
| 2 | Add `push_recent_task(&mut self, id: &str, name: &str)` method | `crates/clickup-tui/src/app.rs` | Deduplicates (removes existing entry with same ID before adding), maintains max 10 |
| 3 | Record task visits | `crates/clickup-tui/src/app.rs` | In `DataPayload::TaskDetail` handler, call `push_recent_task()` when a task detail loads successfully |
| 4 | Add recent picker state | `crates/clickup-tui/src/app.rs` | `recent_picker_open: bool`, `recent_picker_index: usize` |
| 5 | Add `Ctrl+R` key handler | `crates/clickup-tui/src/input.rs` | Global: if `recent_tasks` is non-empty, set `recent_picker_open = true`, `recent_picker_index = 0` |
| 6 | Handle picker navigation | `crates/clickup-tui/src/input.rs` | When picker open: `↑/k` moves up, `↓/j` moves down, `Enter` selects and loads task, `Esc` closes |
| 7 | On Enter: load selected task | `crates/clickup-tui/src/input.rs` | Get task ID from `recent_tasks[recent_picker_index]`, call `spawn_load_task_detail()` |
| 8 | Render recent tasks picker | `crates/clickup-tui/src/ui/mod.rs` or `crates/clickup-tui/src/ui/goto.rs` | Centered popup: title "Recent Tasks", list of task names with index numbers, highlight selected |
| 9 | Update key hints | `crates/clickup-tui/src/ui/mod.rs` | Add `Ctrl+R Recent` hint on all main screens |

#### Acceptance Criteria

- **GIVEN** the user has visited 5 tasks in the session, **WHEN** they press `Ctrl+R`, **THEN** a picker shows the 5 tasks with the most recently visited at the top.
- **GIVEN** the picker is open, **WHEN** the user selects a task and presses Enter, **THEN** the picker closes and the selected task's TaskDetail loads.
- **GIVEN** the user has visited no tasks yet, **WHEN** `Ctrl+R` is pressed, **THEN** nothing happens (picker doesn't open for empty history).
- **GIVEN** the user visits the same task twice, **WHEN** `Ctrl+R` is pressed, **THEN** the task appears only once in the list (deduplicated), at the most recent position.
- **GIVEN** the user has visited 12 tasks, **WHEN** `Ctrl+R` is pressed, **THEN** only the 10 most recent are shown.

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-polish`
- **Context:** `crates/clickup-tui/src/app.rs` (App struct ~line 148), `crates/clickup-tui/src/input.rs`
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on `push_recent_task()`

---

### S-3: Go-To-List Dialog

**As a** TUI user, **I want** to press `G` to open a "Go to list" dialog, type a list ID, and jump directly to the TaskList screen for that list, **so that** I can access any list instantly when I know its ID.

**Timebox:** ≤1d | **Risk:** Low — mirrors S-1 pattern | **Depends on:** S-1 (shares dialog infrastructure)

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `G` key handler | `crates/clickup-tui/src/input.rs` | Global: set `goto_dialog_open = true`, `goto_dialog_kind = GotoKind::List` |
| 2 | Handle Enter for list goto | `crates/clickup-tui/src/input.rs` | When `GotoKind::List`: call `spawn_load_list_and_tasks()` (new helper) |
| 3 | Create `spawn_load_list_and_tasks()` | `crates/clickup-tui/src/data.rs` | Calls `client.get_list(list_id)` then `spawn_load_tasks(list_id)`. Sends `DataPayload::ListDetail(list)` followed by tasks. |
| 4 | Add `DataPayload::ListDetail` variant | `crates/clickup-tui/src/app.rs` | On receipt: set `current_list`, switch screen to `TaskList`, push breadcrumb `"ClickUp › {list.name}"` |
| 5 | Handle list not found | `crates/clickup-tui/src/app.rs` | On 404: flash message `"✗ List not found: {id}"` |
| 6 | Update dialog title | `crates/clickup-tui/src/ui/mod.rs` or `crates/clickup-tui/src/ui/goto.rs` | Show "Go to Task" or "Go to List" based on `goto_dialog_kind` |
| 7 | Update key hints | `crates/clickup-tui/src/ui/mod.rs` | Add `G Go to list` hint |

#### Acceptance Criteria

- **GIVEN** the user presses `G`, **WHEN** the dialog opens, **THEN** the title says "Go to List" and the input hint says "Enter list ID".
- **GIVEN** the user types a valid list ID and presses Enter, **WHEN** the list exists, **THEN** the TaskList screen loads with tasks from that list and breadcrumb shows `"ClickUp › {list name}"`.
- **GIVEN** the user types an invalid list ID, **WHEN** the API returns 404, **THEN** a flash message `"✗ List not found: {id}"` appears.

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-polish`
- **Context:** `crates/clickup-tui/src/input.rs`, `crates/clickup-tui/src/data.rs` (`spawn_load_task_detail` line 107 for pattern), `crates/clickup-api/src/endpoints/lists.rs` (`get_list` line 21)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new public items

---

## Execution Sequence

```
S-1 (go-to-task dialog) ──→ S-3 (go-to-list dialog, reuses dialog infra)
S-2 (recent tasks picker)     [independent]
```

S-1 and S-2 are independent and can be developed in parallel.
S-3 reuses the dialog infrastructure from S-1 (shared `GotoKind` enum, dialog rendering).

## Assumptions

1. Task IDs in ClickUp are short alphanumeric strings (e.g., `abc123`). Users know these IDs from URLs, notifications, or other tools. **Risk if wrong:** None — the API accepts any string and returns 404 for invalid IDs.
2. Ten recent tasks is sufficient for the MVP. **Risk if wrong:** Can be made configurable later.
3. Recent task history is session-only (not persisted to disk). **Risk if wrong:** Users may want persistence. Deferred to a future enhancement.
4. The breadcrumb for jumped-to tasks uses a simplified path (`ClickUp › {task name}`) since we don't know the full hierarchy. **Risk if wrong:** Could reconstruct from `task.list.name`, `task.folder.name`, `task.space.id` — enhancement for later.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 1 | Single crate (TUI only), well-bounded |
| Ambiguity | 1 | Clear UI patterns, simple state management |
| Dependencies | 2 | Touches App state, input, data, and UI; S-3 depends on S-1 |
| Risk | 1 | Uses existing API endpoints, no new external calls |

**Total: 5/12** → Standard processing
