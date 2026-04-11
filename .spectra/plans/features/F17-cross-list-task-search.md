# F17 — Cross-List Task Search

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 4 — Navigation & Content
> **Complexity:** 6/12 | **Confidence:** 78%

---

## Problem Statement

termaup currently only browses tasks within a single list (`GET /list/{list_id}/task`). There is no way to search for tasks across multiple lists, folders, or the entire workspace. Users who need to find a task by name, status, or other criteria must manually navigate through the workspace → space → folder → list hierarchy, checking each list individually. This makes the TUI and CLI impractical for users with many lists or when the task's location is unknown.

The ClickUp API v2 provides `GET /team/{team_id}/task` which returns filtered tasks across the entire workspace, but this endpoint has **no text search parameter** — it only supports filtering by `statuses[]`, `assignees[]`, `tags[]`, `date_created_gt/lt`, `due_date_gt/lt`, `include_closed`, `subtasks`, `order_by`, and pagination via `page`. Text-based task name matching must be done **client-side** by filtering the fetched results.

Additionally, users who know a task's ID should be able to jump directly to it via `GET /task/{task_id}`, which is already implemented.

## Approach

**Two-pronged strategy:**

1. **Workspace task endpoint:** Add `get_workspace_tasks()` wrapping `GET /team/{team_id}/task` with filter params. This fetches paginated task results across the workspace, which can then be filtered client-side by name substring.

2. **Client-side text filtering:** After fetching workspace tasks (potentially multiple pages), apply a case-insensitive substring match on `task.name` against the user's search query. This is necessary because the ClickUp API has no server-side text search.

3. **CLI:** `clickup task search` command with `--query`, `--workspace`, `--status`, `--assignee` flags. Fetches from the workspace endpoint, filters client-side by query.

4. **TUI:** `Ctrl+F` from any screen opens a search overlay. The user types a query, which triggers a workspace-wide task fetch + client-side filter. Results are displayed in a selectable list; pressing Enter navigates to the task's TaskDetail.

### Rejected Alternatives

1. **Fetch ALL tasks then search locally** — Would require loading every task in the workspace upfront. Impractical for large workspaces (thousands of tasks, many API pages, rate-limit risk). Rejected for performance.

2. **Use ClickUp's `GET /team/{id}/task` with aggressive pagination to build a local index** — Cache all tasks and search the cache. Too much upfront cost and staleness risk. Rejected for complexity.

3. **Only support direct task ID lookup** — Simple but doesn't solve the "find task by name" use case. Rejected for insufficient coverage.

4. **Custom search API** — No such endpoint exists in ClickUp API v2. Non-option.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `get_workspace_tasks()` API endpoint with filter params | Full-text search (ClickUp API doesn't support it) | Search result caching |
| Client-side name substring filtering | Search by description content | Fuzzy matching (e.g., fzf-style) |
| CLI `clickup task search --query "..." [--status "..."]` | Search across multiple workspaces | Saved searches / search history |
| TUI search overlay via `Ctrl+F` | Real-time streaming results | Search indexing |
| Direct task ID lookup (already exists via `get_task()`) | | |
| Pagination of workspace task results | | |

## Stories

### S-1: API Endpoint for Filtered Workspace Tasks

**As a** developer using `clickup-api`, **I want** a `get_workspace_tasks()` method that fetches tasks across a workspace with optional filters, **so that** CLI and TUI can implement cross-list search.

**Timebox:** ≤2d | **Risk:** Medium — complex query parameter handling | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create `WorkspaceTaskFilter` struct | `crates/clickup-api/src/models/task.rs` | Fields: `statuses: Vec<String>`, `assignees: Vec<String>`, `tags: Vec<String>`, `due_date_gt: Option<i64>`, `due_date_lt: Option<i64>`, `include_closed: bool`, `subtasks: bool`, `order_by: Option<String>`, `reverse: Option<bool>`. All with `#[serde(skip)]` — this is for query param building, not serialization. |
| 2 | Create `get_workspace_tasks()` method | `crates/clickup-api/src/endpoints/tasks.rs` | `pub async fn get_workspace_tasks(&self, team_id: &str, filters: &WorkspaceTaskFilter) -> Result<Vec<Task>>` — builds query params from filter struct, paginates with `page` param until `last_page == true` |
| 3 | Create `get_workspace_tasks_page()` method | `crates/clickup-api/src/endpoints/tasks.rs` | `pub async fn get_workspace_tasks_page(&self, team_id: &str, filters: &WorkspaceTaskFilter, page: usize) -> Result<TasksResponse>` — single page fetch for UI pagination |
| 4 | Build query params from filter | `crates/clickup-api/src/endpoints/tasks.rs` | Convert `WorkspaceTaskFilter` fields to `Vec<(&str, String)>` params; array fields like `statuses` become multiple `statuses[]=X` params |
| 5 | Add wiremock tests | `crates/clickup-api/tests/workspace_tasks_tests.rs` | Test with various filter combinations; test pagination (2 pages); test empty results |
| 6 | Add fixture | `crates/clickup-api/tests/fixtures/workspace_tasks_response.json` | Realistic multi-task response with `last_page` field |

#### Acceptance Criteria

- **GIVEN** a workspace ID and empty filters, **WHEN** `get_workspace_tasks()` is called, **THEN** it fetches `GET /team/{team_id}/task?page=0`, and if `last_page` is false, continues to `page=1`, etc., collecting all tasks.
- **GIVEN** filters with `statuses: ["open", "in progress"]`, **WHEN** the request is built, **THEN** the URL contains `statuses[]=open&statuses[]=in%20progress`.
- **GIVEN** `include_closed: true`, **WHEN** the request is built, **THEN** the URL contains `include_closed=true`.
- **GIVEN** the API returns 0 tasks, **WHEN** the response is processed, **THEN** an empty `Vec<Task>` is returned (no error).

#### Agent Hints

- **Class:** builder
- **Skill:** `api-endpoints`
- **Context:** `crates/clickup-api/src/endpoints/tasks.rs` (existing `get_tasks_page()` line 60 for pagination pattern), `crates/clickup-api/src/models/task.rs` (`TasksResponse` line 631)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: wiremock tests cover pagination and filter params
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on `get_workspace_tasks()`, `WorkspaceTaskFilter`

---

### S-2: CLI Search Command

**As a** CLI user, **I want** `clickup task search --query "keyword" [--workspace WS_ID] [--status "open"]` to find tasks by name across my workspace, **so that** I can locate tasks without navigating the hierarchy.

**Timebox:** ≤2d | **Risk:** Low — follows existing CLI patterns | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `Search` variant to `TaskCommands` | `crates/clickup-cli/src/commands/tasks.rs` | `Search { #[arg(long)] query: Option<String>, #[arg(long)] workspace: Option<String>, #[arg(long)] status: Vec<String>, #[arg(long)] assignee: Vec<String>, #[arg(long)] include_closed: bool, #[arg(long)] format: Option<OutputFormat> }` |
| 2 | Implement search handler | `crates/clickup-cli/src/commands/tasks.rs` | Build `WorkspaceTaskFilter` from flags, call `get_workspace_tasks()`, apply client-side `query` filter (case-insensitive substring on `task.name`), format results as table or JSON |
| 3 | Auto-detect workspace | `crates/clickup-cli/src/commands/tasks.rs` | If `--workspace` not provided, call `get_workspaces()` and use the first one (or prompt if multiple via `dialoguer`) |
| 4 | Format search results table | `crates/clickup-cli/src/commands/tasks.rs` | Table columns: ID, Name, Status, List, Assignees, Due Date |
| 5 | Handle no results | `crates/clickup-cli/src/commands/tasks.rs` | Print `"ℹ No tasks found matching '{query}'"` |

#### Acceptance Criteria

- **GIVEN** tasks exist with "deploy" in the name, **WHEN** running `clickup task search --query "deploy"`, **THEN** matching tasks are displayed in a table with ID, Name, Status, List columns.
- **GIVEN** a `--status "open"` filter, **WHEN** the command runs, **THEN** only open tasks are returned from the API (server-side filter).
- **GIVEN** no tasks match the query, **WHEN** the command completes, **THEN** `"ℹ No tasks found matching 'nonexistent'"` is printed.
- **GIVEN** `--format json`, **WHEN** results are returned, **THEN** output is pretty-printed JSON array.
- **GIVEN** no `--workspace` flag and the user has one workspace, **WHEN** the command runs, **THEN** the single workspace is used automatically.

#### Agent Hints

- **Class:** builder
- **Skill:** `cli-data`
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (TaskCommands enum lines 8–38, existing handler pattern)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new public items

---

### S-3: TUI Search Overlay

**As a** TUI user, **I want** to press `Ctrl+F` to open a search overlay, type a query, and see matching tasks from my workspace, **so that** I can quickly find and jump to any task.

**Timebox:** ≤3d | **Risk:** Medium — new overlay pattern, async data loading | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add search state fields to `App` | `crates/clickup-tui/src/app.rs` | `search_overlay_open: bool`, `search_query: String`, `search_results: Vec<Task>`, `search_selected_index: usize`, `search_loading: bool` |
| 2 | Create `spawn_search_tasks()` | `crates/clickup-tui/src/data.rs` | Takes `team_id`, `query` string; calls `get_workspace_tasks()` with default filters + `include_closed: false`; filters results client-side by `query`; sends `DataPayload::SearchResults(Vec<Task>)` |
| 3 | Add `DataPayload::SearchResults` variant | `crates/clickup-tui/src/app.rs` | Carries `Vec<Task>` of matching results |
| 4 | Handle `Ctrl+F` globally | `crates/clickup-tui/src/input.rs` | Set `search_overlay_open = true`, focus search input |
| 5 | Handle search input keys | `crates/clickup-tui/src/input.rs` | When overlay is open: `Char(c)` appends to query, `Backspace` removes last char, `Enter` on result navigates to TaskDetail, `Esc` closes overlay, `↑/↓` navigate results |
| 6 | Debounce search trigger | `crates/clickup-tui/src/input.rs` | Trigger search after 300ms of no typing (or on Enter in search box). Use tick counter or `Instant` comparison. |
| 7 | Create search overlay renderer | `crates/clickup-tui/src/ui/search.rs` | Centered popup (60% width, 80% height): search input at top, results list below with task name, status badge, list name. Highlight selected result. Show "Searching..." spinner while loading. |
| 8 | Register module | `crates/clickup-tui/src/ui/mod.rs` | Add `pub mod search;` and call `search::render()` when overlay is open |
| 9 | Navigate to task on Enter | `crates/clickup-tui/src/input.rs` | On Enter with result selected: close overlay, set `current_task`, push breadcrumb, switch to `Screen::TaskDetail` |
| 10 | Update global key hints | `crates/clickup-tui/src/ui/mod.rs` | Add `Ctrl+F Search` to key hints on screens where search is available |

#### Acceptance Criteria

- **GIVEN** the user is on any screen, **WHEN** they press `Ctrl+F`, **THEN** a search overlay appears centered on screen with a text input.
- **GIVEN** the overlay is open and the user types "deploy", **WHEN** the search executes, **THEN** matching tasks from the current workspace appear in the results list.
- **GIVEN** search results are displayed, **WHEN** the user selects a task with `↓/↑` and presses Enter, **THEN** the overlay closes and TaskDetail for that task loads.
- **GIVEN** the overlay is open, **WHEN** the user presses Esc, **THEN** the overlay closes and the previous screen is restored.
- **GIVEN** no tasks match the query, **WHEN** the search completes, **THEN** the overlay shows `"No results found"`.
- **GIVEN** the search is in progress, **WHEN** rendering, **THEN** a loading spinner is displayed.

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-polish`
- **Context:** `crates/clickup-tui/src/ui/mod.rs` (overlay rendering pattern — see help overlay), `crates/clickup-tui/src/input.rs` (global key handling lines 44–87), `crates/clickup-tui/src/data.rs`
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected — TUI only renders + dispatches, API does the fetching
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new public items

---

### S-4: Client-Side Text Filtering Utility

**As a** developer, **I want** a reusable client-side text filtering function for task lists, **so that** both CLI and TUI search implementations use consistent matching logic.

**Timebox:** ≤1d | **Risk:** Low — pure logic, no I/O | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create `filter_tasks_by_name()` function | `crates/clickup-api/src/models/task.rs` | `pub fn filter_tasks_by_name(tasks: &[Task], query: &str) -> Vec<&Task>` — case-insensitive substring match on `task.name`. Also matches `custom_id` if present. |
| 2 | Add unit tests | `crates/clickup-api/src/models/task.rs` | Test: exact match, substring match, case-insensitive, no match, empty query (returns all), match on `custom_id` |

#### Acceptance Criteria

- **GIVEN** tasks `["Deploy API", "Deploy Frontend", "Fix Bug"]` and query `"deploy"`, **WHEN** `filter_tasks_by_name()` is called, **THEN** `["Deploy API", "Deploy Frontend"]` are returned.
- **GIVEN** query `"DEPLOY"` (uppercase), **WHEN** filtering, **THEN** case-insensitive match returns the same results.
- **GIVEN** an empty query `""`, **WHEN** filtering, **THEN** all tasks are returned.
- **GIVEN** a task with `custom_id: Some("PROJ-123")` and query `"PROJ-123"`, **WHEN** filtering, **THEN** the task matches.

#### Agent Hints

- **Class:** builder
- **Skill:** `domain-models`
- **Context:** `crates/clickup-api/src/models/task.rs` (Task struct line 8)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P2: `cargo clippy` clean
  - [ ] P2: `///` doc comments on `filter_tasks_by_name()`

---

## Execution Sequence

```
S-4 (text filter util) ─┐
                         ├─→ S-2 (CLI search, uses S-1 + S-4)
S-1 (API endpoint)     ─┤
                         └─→ S-3 (TUI search overlay, uses S-1 + S-4)
```

S-1 and S-4 are independent and can be developed in parallel.
S-2 and S-3 both depend on S-1 and S-4, but are independent of each other.

## Assumptions

1. `GET /team/{team_id}/task` supports the `page` parameter for pagination and returns `last_page: bool` in the response. **Risk if wrong:** Pagination pattern matches `GET /list/{id}/task` which is already implemented and tested.
2. The ClickUp API does not add server-side text search in the near future. **Risk if wrong:** If text search becomes available, the client-side filter can be replaced with a server-side param — improvement, not breakage.
3. Client-side filtering across all workspace tasks is feasible for typical workspaces (< 10,000 tasks). **Risk if wrong:** For very large workspaces, fetching all tasks would be slow. Could add a page limit and "showing first N results" message.
4. The `GET /team/{team_id}/task` response uses the same `TasksResponse` struct as `GET /list/{id}/task`. **Risk if wrong:** May need a separate response struct; low risk per ClickUp API docs.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Three crates affected (API, CLI, TUI), new overlay UI |
| Ambiguity | 2 | No server-side search — client-side filtering UX has open questions (debounce, pagination) |
| Dependencies | 1 | New endpoint, no conflicts |
| Risk | 1 | API endpoint is documented; client-side filtering is deterministic |

**Total: 6/12** → Standard processing
