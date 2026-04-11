# F04 — Subtask Navigation

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 1 — Critical MVP Gaps
> **Complexity:** 5/12 | **Confidence:** 90%

---

## Problem Statement

On the TUI TaskDetail screen, subtasks are displayed as a read-only list (status bullet + name + status badge), but pressing Enter on a subtask does nothing. Users must manually note the subtask ID, press Esc to go back to the task list, and use a different navigation path to view the subtask — or give up and use the web app. The parent → child → grandchild drill-down is fundamental to ClickUp's hierarchical task model, and blocking it in the TUI undermines the tool's value for projects with deep task structures.

## Approach

Add subtask drill-down by tracking a task navigation stack (`Vec<String>` of task IDs) in App state. When the user presses Enter on a subtask row, load its detail and push the parent task ID onto the stack. When Esc is pressed on a subtask detail, pop the stack and navigate back to the parent task (not the task list). Breadcrumbs show the full parent → subtask chain. This requires no API changes — `spawn_load_task_detail()` already works for any task ID, and `get_task()` with `include_subtasks=true` returns nested subtasks.

### Rejected Alternatives

1. **Open subtask in a new split pane** — Side-by-side parent + child would be powerful but requires a layout rework (splitting the main area), pane focus management, and synchronized scrolling. Over-scoped for the navigation problem. Deferred to a future "multi-pane" enhancement.

2. **Inline subtask expansion** (toggle subtask details within the parent's detail view) — Would require rendering a nested detail view inside the scroll area, creating layout complexity with variable-height content. The stack-based navigation is simpler and matches how the web app works. Rejected for UI complexity.

3. **Tab-based subtask switching** (Tab/Shift+Tab cycles through subtasks) — Would lose the parent context entirely. Users can't compare siblings or return to the parent easily. The stack model preserves full navigation history. Rejected for poor navigation UX.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| Enter on subtask row → load subtask detail | Subtask creation from TUI (see F02) | Parallel subtask viewing |
| Task navigation stack in App state | Subtask reordering / drag | Subtask status quick-change |
| Esc on subtask → return to parent (not task list) | Subtask deletion | Collapse/expand subtask tree |
| Breadcrumb shows parent → subtask chain | API changes | Subtask indentation levels |
| Support arbitrary nesting depth | | |

## Stories

### S-1: Task navigation stack and subtask drill-down

**As a** TUI user viewing a task's details, **I want** to press Enter on a subtask row to navigate into that subtask's detail view, **so that** I can inspect subtask details without leaving the terminal.

**Timebox:** ≤2d | **Risk:** P0 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify | `crates/clickup-tui/src/app.rs` | Add `task_detail_stack: Vec<String>` field to `App` struct (initialized as empty vec). This stores task IDs for parent tasks that we've drilled down from. Add helper methods: `push_task_stack(task_id: &str)` pushes current task ID onto stack, `pop_task_stack() -> Option<String>` pops the last task ID |
| 2 | Modify | `crates/clickup-tui/src/app.rs` | Add `subtask_selected_index: usize` field to track which subtask row is selected in the TaskDetail subtask section. Add `subtask_section_focused: bool` to track whether the subtask list has focus (vs the main detail scroll area) |
| 3 | Modify | `crates/clickup-tui/src/input.rs` | In `handle_task_detail()` (lines 536-710), add key handling for subtask navigation. When sidebar is closed: `Tab` toggles focus between detail scroll and subtask list. When subtask section is focused: `j`/`Down` = next subtask, `k`/`Up` = prev subtask, `Enter` = drill into selected subtask: push current task ID to stack, `spawn_load_task_detail(subtask.id)`, push subtask name to breadcrumb |
| 4 | Modify | `crates/clickup-tui/src/input.rs` | Modify `Esc` handling in `handle_task_detail()` (lines 694-701): check if `task_detail_stack` is non-empty. If so, pop the parent task ID, call `spawn_load_task_detail(parent_id)`, pop breadcrumb, set screen to `TaskDetail` (stay on detail). If stack is empty, navigate back to TaskList as before |
| 5 | Modify | `crates/clickup-tui/src/ui/task_detail.rs` | Update subtask rendering (lines 162-183): add selection highlight (reverse colors) on the subtask row matching `app.subtask_selected_index` when `app.subtask_section_focused` is true. Add a visual indicator (e.g., `▶` prefix) on the focused subtask section header when it has focus |
| 6 | Modify | `crates/clickup-tui/src/ui/task_detail.rs` | Add bottom-bar key hints when subtask section is focused: `[Enter] Open  [Tab] Back to detail  [Esc] Back` |
| 7 | Modify | `crates/clickup-tui/src/main.rs` | When `DataPayload::TaskDetail` arrives and `task_detail_stack` is non-empty, ensure the screen stays on `TaskDetail` (don't reset to Loading or another screen) |

#### Acceptance Criteria

- [ ] GIVEN a task with subtasks is displayed WHEN the user presses `Tab` THEN focus moves to the subtask list and the first subtask is highlighted
- [ ] GIVEN the subtask list is focused WHEN the user presses `j`/`Down` or `k`/`Up` THEN the selection moves through the subtask list with wrapping
- [ ] GIVEN a subtask is selected WHEN the user presses `Enter` THEN `spawn_load_task_detail()` is called with the subtask's ID, the parent task ID is pushed onto `task_detail_stack`, and the breadcrumb updates to include the subtask name
- [ ] GIVEN the user has drilled into a subtask WHEN they press `Esc` THEN the parent task is reloaded via `spawn_load_task_detail()` (using the ID popped from the stack), and the breadcrumb pops the subtask name
- [ ] GIVEN the user is on a top-level task (stack is empty) WHEN they press `Esc` THEN navigation goes back to TaskList as before (existing behavior preserved)
- [ ] GIVEN the user has drilled three levels deep (task → subtask → sub-subtask) WHEN they press `Esc` twice THEN they return to the original task, with breadcrumbs reflecting each level
- [ ] GIVEN a subtask has no subtasks of its own WHEN its detail view is displayed THEN the subtask section is absent and Tab does nothing (no focus to an empty list)

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/input.rs` (lines 536-710 `handle_task_detail`, lines 694-701 Esc handling), `crates/clickup-tui/src/ui/task_detail.rs` (lines 162-183 subtask rendering), `crates/clickup-tui/src/app.rs` (lines 148-276 App struct, breadcrumb field), `crates/clickup-tui/src/data.rs` (lines 107-128 `spawn_load_task_detail`)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code; no `panic!()` in library code
  - [ ] P0: Main render thread is never blocked — task detail load happens via `tokio::spawn`
  - [ ] P0: Existing Esc behavior (back to TaskList when stack is empty) is preserved
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` passes
  - [ ] P2: All new public items have `///` doc comments

---

### S-2: Breadcrumb and back-navigation UX polish

**As a** TUI user navigating subtask hierarchies, **I want** the breadcrumb bar to show the full path (e.g., `Workspace > Space > List > Parent Task > Subtask`) and the bottom bar to show context-appropriate hints, **so that** I always know where I am and how to go back.

**Timebox:** ≤1d | **Risk:** P2 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify | `crates/clickup-tui/src/input.rs` | When drilling into a subtask (Enter handler from S-1), push the subtask name (not just ID) to `app.breadcrumb`. When returning from a subtask (Esc handler from S-1), pop the last breadcrumb entry |
| 2 | Modify | `crates/clickup-tui/src/ui/layout.rs` | Ensure the breadcrumb bar truncates gracefully when the path is long (e.g., ellipsis on the left: `… > List > Parent > Subtask`). The existing breadcrumb renderer may already handle this — verify and adjust if needed |
| 3 | Modify | `crates/clickup-tui/src/ui/mod.rs` | Update `key_hints()` for TaskDetail screen: when `subtask_section_focused`, show `[Enter] Open  [↑↓] Navigate  [Tab] Detail  [Esc] Back`. When on a subtask (stack non-empty), change Esc hint from `[Esc] Back to list` to `[Esc] Parent task` |
| 4 | Modify | `crates/clickup-tui/src/ui/help.rs` | Add subtask navigation keys to the TaskDetail help overlay: `Tab` = Focus subtasks, `Enter` = Open subtask, `Esc` = Parent task / Back to list |

#### Acceptance Criteria

- [ ] GIVEN the user has navigated from TaskList into a task and then into a subtask WHEN the breadcrumb bar renders THEN it shows the full path like `My Workspace > My Space > My List > Parent Task > Subtask Name`
- [ ] GIVEN a very long breadcrumb path (5+ levels) WHEN the terminal width is narrow THEN the breadcrumb truncates gracefully without layout breakage
- [ ] GIVEN the user is viewing a subtask (stack is non-empty) WHEN the bottom bar renders THEN the Esc hint says `Parent task` (not `Back to list`)
- [ ] GIVEN the user is viewing a top-level task (stack is empty) WHEN the bottom bar renders THEN the Esc hint says `Back to list` (existing behavior preserved)
- [ ] GIVEN the user presses `?` on TaskDetail WHEN the help overlay renders THEN it includes subtask navigation keys (Tab, Enter, Esc context)

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/layout.rs` (breadcrumb rendering), `crates/clickup-tui/src/ui/mod.rs` (key_hints function), `crates/clickup-tui/src/ui/help.rs` (help overlay)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` passes
  - [ ] P2: Help overlay includes subtask navigation documentation

## Execution Sequence

```
S-1 (navigation stack + drill-down) → S-2 (breadcrumb + UX polish)
```

Linear dependency: S-2 polishes the navigation that S-1 implements.

## Assumptions

1. `get_task()` with `include_subtasks=true` returns subtasks with enough data (at minimum `id`, `name`, `status`) to display and navigate — Risk if wrong: subtask entries may be stubs requiring a separate API call per subtask (LOW — verified from existing Task model, subtasks are `Vec<Task>` with full fields)
2. Subtask IDs are globally unique and can be used with `get_task()` directly — Risk if wrong: subtask IDs might require the parent task context to resolve (NEGLIGIBLE — ClickUp task IDs are globally unique custom IDs)
3. The existing breadcrumb `Vec<String>` push/pop model is sufficient for task hierarchy (no need for typed entries) — Risk if wrong: breadcrumb entries need metadata for back-navigation, requiring a struct instead of String (LOW — the task_detail_stack provides the IDs, breadcrumb is purely display)
4. `Tab` key is not already bound on the TaskDetail screen — Risk if wrong: key conflict (LOW — verified from input.rs, Tab is unused on TaskDetail when sidebar is closed)

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 1 | TUI-only; no API or CLI changes; 4-5 files modified |
| Ambiguity | 1 | Navigation stack is a well-understood pattern; exact files and line numbers known |
| Dependencies | 2 | Relies on existing `spawn_load_task_detail()` and `Task.subtasks` field; must integrate with breadcrumb, Esc handling, and key dispatch without breaking existing flows |
| Risk | 1 | No new API calls; no new data models; all changes are UI state management |
