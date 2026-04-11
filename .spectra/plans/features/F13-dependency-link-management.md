# F13 — Dependency & Link Management

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 3 — Task Enrichment
> **Complexity:** 5/12 | **Confidence:** 88%

---

## Problem Statement

Task dependencies ("Task A is waiting on Task B") and linked tasks ("Task A is related to Task C") are core ClickUp features for expressing relationships between tasks. termaup currently displays dependencies and linked tasks on the TaskDetail screen (rendering `LinkedTask` and `TaskDependency` models) but provides no way to create or remove these relationships. Users must switch to the ClickUp web app to manage task relationships, breaking their terminal workflow.

## Approach

Add four API endpoints: `add_dependency()`, `remove_dependency()`, `add_task_link()`, and `remove_task_link()`. The dependency endpoints use `POST/DELETE /task/{id}/dependency` with a JSON body specifying the direction (`depends_on` or `dependency_of`). The link endpoints use `POST/DELETE /task/{id}/link/{links_to}` with the linked task ID in the URL path. The CLI provides intuitive commands with clear direction semantics (`--waiting-on` vs `--blocks`). The TUI adds interactions in the dependency/link sections of TaskDetail.

### Rejected Alternatives

1. **Dependency graph visualization** — Render a visual DAG of task dependencies in the TUI. Complex rendering, not needed for CRUD operations, and deferred to a potential future feature (F14 or later). Rejected for scope.

2. **Bidirectional auto-creation** — When creating "A depends on B", automatically create "B blocks A" on the other task. The ClickUp API handles this server-side; we don't need to make two calls. Rejected as unnecessary — single API call creates both sides.

3. **Task search for link target** — Open a task search dialog to find the target task ID. Useful but complex (requires cross-list search, F17). For MVP, users provide the task ID directly. Rejected for now; deferred to F17 integration.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `POST /task/{id}/dependency` endpoint | Dependency cycle detection | Dependency graph visualization |
| `DELETE /task/{id}/dependency` endpoint | Cross-workspace dependencies | Task search for link targets |
| `POST /task/{id}/link/{links_to}` endpoint | Dependency type customization | Dependency impact analysis |
| `DELETE /task/{id}/link/{links_to}` endpoint | Automatic dependency resolution | Bulk dependency management |
| CLI `task depend` and `task link`/`unlink` commands | | |
| TUI dependency/link add and remove | | |

## Stories

### S-1: API Endpoints

**As a** developer using clickup-api, **I want** methods to add/remove task dependencies and links, **so that** CLI and TUI can manage task relationships.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/src/endpoints/dependencies.rs` | Add `add_dependency(&self, task_id: &str, depends_on: &str) -> Result<()>`: `POST /task/{task_id}/dependency` with body `{"depends_on": "OTHER_TASK_ID", "dependency_of": "{task_id}"}`. Log with `tracing::debug!`. |
| 2 | Extend | `crates/clickup-api/src/endpoints/dependencies.rs` | Add `remove_dependency(&self, task_id: &str, depends_on: &str) -> Result<()>`: `DELETE /task/{task_id}/dependency` with body `{"depends_on": "OTHER_ID", "dependency_of": "{task_id}"}`. Note: ClickUp's DELETE endpoint accepts a body — use a custom method if `client.delete()` doesn't support bodies. |
| 3 | Extend | `crates/clickup-api/src/endpoints/dependencies.rs` | Add `add_task_link(&self, task_id: &str, links_to: &str) -> Result<()>`: `POST /task/{task_id}/link/{links_to}` with empty body. |
| 4 | Extend | `crates/clickup-api/src/endpoints/dependencies.rs` | Add `remove_task_link(&self, task_id: &str, links_to: &str) -> Result<()>`: `DELETE /task/{task_id}/link/{links_to}`. |
| 5 | Create | `crates/clickup-api/src/models/dependency.rs` | Add `AddDependencyRequest` struct: `depends_on: String`, `dependency_of: String`. Add `RemoveDependencyRequest` with same fields. Both derive `Serialize, Deserialize, Debug, Clone`. |
| 6 | Modify | `crates/clickup-api/src/endpoints/mod.rs` | Add `pub mod dependencies;` declaration. |
| 7 | Modify | `crates/clickup-api/src/models/mod.rs` | Add `pub mod dependency;` and re-export types. |
| 8 | Extend | `crates/clickup-api/src/client.rs` | If `delete()` doesn't support request bodies, add `delete_with_body<B: Serialize>(&self, path: &str, body: &B) -> Result<()>` method. The ClickUp dependency DELETE endpoint requires a JSON body — this is unusual but documented. |
| 9 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_add_dependency` — verify POST body contains `depends_on` and `dependency_of`. |
| 10 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_remove_dependency` — verify DELETE with body. |
| 11 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_add_task_link` — verify POST to correct path `/task/{id}/link/{links_to}`. |
| 12 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_remove_task_link` — verify DELETE to correct path. |

#### Acceptance Criteria

- [ ] GIVEN task A and task B, WHEN `client.add_dependency("A", "B")` is called, THEN `POST /task/A/dependency` is sent with `{"depends_on":"B","dependency_of":"A"}`.
- [ ] GIVEN an existing dependency, WHEN `client.remove_dependency("A", "B")` is called, THEN `DELETE /task/A/dependency` is sent with the correct body.
- [ ] GIVEN task A and task C, WHEN `client.add_task_link("A", "C")` is called, THEN `POST /task/A/link/C` is sent.
- [ ] GIVEN an existing link, WHEN `client.remove_task_link("A", "C")` is called, THEN `DELETE /task/A/link/C` is sent.
- [ ] GIVEN the DELETE endpoint requires a body, WHEN `remove_dependency()` is called, THEN the HTTP DELETE request includes the JSON body (not just path params).

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/endpoints/comments.rs` (endpoint exemplar), `crates/clickup-api/src/models/linked_task.rs` (existing LinkedTask/TaskDependency models), `crates/clickup-api/src/client.rs` (HTTP methods — check if delete() supports body)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Wiremock tests for all 4 endpoints
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean
  - [ ] P2: `///` doc comments on all public methods and structs

---

### S-2: CLI Commands

**As a** CLI user, **I want** `clickup task depend TASK_ID --waiting-on OTHER_ID` to create dependencies and `clickup task link TASK_ID OTHER_ID` to create links, **so that** I can manage task relationships from the command line.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-cli/src/commands/tasks.rs` | Add `Depend` variant to `TaskCommands`: `task_id: String`, `#[arg(long)] waiting_on: Option<String>`, `#[arg(long)] blocks: Option<String>`, `#[arg(long)] remove: bool` (if set, removes instead of adds). |
| 2 | Create | `crates/clickup-cli/src/commands/tasks.rs` | Implement `depend_task()` handler: if `--waiting-on` → `client.add_dependency(task_id, waiting_on_id)` (task depends on other); if `--blocks` → `client.add_dependency(blocks_id, task_id)` (other depends on task). If `--remove` → call `remove_dependency()` instead. Print ✓ with relationship direction. |
| 3 | Extend | `crates/clickup-cli/src/commands/tasks.rs` | Add `Link` variant: `task_id: String`, `other_id: String`. Add `Unlink` variant: `task_id: String`, `other_id: String`. |
| 4 | Create | `crates/clickup-cli/src/commands/tasks.rs` | Implement `link_task()`: `client.add_task_link()` → print `✓ Linked {task_id} ↔ {other_id}`. Implement `unlink_task()`: `client.remove_task_link()` → print `✓ Unlinked {task_id} ↔ {other_id}`. |
| 5 | Modify | `crates/clickup-cli/src/main.rs` | Wire `TaskCommands::Depend`, `TaskCommands::Link`, `TaskCommands::Unlink` in dispatch. |

#### Acceptance Criteria

- [ ] GIVEN `clickup task depend abc123 --waiting-on xyz789`, WHEN executed, THEN `add_dependency("abc123", "xyz789")` is called and `✓ Task abc123 now depends on xyz789` is printed.
- [ ] GIVEN `clickup task depend abc123 --blocks xyz789`, WHEN executed, THEN `add_dependency("xyz789", "abc123")` is called (reverse direction) and `✓ Task abc123 now blocks xyz789` is printed.
- [ ] GIVEN `clickup task depend abc123 --waiting-on xyz789 --remove`, WHEN executed, THEN `remove_dependency("abc123", "xyz789")` is called.
- [ ] GIVEN `clickup task link abc123 xyz789`, WHEN executed, THEN `add_task_link("abc123", "xyz789")` is called and `✓ Linked abc123 ↔ xyz789` is printed.
- [ ] GIVEN `clickup task unlink abc123 xyz789`, WHEN executed, THEN `remove_task_link("abc123", "xyz789")` is called.
- [ ] GIVEN neither `--waiting-on` nor `--blocks` is provided for `depend`, WHEN executed, THEN an error is printed.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (TaskCommands enum), `crates/clickup-cli/src/main.rs` (dispatch)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-3: TUI Dependency and Link Management

**As a** TUI user on the TaskDetail screen, **I want** to add links via `L` (task ID input dialog) and remove dependencies/links via `d` when focused on a relationship item, **so that** I can manage task relationships within the TUI.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-tui/src/app.rs` | Add state: `link_dialog_open: bool`, `link_dialog_input: String`, `link_dialog_mode: LinkDialogMode` (enum: `AddLink`, `AddDependency { waiting: bool }`). Add `dependency_focus: Option<DependencyFocus>` for cursor tracking in the dependency/link section. |
| 2 | Modify | `crates/clickup-tui/src/ui/task_detail.rs` | In dependency/link sections: when `dependency_focus` matches a row, highlight it. Show keybind hints in the section: `L: add link | D: add dependency | d: remove`. |
| 3 | Create | `crates/clickup-tui/src/ui/link_dialog.rs` | Render floating dialog: "Enter Task ID" text input. For dependency: additional toggle "Direction: waiting on ⇄ blocks" (switchable with Tab). Enter submits, Esc cancels. |
| 4 | Modify | `crates/clickup-tui/src/ui/mod.rs` | Add `pub mod link_dialog;` declaration. |
| 5 | Extend | `crates/clickup-tui/src/input.rs` | In `handle_task_detail()`: `L` key → open link dialog (AddLink mode). `D` key → open link dialog (AddDependency mode). When cursor is on a dependency/link row, `d` key → confirm deletion → spawn remove API call. |
| 6 | Extend | `crates/clickup-tui/src/data.rs` | Add `spawn_add_dependency()`, `spawn_remove_dependency()`, `spawn_add_task_link()`, `spawn_remove_task_link()`: each calls the corresponding API method → sends success/error event → refresh task detail. |
| 7 | Extend | `crates/clickup-tui/src/event.rs` | Add `DataPayload::DependencyAdded`, `DataPayload::DependencyRemoved`, `DataPayload::LinkAdded`, `DataPayload::LinkRemoved` variants. Handle: refresh task detail to show updated relationships. |

#### Acceptance Criteria

- [ ] GIVEN the user presses `L` on TaskDetail, WHEN the link dialog opens, THEN a text input for "Task ID" is shown.
- [ ] GIVEN the user enters task ID "xyz789" and presses Enter in AddLink mode, WHEN submitted, THEN `add_task_link(current_task_id, "xyz789")` is called.
- [ ] GIVEN the user presses `D` on TaskDetail, WHEN the dependency dialog opens, THEN a text input with a direction toggle is shown.
- [ ] GIVEN the user enters "xyz789" with direction "waiting on" and presses Enter, WHEN submitted, THEN `add_dependency(current_task_id, "xyz789")` is called.
- [ ] GIVEN the cursor is on a linked task in the links section, WHEN the user presses `d`, THEN a confirmation is shown and `remove_task_link()` is called on confirm.
- [ ] GIVEN the cursor is on a dependency, WHEN the user presses `d`, THEN `remove_dependency()` is called with the correct direction.
- [ ] GIVEN a successful add/remove, WHEN the response arrives, THEN the task detail refreshes showing the updated relationships.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/task_detail.rs` (existing dependency/link rendering), `crates/clickup-tui/src/input.rs` (key routing), `crates/clickup-tui/src/data.rs` (spawn pattern), `crates/clickup-tui/src/ui/comment_sidebar.rs` (dialog/input patterns)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

## Execution Sequence

```
S-1 (API, 4 endpoints) ──┬──→ S-2 (CLI commands)
                          └──→ S-3 (TUI management)
```

- **Phase 1:** S-1 — All 4 API endpoints
- **Phase 2 (parallel):** S-2 (CLI) + S-3 (TUI)

## Assumptions

1. The ClickUp `DELETE /task/{id}/dependency` endpoint requires a JSON body with `depends_on` and `dependency_of` fields. This is unusual for a DELETE endpoint but is documented in the ClickUp API. **Risk if wrong:** May use query params instead — adjust to `delete_with_params()`.
2. The existing `delete()` method on `ClickUpClient` does not support sending a request body. A new `delete_with_body()` method may be needed. **Risk if wrong:** If `delete()` already supports bodies via reqwest, no change needed.
3. The `POST /task/{id}/link/{links_to}` endpoint returns an empty body or minimal success response. **Risk if wrong:** Add response struct — low impact.
4. The existing `LinkedTask` and `TaskDependency` models in the codebase accurately reflect the API response format. After add/remove operations, refreshing the task detail will show updated relationships. **Risk if wrong:** May need to manually update the local state instead of re-fetching.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | 4 endpoints, CLI commands, TUI dialogs — moderate surface area |
| Ambiguity | 1 | Dependency direction (depends_on vs dependency_of) is well-defined; link API is simple |
| Dependencies | 1 | No feature dependencies; builds on existing models and TUI patterns |
| Risk | 1 | The DELETE-with-body requirement is the main risk; everything else is straightforward |

**Total: 5/12** → Standard processing
