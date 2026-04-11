# F30 — View Management

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 7 — Advanced Features
> **Complexity:** 5/12 | **Confidence:** 88%

---

## Problem Statement

ClickUp Views define how tasks are filtered, sorted, and grouped in the web UI. termaup currently has no concept of Views — users see a flat task list with manual filtering. Power users who have carefully configured Views in ClickUp (e.g., "My Active Tasks", "Sprint Board", "Bugs by Priority") must recreate those filter/sort configurations manually in termaup every session. Supporting Views allows termaup to match the web UI's curated task perspectives.

## Approach

Implement View read endpoints at all hierarchy levels (workspace, space, folder, list) and a View task fetcher that returns tasks matching a View's configured filters. In the CLI, add `clickup view list` and `clickup view tasks VIEW_ID` commands. In the TUI, add a View selector on the TaskList screen that lets users switch between available Views to apply the View's filters and sorts to the task display.

Views are read-only — creating and editing Views remains in the web UI, as the View configuration schema is complex and best managed visually.

### Rejected Alternatives

1. **Replicate View filter logic client-side** — Parse the View's `filters` and `sorts` configuration and apply them to locally fetched tasks. This duplicates ClickUp's filtering engine and would be fragile. Rejected because the API provides `GET /view/{id}/task` which returns pre-filtered tasks.

2. **Full View CRUD** — Allow creating/editing/deleting Views from the terminal. The View configuration includes complex filter trees, grouping rules, and column settings that are impractical to manage via CLI/TUI. Rejected for complexity.

3. **Auto-detect "default" view** — Automatically apply the first View found for each list. Views are context-dependent (some are personal, some shared), and auto-applying could confuse users. Rejected — explicit View selection is better UX.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| View listing at workspace/space/folder/list levels | View creation/editing/deletion | View task count caching |
| View task fetching (`GET /view/{id}/task`) | View column configuration parsing | View type icons (board, list, calendar, etc.) |
| CLI `view list` and `view tasks` commands | View sharing/permissions | View-based grouping in TUI |
| TUI View selector on TaskList screen | Calendar/Gantt/Board view rendering | View favorites |
| View models (id, name, type, parent) | | |

## Stories

### S-1: API Endpoints + Models

**As a** developer, **I want** View models and endpoint methods on `ClickUpClient`, **so that** CLI and TUI can fetch and display Views.

**Timebox:** ≤2d | **Risk:** Low | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create `View` model | `crates/clickup-api/src/models/view.rs` | `pub struct View { pub id: String, pub name: String, pub view_type: Option<String>, pub parent: Option<ViewParent>, pub grouping: Option<serde_json::Value>, pub divide: Option<serde_json::Value>, pub sorting: Option<serde_json::Value>, pub filters: Option<serde_json::Value>, pub columns: Option<serde_json::Value>, pub settings: Option<serde_json::Value>, pub date_created: Option<String>, pub creator: Option<User>, pub visibility: Option<String>, pub protected: Option<bool> }` |
| 2 | Create `ViewParent` model | `crates/clickup-api/src/models/view.rs` | `pub struct ViewParent { pub id: String, pub parent_type: Option<String> }` — with `#[serde(rename = "type")] parent_type` for reserved keyword |
| 3 | Create response wrappers | `crates/clickup-api/src/models/view.rs` | `pub struct ViewsResponse { pub views: Vec<View> }`, `pub struct ViewResponse { pub view: View }` |
| 4 | Register in models/mod.rs | `crates/clickup-api/src/models/mod.rs` | `pub mod view;` |
| 5 | Implement view listing endpoints | `crates/clickup-api/src/endpoints/views.rs` | `get_workspace_views(team_id)`, `get_space_views(space_id)`, `get_folder_views(folder_id)`, `get_list_views(list_id)` — each returns `Vec<View>` |
| 6 | Implement view detail endpoint | `crates/clickup-api/src/endpoints/views.rs` | `get_view(view_id)` → `View` |
| 7 | Implement view tasks endpoint | `crates/clickup-api/src/endpoints/views.rs` | `get_view_tasks(view_id, page)` → reuse `TasksResponse` from existing task models. Paginate until `last_page == true` |
| 8 | Register in endpoints/mod.rs | `crates/clickup-api/src/endpoints/mod.rs` | `pub mod views;` |
| 9 | Add `tracing::debug!` logging | `crates/clickup-api/src/endpoints/views.rs` | Log URL for each endpoint call |
| 10 | Create fixtures | `crates/clickup-api/tests/fixtures/views.json` | Realistic Views API response with multiple view types (list, board, calendar) |
| 11 | Unit tests | `crates/clickup-api/src/models/view.rs` | Deserialization tests: full, minimal (id+name only), views with complex filter JSON |
| 12 | Wiremock tests | `crates/clickup-api/tests/views_test.rs` | Test each endpoint: correct URL, correct response parsing |

#### Acceptance Criteria

- **GIVEN** a ClickUp API response with views containing `grouping`, `filters`, and `sorting` as nested JSON, **WHEN** deserialized, **THEN** these fields are preserved as `Option<serde_json::Value>`.
- **GIVEN** a minimal view response with only `id` and `name`, **WHEN** deserialized, **THEN** all other fields are `None`.
- **GIVEN** a space ID, **WHEN** `get_space_views(space_id)` is called, **THEN** a `GET /space/{space_id}/view` request is made.
- **GIVEN** a view ID, **WHEN** `get_view_tasks(view_id, 0)` is called, **THEN** a `GET /view/{view_id}/task?page=0` request is made.
- **GIVEN** the view's parent has `type: "space"`, **WHEN** deserialized, **THEN** `parent.parent_type` is `Some("space")` (using `#[serde(rename = "type")]`).

#### Agent Hints

- **Class:** builder
- **Context:** Follow `crates/clickup-api/src/endpoints/spaces.rs` and `crates/clickup-api/src/models/space.rs` as patterns. The `filters`, `sorting`, `grouping` fields are complex nested JSON that varies by view type — store as `serde_json::Value` to avoid modeling every variant. The `ViewParent.type` field conflicts with Rust's reserved keyword — use `#[serde(rename = "type")]` on `parent_type`, same pattern as `Status` model in `crates/clickup-api/src/models/status.rs`.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Deserialization tests cover full, minimal
  - [ ] P2: `cargo clippy` clean
  - [ ] P2: `///` doc comments on all public items

---

### S-2: CLI View Commands

**As a** CLI user, **I want** `clickup view list` and `clickup view tasks VIEW_ID`, **so that** I can discover available Views and see tasks through a View's filters.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create command module | `crates/clickup-cli/src/commands/views.rs` | `ViewCommands` enum with `List { space_id, folder_id, list_id }` and `Tasks { view_id }` |
| 2 | Implement `list` handler | `crates/clickup-cli/src/commands/views.rs` | Determine scope from provided flags (workspace/space/folder/list), call appropriate endpoint, display as table (name, type, creator, visibility) |
| 3 | Implement `tasks` handler | `crates/clickup-cli/src/commands/views.rs` | Call `get_view_tasks()`, display tasks using existing task table formatting from `commands/tasks.rs` |
| 4 | JSON + Markdown support | `crates/clickup-cli/src/commands/views.rs` | Support `--format json` and `--format markdown` |
| 5 | Register in mod.rs | `crates/clickup-cli/src/commands/mod.rs` | `pub mod views;` |
| 6 | Register in main.rs | `crates/clickup-cli/src/main.rs` | Add `View { command: ViewCommands }` to `Commands` enum |

#### Acceptance Criteria

- **GIVEN** `clickup view list --space SPACE_ID`, **WHEN** run, **THEN** views for the space are displayed in a table with name, type, and visibility.
- **GIVEN** `clickup view tasks VIEW_ID`, **WHEN** run, **THEN** tasks matching the view's filters are displayed in the standard task table format.
- **GIVEN** `clickup view list` without any scope flag, **WHEN** run with `--workspace` set, **THEN** workspace-level views are listed.
- **GIVEN** `--format json`, **WHEN** any view command is run, **THEN** output is pretty-printed JSON.

#### Agent Hints

- **Class:** builder
- **Context:** Follow `crates/clickup-cli/src/commands/spaces.rs` for command structure. The `list` subcommand should accept mutually exclusive `--space`, `--folder`, `--list` flags to scope the view query. If none specified, use workspace-level. Reuse task table formatting from `crates/clickup-cli/src/commands/tasks.rs` for the `tasks` subcommand output.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo clippy` clean

---

### S-3: TUI View Integration

**As a** TUI user browsing tasks, **I want** to select a View to apply its filters and sorts to the task list, **so that** I can see tasks the same way I've configured them in ClickUp.

**Timebox:** ≤2d | **Risk:** Medium (integrates with existing TaskList screen) | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add view state to `App` | `crates/clickup-tui/src/app.rs` | `pub available_views: Vec<View>`, `pub active_view: Option<View>`, `pub view_selector_open: bool`, `pub view_selector_index: usize` |
| 2 | Load views on list entry | `crates/clickup-tui/src/data.rs` | `pub fn spawn_load_list_views(client, tx, list_id)` — fetch views for the current list |
| 3 | Add data payload variant | `crates/clickup-tui/src/event.rs` | `DataPayload::Views(Vec<View>)`, `DataPayload::ViewTasks(Vec<Task>)` |
| 4 | View selector overlay | `crates/clickup-tui/src/ui/list_view.rs` | When `v` is pressed on TaskList, show a popup with available views. j/k to navigate, Enter to select, Esc to dismiss |
| 5 | Apply view tasks | `crates/clickup-tui/src/data.rs` | When a view is selected, call `spawn_load_view_tasks(client, tx, view_id)` to fetch tasks through the view's filters |
| 6 | Show active view name | `crates/clickup-tui/src/ui/list_view.rs` | Display `[View: Sprint Board]` indicator in the task list header when a view is active |
| 7 | Clear view | `crates/clickup-tui/src/input.rs` | Press `V` (shift+v) to clear the active view and return to the default task list |
| 8 | Key handling | `crates/clickup-tui/src/input.rs` | Handle `v` to toggle view selector, j/k/Enter/Esc within the selector popup |

#### Acceptance Criteria

- **GIVEN** the TaskList screen, **WHEN** `v` is pressed, **THEN** a popup shows available Views for the current list.
- **GIVEN** the view selector popup, **WHEN** a view is selected with Enter, **THEN** the task list reloads with tasks from that view's endpoint (`GET /view/{id}/task`).
- **GIVEN** an active view, **WHEN** the task list is rendered, **THEN** the view name is shown in the header (e.g., `[View: My Active Tasks]`).
- **GIVEN** an active view, **WHEN** `V` (Shift+V) is pressed, **THEN** the view is cleared and the task list reverts to the standard `GET /list/{id}/task` fetch.
- **GIVEN** a list with no views configured, **WHEN** `v` is pressed, **THEN** the popup shows "No views available" and dismisses on Esc.

#### Agent Hints

- **Class:** builder
- **Context:** The TaskList screen is rendered by `crates/clickup-tui/src/ui/list_view.rs`. The view selector popup should be similar to the existing filter panel or search overlay pattern. View tasks should be loaded using the same pagination pattern as `spawn_load_tasks()` in `crates/clickup-tui/src/data.rs`. The `active_view` on `App` determines which fetch method is used: if `Some(view)`, use `get_view_tasks(view.id)`, else use `get_tasks()`.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo clippy` clean

---

## Execution Sequence

```
S-1 (API endpoints + models) ──┬──→ S-2 (CLI commands)
                               └──→ S-3 (TUI view integration)
```

S-2 and S-3 can execute in parallel after S-1.

## Assumptions

1. **`GET /view/{id}/task` returns tasks in the same format** as `GET /list/{id}/task` (using `TasksResponse`). Risk if wrong: may need a separate response model. Mitigation: test with real API responses and handle extra/missing fields with `#[serde(default)]`.
2. **View `type` values** include `list`, `board`, `calendar`, `gantt`, `table`, `doc`, etc. Risk if wrong: the `view_type` field is `Option<String>` which handles any value.
3. **Views at the list level are the most common use case** for TUI integration. Risk if wrong: can add space-level and folder-level view selection later.
4. **Complex filter/sort/grouping configurations** in views are applied server-side by the `GET /view/{id}/task` endpoint. Risk if wrong: would need to parse and apply filters client-side. Mitigation: the API docs confirm server-side filtering for this endpoint.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | New models, endpoints, CLI commands, TUI popup + task loading integration |
| Ambiguity | 1 | Well-documented API; view data is read-only which simplifies implementation |
| Dependencies | 1 | Adds new files; TUI integration touches existing TaskList screen but is additive |
| Risk | 1 | Additive feature; existing task display is not modified, only the data source changes |

**Total: 5/12** → Lightweight processing
