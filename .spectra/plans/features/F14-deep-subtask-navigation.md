# F14 — Deep Subtask Navigation

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 4 — Navigation & Content
> **Complexity:** 5/12 | **Confidence:** 87%

---

## Problem Statement

The TUI currently displays subtasks as a flat list on the TaskDetail screen (`crates/clickup-tui/src/ui/task_detail.rs`, lines 162–183) with status icons, but they are not interactive. Users cannot drill into a subtask to view its details, nor can they navigate subtask hierarchies (subtasks of subtasks). In ClickUp, tasks can nest arbitrarily deep, so a user tracking a feature epic with nested work items has no way to explore the subtask tree without leaving the TUI and opening the web app.

F04 (Phase 1) added basic one-level subtask navigation — pressing Enter on a subtask loads its TaskDetail. F14 extends this to support **recursive multi-level subtask trees** with indented rendering, lazy loading of grandchild subtasks, and a parent-chain stack for full breadcrumb tracking through arbitrarily deep hierarchies.

## Approach

Introduce a `task_stack: Vec<String>` field in the `App` struct to track the chain of parent task IDs as the user drills into subtasks. When viewing a task's subtasks section:

1. **Tree rendering:** Subtasks are displayed with indentation levels reflecting their depth. The first level of subtasks is fetched with the parent task detail. Sub-subtasks (children of subtasks) are lazy-loaded on expand — only when the user expands a subtask node or navigates into it.

2. **Recursive navigation:** Pressing Enter on a subtask in the detail view pushes the current task ID onto `task_stack`, loads the subtask's detail via `spawn_load_task_detail()`, and pushes the subtask name onto the breadcrumb. Pressing Esc pops the stack, restoring the parent task's detail from cache or re-fetching it.

3. **Lazy loading:** Sub-subtask lists are fetched on demand using `GET /task/{subtask_id}?include_subtasks=true`. A `subtask_cache: HashMap<String, Vec<Task>>` in `App` stores previously loaded subtask lists to avoid redundant API calls on back-navigation.

### Rejected Alternatives

1. **Eager recursive fetch of entire tree** — Would issue O(n) API calls on initial task load, causing latency and rate-limit risk. Rejected for performance.

2. **Separate tree-view screen** — Adding a new `Screen::SubtaskTree` variant would fragment navigation. Keeping subtask display within TaskDetail is more intuitive and consistent with ClickUp's own UX. Rejected for UX coherence.

3. **Flat list with depth prefix** — Showing all subtasks at all levels in a single flat list loses the interactive drill-down. Rejected because it doesn't solve the navigation problem.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `task_stack: Vec<String>` in `App` for parent tracking | Subtask creation or editing | Drag-and-drop subtask reordering |
| Indented subtask tree rendering in TaskDetail | Moving subtasks between parents | Collapsible tree nodes in subtask section |
| Lazy loading sub-subtasks via `get_task()` | Subtask count badges | Parallel prefetching of adjacent subtasks |
| Full breadcrumb chain (root → ... → subtask name) | Tree view as a standalone screen | |
| Back-navigation via Esc through task stack | | |
| `subtask_cache: HashMap<String, Vec<Task>>` | | |

## Stories

### S-1: Indented Subtask Tree Rendering with Lazy Loading

**As a** TUI user viewing a task with nested subtasks, **I want** subtasks displayed with indentation reflecting their depth and sub-subtasks loaded on demand, **so that** I can see the task hierarchy without waiting for the entire tree to load.

**Timebox:** ≤3d | **Risk:** Medium — API may return inconsistent subtask depth | **Depends on:** F04 (basic subtask nav)

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `task_stack: Vec<String>` field | `crates/clickup-tui/src/app.rs` | Tracks parent task IDs for back-navigation; initialize empty in `App::new()` |
| 2 | Add `subtask_cache: HashMap<String, Vec<Task>>` field | `crates/clickup-tui/src/app.rs` | Cache subtask lists keyed by parent task ID |
| 3 | Add `subtask_expanded: HashSet<String>` field | `crates/clickup-tui/src/app.rs` | Track which subtask nodes are expanded in the tree view |
| 4 | Add `selected_subtask_index: usize` field | `crates/clickup-tui/src/app.rs` | Track cursor position within the subtask section |
| 5 | Create `spawn_load_subtasks()` function | `crates/clickup-tui/src/data.rs` | Calls `client.get_task(subtask_id)` with `include_subtasks=true`, sends `DataPayload::SubtasksLoaded(parent_id, subtasks)` |
| 6 | Add `DataPayload::SubtasksLoaded` variant | `crates/clickup-tui/src/app.rs` | Carries `(String, Vec<Task>)` — parent task ID and its subtasks |
| 7 | Handle `SubtasksLoaded` in event processing | `crates/clickup-tui/src/app.rs` | Insert into `subtask_cache`, mark expanded |
| 8 | Refactor subtask rendering to tree view | `crates/clickup-tui/src/ui/task_detail.rs` | Replace flat subtask list (lines 162–183) with recursive indented rendering; show `▶`/`▼` expand indicators; indent = `depth * 4` spaces |
| 9 | Add unit tests for tree rendering | `crates/clickup-tui/src/ui/task_detail.rs` | Test with 0, 1, 2, 3 levels of nesting; test empty subtask lists |

#### Acceptance Criteria

- **GIVEN** a task with subtasks that themselves have subtasks, **WHEN** viewing TaskDetail, **THEN** first-level subtasks are displayed with `▶` indicator and 4-space indentation.
- **GIVEN** a subtask node is expanded, **WHEN** its children have been loaded, **THEN** children are displayed indented 8 spaces with their own `▶`/`▼` indicators.
- **GIVEN** a subtask node has not been expanded, **WHEN** the user has not interacted with it, **THEN** no API call is made for its children (lazy loading).
- **GIVEN** subtask data was previously loaded, **WHEN** navigating back and re-expanding, **THEN** the cached data is used (no redundant API call).

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-tasks`
- **Context:** `crates/clickup-tui/src/app.rs` (App struct ~line 148), `crates/clickup-tui/src/ui/task_detail.rs` (subtask rendering lines 162–183), `crates/clickup-tui/src/data.rs` (spawn functions)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected (TUI imports from `clickup-api`, not vice versa)
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new public items

---

### S-2: Recursive Navigation with Breadcrumb Chain

**As a** TUI user, **I want** to press Enter on a subtask to drill into its TaskDetail and press Esc to go back through the parent chain, **so that** I can navigate arbitrarily deep subtask hierarchies with clear context of where I am.

**Timebox:** ≤2d | **Risk:** Low — builds on existing navigation pattern | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add Enter key handler for subtask selection | `crates/clickup-tui/src/input.rs` | When on TaskDetail screen and subtask section is focused: push current `task.id` to `task_stack`, call `spawn_load_task_detail(subtask_id)`, push subtask name to breadcrumb |
| 2 | Modify Esc handler for task stack | `crates/clickup-tui/src/input.rs` | When on TaskDetail and `task_stack` is non-empty: pop `task_stack`, pop breadcrumb, load parent task from cache or re-fetch via `spawn_load_task_detail()` |
| 3 | Add `task_detail_cache: HashMap<String, Task>` | `crates/clickup-tui/src/app.rs` | Cache visited task details to avoid re-fetch on Esc |
| 4 | Cache current task on drill-down | `crates/clickup-tui/src/input.rs` | Before pushing to stack, store `current_task` in `task_detail_cache` |
| 5 | Restore from cache on Esc | `crates/clickup-tui/src/input.rs` | On pop, check `task_detail_cache` first; fall back to API fetch |
| 6 | Update key hints for subtask navigation | `crates/clickup-tui/src/ui/mod.rs` | Add `Enter Drill in` hint when subtask section is active; show depth indicator in breadcrumb |
| 7 | Add integration test | `crates/clickup-tui/tests/` | Test push/pop cycle: drill 3 levels deep, Esc back to root, verify breadcrumb at each level |

#### Acceptance Criteria

- **GIVEN** the user is viewing a TaskDetail with subtasks, **WHEN** they select a subtask and press Enter, **THEN** the subtask's TaskDetail loads, the breadcrumb shows `ClickUp › WS › Space › List › Parent Task › Subtask`, and `task_stack` contains the parent task ID.
- **GIVEN** the user is 3 levels deep in subtask navigation, **WHEN** they press Esc, **THEN** they return to the parent task's TaskDetail with correct breadcrumb (`ClickUp › WS › Space › List › Parent Task`).
- **GIVEN** the user drills into a subtask and presses Esc, **WHEN** the parent task was cached, **THEN** no API call is made (restored from `task_detail_cache`).
- **GIVEN** the user is at the first level TaskDetail (empty `task_stack`), **WHEN** they press Esc, **THEN** normal back-navigation occurs (return to TaskList screen).

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-tasks`
- **Context:** `crates/clickup-tui/src/input.rs` (TaskDetail key handling lines 536–710), `crates/clickup-tui/src/app.rs` (breadcrumb methods lines 354–363)
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
S-1 (tree rendering + lazy loading) ──→ S-2 (recursive navigation + breadcrumb)
```

S-2 depends on the `task_stack`, `subtask_cache`, and tree rendering infrastructure from S-1.

## Assumptions

1. The ClickUp API returns subtasks in `GET /task/{id}?include_subtasks=true` at any depth — each subtask itself has a `subtasks` array. **Risk if wrong:** Lazy loading pattern mitigates this; each level is a separate API call.
2. Subtask nesting in practice rarely exceeds 4–5 levels. **Risk if wrong:** Rendering with 20+ indent levels would overflow. Can add max-depth guard later.
3. F04 (basic subtask navigation) is already implemented, providing the Enter-on-subtask pattern. **Risk if wrong:** S-2 would need to implement the basic pattern as well.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 1 | Single crate (TUI only), well-bounded changes |
| Ambiguity | 1 | Clear requirements, existing navigation patterns to follow |
| Dependencies | 2 | Depends on F04; touches App state, input, data, and UI |
| Risk | 1 | Additive changes, no breaking modifications |

**Total: 5/12** → Standard processing
