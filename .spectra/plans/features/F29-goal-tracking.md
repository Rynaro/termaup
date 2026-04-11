# F29 — Goal Tracking

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 7 — Advanced Features
> **Complexity:** 6/12 | **Confidence:** 85%

---

## Problem Statement

ClickUp Goals are a key feature for tracking OKRs and team objectives, but termaup has no support for viewing or managing goals. Users who rely on Goals for planning must switch to the web UI to check goal progress, create key results, or update targets. Adding Goals support completes the "read everything, manage what matters" value proposition.

## Approach

Follow the established pattern for new resource types:

1. **API crate:** Define models (`Goal`, `KeyResult`, request/response wrappers), implement endpoint methods on `ClickUpClient` following the existing patterns in `crates/clickup-api/src/endpoints/`.
2. **CLI crate:** Add `clickup goal` command group with `list`, `get`, `create`, `delete` subcommands.
3. **TUI crate:** Add a Goals screen accessible from the WorkspaceSelect screen, showing goals with progress bars and key results.

The ClickUp Goals API endpoints are well-documented and follow the standard REST pattern used by other resources.

### Rejected Alternatives

1. **Goals as a filter on tasks** — Show only tasks linked to goals. This misses the goal metadata (progress, targets, key results) and doesn't represent the Goals concept. Rejected for incompleteness.

2. **Read-only goals** — Only `list` and `get` without `create`/`delete`. While simpler, goals are often managed alongside tasks in planning workflows. Including basic CRUD makes the feature self-contained. Rejected for limited value.

3. **Defer TUI integration** — Build only API + CLI first. The TUI is where users spend most time; a Goals screen adds significant value and follows the same patterns as existing screens. Rejected for missed opportunity.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| Goal CRUD (create, read, update, delete) | Goal history/activity feed | Goal-to-task linking display |
| Key Result CRUD | Goal templates | Goal progress charts |
| CLI `goal` command group | Goal sharing/permissions | Goal notifications |
| TUI Goals screen with progress display | Key result metrics (time-based, currency) | Goal folders/categories |
| Defensive serde models | | Bulk goal operations |

## Stories

### S-1: API Models + Endpoints

**As a** developer using the API crate, **I want** Goal and KeyResult models with full CRUD endpoints, **so that** CLI and TUI can build goal management features on a solid foundation.

**Timebox:** ≤3d | **Risk:** Low | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create `Goal` model | `crates/clickup-api/src/models/goal.rs` | `pub struct Goal { pub id: String, pub name: String, pub description: Option<String>, pub color: Option<String>, pub owner: Option<User>, pub date_created: Option<String>, pub due_date: Option<String>, pub percent_completed: Option<f64>, pub key_results: Vec<KeyResult>, pub members: Vec<User>, pub folder_id: Option<String>, pub pinned: Option<bool>, pub multiple_owners: Option<bool> }` — derive `Debug, Clone, Serialize, Deserialize` |
| 2 | Create `KeyResult` model | `crates/clickup-api/src/models/goal.rs` | `pub struct KeyResult { pub id: String, pub name: Option<String>, pub key_result_type: Option<String>, pub steps_current: Option<f64>, pub steps_start: Option<f64>, pub steps_end: Option<f64>, pub unit: Option<String>, pub owner: Option<User>, pub task_ids: Vec<String>, pub percent_completed: Option<f64> }` |
| 3 | Create response wrappers | `crates/clickup-api/src/models/goal.rs` | `pub struct GoalsResponse { pub goals: Vec<Goal> }`, `pub struct GoalResponse { pub goal: Goal }` |
| 4 | Create request models | `crates/clickup-api/src/models/goal.rs` | `pub struct CreateGoalRequest { pub name: String, pub due_date: Option<i64>, pub description: Option<String>, pub color: Option<String>, pub multiple_owners: Option<bool>, pub owners: Vec<u64> }`, `pub struct CreateKeyResultRequest { pub name: String, pub key_result_type: Option<String>, pub steps_start: Option<f64>, pub steps_end: Option<f64>, pub unit: Option<String>, pub task_ids: Option<Vec<String>> }` — all with `skip_serializing_if = "Option::is_none"` |
| 5 | Register in models/mod.rs | `crates/clickup-api/src/models/mod.rs` | `pub mod goal;` and re-export types |
| 6 | Implement endpoints | `crates/clickup-api/src/endpoints/goals.rs` | `impl ClickUpClient { get_goals(), get_goal(), create_goal(), update_goal(), delete_goal(), create_key_result(), update_key_result(), delete_key_result() }` |
| 7 | Register in endpoints/mod.rs | `crates/clickup-api/src/endpoints/mod.rs` | `pub mod goals;` |
| 8 | Add `tracing::debug!` | `crates/clickup-api/src/endpoints/goals.rs` | Log HTTP method + URL for each endpoint call |
| 9 | Serde helpers | `crates/clickup-api/src/models/goal.rs` | Use `#[serde(default)]` on Vec fields, `Option<T>` for all fields except `id` and `name` on `Goal` |
| 10 | Fixture JSON | `crates/clickup-api/tests/fixtures/goals.json` | Realistic ClickUp Goals API response |
| 11 | Unit tests | `crates/clickup-api/src/models/goal.rs` | Deserialization tests: full, minimal (id+name only), null-heavy, with/without key_results |
| 12 | Integration tests | `crates/clickup-api/tests/goals_test.rs` | Wiremock tests for each endpoint: verify URL, method, request body, response parsing |

#### Acceptance Criteria

- **GIVEN** a ClickUp API response with full goal data, **WHEN** deserialized, **THEN** all fields are correctly populated including nested `key_results` and `members`.
- **GIVEN** a minimal API response with only `id` and `name`, **WHEN** deserialized, **THEN** all other fields are `None` or empty `Vec` (no panic).
- **GIVEN** a valid team ID, **WHEN** `get_goals(team_id)` is called, **THEN** a `GET /team/{team_id}/goal` request is made and the response is parsed into `Vec<Goal>`.
- **GIVEN** a `CreateGoalRequest`, **WHEN** `create_goal(team_id, request)` is called, **THEN** a `POST /team/{team_id}/goal` request is made with the correct JSON body.
- **GIVEN** a goal ID, **WHEN** `delete_goal(goal_id)` is called, **THEN** a `DELETE /goal/{goal_id}` request is made.
- **GIVEN** a `CreateKeyResultRequest`, **WHEN** `create_key_result(goal_id, request)` is called, **THEN** a `POST /goal/{goal_id}/key_result` request is made.

#### Agent Hints

- **Class:** builder
- **Context:** Follow the pattern in `crates/clickup-api/src/endpoints/comments.rs` and `crates/clickup-api/src/models/comment.rs` for CRUD endpoints. Use existing HTTP methods on `ClickUpClient`: `self.get()`, `self.post()`, `self.put()`, `self.delete()`. The `User` model is already defined in `crates/clickup-api/src/models/user.rs` — reuse it for `owner` and `members` fields. Use `deserialize_null_as_default` from `crates/clickup-api/src/serde_helpers.rs` for Vec fields.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Deserialization tests cover full, minimal, null-heavy
  - [ ] P2: `cargo clippy` clean
  - [ ] P2: `///` doc comments on all public items

---

### S-2: CLI Goal Commands

**As a** CLI user, **I want** `clickup goal list` and `clickup goal get GOAL_ID`, **so that** I can view goals and their progress from the terminal.

**Timebox:** ≤2d | **Risk:** Low | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create command module | `crates/clickup-cli/src/commands/goals.rs` | `GoalCommands` enum with `List`, `Get { goal_id: String }`, `Create { name, description, due_date, color }`, `Delete { goal_id, yes }` |
| 2 | Implement `list` handler | `crates/clickup-cli/src/commands/goals.rs` | Fetch goals via `client.get_goals()`, format as table (name, owner, progress %, due date, key results count) |
| 3 | Implement `get` handler | `crates/clickup-cli/src/commands/goals.rs` | Fetch goal detail, display goal metadata + list of key results with progress |
| 4 | Implement `create` handler | `crates/clickup-cli/src/commands/goals.rs` | Build `CreateGoalRequest`, call `client.create_goal()`, print success with goal ID |
| 5 | Implement `delete` handler | `crates/clickup-cli/src/commands/goals.rs` | Confirm with `dialoguer` (unless `--yes`), call `client.delete_goal()`, print success |
| 6 | Table formatting | `crates/clickup-cli/src/commands/goals.rs` | Use `comfy-table` for table output, `owo-colors` for progress coloring (green >80%, yellow >50%, red ≤50%) |
| 7 | JSON + Markdown formats | `crates/clickup-cli/src/commands/goals.rs` | Support `--format json` and `--format markdown` |
| 8 | Register in mod.rs | `crates/clickup-cli/src/commands/mod.rs` | `pub mod goals;` |
| 9 | Register in main.rs | `crates/clickup-cli/src/main.rs` | Add `Goal { command: GoalCommands }` to `Commands` enum |

#### Acceptance Criteria

- **GIVEN** a workspace with goals, **WHEN** `clickup goal list` is run, **THEN** a table is displayed with goal name, owner, progress, due date, and key result count.
- **GIVEN** a valid goal ID, **WHEN** `clickup goal get GOAL_ID` is run, **THEN** goal details and key results are displayed.
- **GIVEN** `clickup goal create --name "Q3 Revenue"`, **WHEN** run, **THEN** a goal is created and the new goal ID is printed with `✓` prefix.
- **GIVEN** `clickup goal delete GOAL_ID`, **WHEN** run without `--yes`, **THEN** a confirmation prompt is shown. **WHEN** confirmed, **THEN** the goal is deleted.
- **GIVEN** `--format json`, **WHEN** any goal command is run, **THEN** output is pretty-printed JSON.

#### Agent Hints

- **Class:** builder
- **Context:** Follow the pattern in `crates/clickup-cli/src/commands/tasks.rs` for table formatting and JSON output. The `client_factory.rs` handles creating an authenticated `ClickUpClient`. Goal list requires a `team_id` — use the `--workspace` global arg or `Config::default_workspace_id`.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo clippy` clean

---

### S-3: TUI Goals Screen

**As a** TUI user, **I want** a Goals screen showing all workspace goals with progress bars, **so that** I can track OKR progress without leaving the terminal.

**Timebox:** ≤2d | **Risk:** Low | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `Goals` screen variant | `crates/clickup-tui/src/app.rs` | Add `Goals` to `Screen` enum |
| 2 | Add goal state to `App` | `crates/clickup-tui/src/app.rs` | `pub goals: Vec<Goal>`, `pub selected_goal_index: usize`, `pub selected_goal: Option<Goal>` |
| 3 | Add data payload variant | `crates/clickup-tui/src/event.rs` | `DataPayload::Goals(Vec<Goal>)`, `DataPayload::GoalDetail(Goal)` |
| 4 | Implement data loader | `crates/clickup-tui/src/data.rs` | `pub fn spawn_load_goals(client, tx, team_id)` following existing `spawn_load_*` pattern |
| 5 | Create goals screen UI | `crates/clickup-tui/src/ui/goals.rs` | List of goals with: name, owner, progress bar (using ratatui `Gauge` or custom), due date, key result count |
| 6 | Create goal detail rendering | `crates/clickup-tui/src/ui/goals.rs` | When goal is selected: show description, key results with individual progress, member list |
| 7 | Register in ui/mod.rs | `crates/clickup-tui/src/ui/mod.rs` | `pub mod goals;` |
| 8 | Add navigation entry | `crates/clickup-tui/src/input.rs` | From WorkspaceSelect, add a `g` key shortcut to navigate to Goals screen; Esc to go back |
| 9 | Add input handlers | `crates/clickup-tui/src/input.rs` | `handle_goals()` function for j/k navigation, Enter to view detail, Esc to go back |
| 10 | Update data event handler | `crates/clickup-tui/src/app.rs` | Handle `DataPayload::Goals` — store in `app.goals` |
| 11 | Progress bar widget | `crates/clickup-tui/src/ui/goals.rs` | Render `percent_completed` as a colored progress bar: green (>80%), yellow (>50%), red (≤50%) |

#### Acceptance Criteria

- **GIVEN** the WorkspaceSelect screen, **WHEN** `g` is pressed, **THEN** the TUI navigates to the Goals screen and loads goals for the selected workspace.
- **GIVEN** the Goals screen with loaded data, **WHEN** rendered, **THEN** each goal shows its name, owner, a progress bar, and due date.
- **GIVEN** a goal selected with `Enter`, **WHEN** the goal detail is shown, **THEN** key results are listed with individual progress.
- **GIVEN** the Goals screen, **WHEN** `Esc` is pressed, **THEN** the user returns to WorkspaceSelect.
- **GIVEN** no goals exist in the workspace, **WHEN** the Goals screen loads, **THEN** an "No goals found" message is displayed.
- **GIVEN** a goal with 75% completion, **WHEN** rendered, **THEN** the progress bar is yellow.

#### Agent Hints

- **Class:** builder
- **Context:** Follow the pattern of `crates/clickup-tui/src/ui/workspace_select.rs` for screen structure. The `render()` function signature is `pub fn render(app: &App, frame: &mut Frame, area: Rect)`. For progress bars, ratatui's `Gauge` widget works well. Add the Goals screen to the breadcrumb navigation. The `g` shortcut for goals should only be active on WorkspaceSelect (not other screens).
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo clippy` clean

---

### S-4: Goal Tests

**As a** maintainer, **I want** comprehensive tests for goal models, endpoints, and CLI output, **so that** goal functionality is reliable and regression-protected.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-1, S-2

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create goal fixtures | `crates/clickup-api/tests/fixtures/goals.json` | Full realistic Goals API response with multiple goals, key results, owners, members |
| 2 | Create minimal fixture | `crates/clickup-api/tests/fixtures/goal_minimal.json` | Goal with only `id` and `name` |
| 3 | Fixture deserialization tests | `crates/clickup-api/tests/goals_test.rs` | Deserialize full and minimal fixtures, assert field values |
| 4 | Wiremock endpoint tests | `crates/clickup-api/tests/goals_test.rs` | Test `get_goals()`, `get_goal()`, `create_goal()`, `delete_goal()` with mocked responses |
| 5 | CLI output snapshot tests | `crates/clickup-cli/tests/goals_test.rs` | If applicable, use `insta` for snapshot testing of table and JSON output formats |

#### Acceptance Criteria

- **GIVEN** the full goals fixture, **WHEN** deserialized, **THEN** all goals and key results are correctly parsed.
- **GIVEN** the minimal goal fixture (id + name only), **WHEN** deserialized, **THEN** no errors occur and optional fields are `None`.
- **GIVEN** a wiremock server returning goals JSON, **WHEN** `get_goals()` is called, **THEN** the correct URL is requested and the response is parsed.
- **GIVEN** a `create_goal()` call, **WHEN** wiremock inspects the request, **THEN** the JSON body matches the `CreateGoalRequest` serialization.

#### Agent Hints

- **Class:** builder
- **Context:** Follow patterns in `crates/clickup-api/tests/` for wiremock test structure. Use `wiremock::Mock::given(method(GET)).and(path("/api/v2/team/123/goal"))` patterns.
- **Gates:**
  - [ ] P0: `cargo test --workspace` passes
  - [ ] P1: Coverage for full, minimal, and null-heavy fixtures
  - [ ] P2: `cargo clippy` clean

---

## Execution Sequence

```
S-1 (API models + endpoints) ──┬──→ S-2 (CLI commands)
                               ├──→ S-3 (TUI Goals screen)
                               └──→ S-4 (tests)
```

S-2, S-3, and S-4 can execute in parallel after S-1.

## Assumptions

1. **ClickUp Goals API v2 is stable** and matches the documented endpoints (`GET/POST/PUT/DELETE /goal/{id}`). Risk if wrong: API may have undocumented quirks. Mitigation: defensive serde with `Option<T>` for all non-ID fields.
2. **Goals belong to workspaces (teams)**, not spaces or lists. Risk if wrong: may need space-scoped goal endpoints. Mitigation: the API uses `GET /team/{team_id}/goal` which confirms workspace scope.
3. **Key result `type` values** are well-defined (`number`, `currency`, `boolean`, `percentage`, `automatic`). Risk if wrong: use `Option<String>` for the type field and handle unknown types gracefully.
4. **Progress percentage** is returned directly by the API (not calculated client-side). Risk if wrong: would need to calculate from key result steps. Mitigation: the API returns `percent_completed` on goals.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | New models, endpoints, CLI commands, TUI screen — touches all 3 crates |
| Ambiguity | 1 | Well-documented API; follows established patterns in the codebase |
| Dependencies | 2 | Adds new files to each crate; TUI screen depends on new App state + events |
| Risk | 1 | Purely additive; no changes to existing functionality |

**Total: 6/12** → Standard processing
