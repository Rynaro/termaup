# F10 — Checklist Management

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 3 — Task Enrichment
> **Complexity:** 6/12 | **Confidence:** 87%

---

## Problem Statement

Checklists are a core ClickUp feature for breaking tasks into actionable steps. termaup currently renders checklists on the TaskDetail screen (showing items with `[x]`/`[ ]` indicators and a progress bar) but provides no way to create checklists, add items, toggle item completion, or delete items. This is a frequent source of frustration for users who need to quickly check off items as they work through a task.

## Approach

Implement the full checklist CRUD API (6 endpoints), CLI commands for each operation, and TUI inline interactions on the TaskDetail screen. The TUI focus is on the most common operations: toggling item completion (Space key) and adding items (A key) — these should feel instant and natural within the existing TaskDetail scroll view. The CLI provides a `clickup checklist` command group with subcommands.

### Rejected Alternatives

1. **Dedicated TUI checklist screen** — Navigate to a separate screen for checklist management. This breaks the flow of viewing a task and adds unnecessary navigation. Rejected; inline editing on TaskDetail is more natural.

2. **Batch API calls** — Collect all changes and send them in one request. The ClickUp API doesn't support batch checklist operations — each item toggle is a separate PUT. Rejected for API incompatibility; individual calls with optimistic UI updates provide better UX.

3. **Reuse UpdateTaskRequest for checklists** — Include checklist changes in the task update endpoint. The ClickUp API uses dedicated checklist endpoints, not the task update endpoint. Rejected for API mismatch.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| 6 checklist API endpoints (create/update/delete checklist + create/update/delete item) | Checklist item reordering | Nested checklist items (parent field) |
| CLI `checklist` command group | Checklist item assignees | Checklist templates |
| TUI Space to toggle item, `A` to add item, `d` to delete | Drag-and-drop reordering | Checklist progress notifications |
| TUI checklist creation dialog | Checklist item due dates | |
| Optimistic UI updates for toggle | | |

## Stories

### S-1: API Endpoints (6 Endpoints)

**As a** developer using clickup-api, **I want** complete CRUD methods for checklists and checklist items, **so that** CLI and TUI can manage checklists programmatically.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/src/endpoints/checklists.rs` | Add `create_checklist(&self, task_id: &str, name: &str) -> Result<Checklist>`: `POST /task/{task_id}/checklist` with `{"name": "..."}`. Log with `tracing::debug!`. |
| 2 | Extend | `crates/clickup-api/src/endpoints/checklists.rs` | Add `update_checklist(&self, checklist_id: &str, name: Option<&str>, position: Option<i32>) -> Result<Checklist>`: `PUT /checklist/{checklist_id}` with optional name and position fields. |
| 3 | Extend | `crates/clickup-api/src/endpoints/checklists.rs` | Add `delete_checklist(&self, checklist_id: &str) -> Result<()>`: `DELETE /checklist/{checklist_id}`. |
| 4 | Extend | `crates/clickup-api/src/endpoints/checklists.rs` | Add `create_checklist_item(&self, checklist_id: &str, name: &str, assignee: Option<i64>) -> Result<Checklist>`: `POST /checklist/{checklist_id}/checklist_item` with `{"name": "...", "assignee": null}`. |
| 5 | Extend | `crates/clickup-api/src/endpoints/checklists.rs` | Add `update_checklist_item(&self, checklist_id: &str, item_id: &str, resolved: Option<bool>, name: Option<&str>) -> Result<Checklist>`: `PUT /checklist/{checklist_id}/checklist_item/{item_id}`. |
| 6 | Extend | `crates/clickup-api/src/endpoints/checklists.rs` | Add `delete_checklist_item(&self, checklist_id: &str, item_id: &str) -> Result<()>`: `DELETE /checklist/{checklist_id}/checklist_item/{item_id}`. |
| 7 | Create | `crates/clickup-api/src/models/checklist.rs` | Add request structs: `CreateChecklistRequest { name: String }`, `UpdateChecklistRequest { name: Option<String>, position: Option<i32> }`, `CreateChecklistItemRequest { name: String, assignee: Option<i64> }`, `UpdateChecklistItemRequest { resolved: Option<bool>, name: Option<String> }`. All with `#[serde(skip_serializing_if = "Option::is_none")]`. |
| 8 | Modify | `crates/clickup-api/src/endpoints/mod.rs` | Add `pub mod checklists;` declaration. |
| 9 | Test | `crates/clickup-api/tests/fixtures/checklist_response.json` | Create fixture: checklist object with items, matching existing `Checklist` model shape. |
| 10 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock tests: `test_create_checklist`, `test_toggle_checklist_item`, `test_delete_checklist_item` — mock endpoints and verify request/response. |

#### Acceptance Criteria

- [ ] GIVEN a task ID, WHEN `client.create_checklist("task1", "My Checklist")` is called, THEN `POST /task/task1/checklist` is sent with `{"name":"My Checklist"}` and a `Checklist` is returned.
- [ ] GIVEN a checklist ID and item ID, WHEN `client.update_checklist_item("cl1", "item1", Some(true), None)` is called, THEN `PUT /checklist/cl1/checklist_item/item1` is sent with `{"resolved":true}`.
- [ ] GIVEN a checklist ID, WHEN `client.delete_checklist("cl1")` is called, THEN `DELETE /checklist/cl1` is sent and `Ok(())` is returned.
- [ ] GIVEN a checklist ID, WHEN `client.create_checklist_item("cl1", "New Item", None)` is called, THEN `POST /checklist/cl1/checklist_item` is sent with `{"name":"New Item","assignee":null}`.
- [ ] GIVEN all 6 endpoints, WHEN called with valid parameters, THEN each includes a `tracing::debug!` log line.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/endpoints/comments.rs` (full CRUD exemplar), `crates/clickup-api/src/models/checklist.rs` (existing Checklist/ChecklistItem models), `crates/clickup-api/src/client.rs` (HTTP helpers: post, put, delete)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Wiremock tests for create, toggle, and delete operations
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean
  - [ ] P2: `///` doc comments on all 6 endpoint methods

---

### S-2: CLI Checklist Commands

**As a** CLI user, **I want** a `clickup checklist` command group with subcommands for creating checklists, adding items, toggling completion, and deleting, **so that** I can manage checklists from the command line.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-cli/src/commands/checklists.rs` | Add `ChecklistCommands` enum with variants: `Create { #[arg(long)] task: String, #[arg(long)] name: String }`, `Delete { checklist_id: String }`, `AddItem { #[arg(long)] checklist: String, #[arg(long)] name: String }`, `ToggleItem { #[arg(long)] checklist: String, #[arg(long)] item: String }`, `RemoveItem { #[arg(long)] checklist: String, #[arg(long)] item: String }`. |
| 2 | Create | `crates/clickup-cli/src/commands/checklists.rs` | Implement `run()` for each variant: Create → `client.create_checklist()` → print ✓ with checklist name; AddItem → `client.create_checklist_item()` → print ✓; ToggleItem → fetch current resolved state, flip it, call `client.update_checklist_item()` → print ✓ with new state; RemoveItem → `client.delete_checklist_item()` → print ✓; Delete → `client.delete_checklist()` → print ✓. |
| 3 | Modify | `crates/clickup-cli/src/commands/mod.rs` | Add `pub mod checklists;` declaration. |
| 4 | Modify | `crates/clickup-cli/src/main.rs` | Add `Checklist(ChecklistCommands)` variant to root `Commands` enum and wire dispatch. |

#### Acceptance Criteria

- [ ] GIVEN `clickup checklist create --task abc123 --name "QA Steps"`, WHEN executed, THEN a checklist named "QA Steps" is created on the task and `✓ Checklist "QA Steps" created` is printed.
- [ ] GIVEN `clickup checklist add-item --checklist cl1 --name "Step 1"`, WHEN executed, THEN an item is added and `✓ Item "Step 1" added to checklist` is printed.
- [ ] GIVEN `clickup checklist toggle-item --checklist cl1 --item item1`, WHEN executed, THEN the item's resolved state is toggled and `✓ Item toggled to [x]` (or `[ ]`) is printed.
- [ ] GIVEN `clickup checklist remove-item --checklist cl1 --item item1`, WHEN executed, THEN the item is deleted and `✓ Item removed` is printed.
- [ ] GIVEN `clickup checklist delete cl1`, WHEN executed, THEN the checklist is deleted and `✓ Checklist deleted` is printed.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (command group exemplar), `crates/clickup-cli/src/commands/comments.rs` (CRUD command exemplar), `crates/clickup-cli/src/main.rs` (dispatch pattern)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-3: TUI Checklist Interaction

**As a** TUI user on the TaskDetail screen, **I want** to toggle checklist items with Space, add items with `A`, and delete items with `d`, **so that** I can manage checklists inline without leaving the task view.

**Timebox:** ≤3d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-tui/src/app.rs` | Add state: `checklist_focus: Option<ChecklistFocus>` where `ChecklistFocus { checklist_index: usize, item_index: Option<usize> }`. This tracks which checklist/item the cursor is on when navigating within the checklist section. |
| 2 | Extend | `crates/clickup-tui/src/app.rs` | Add `add_item_input: Option<String>` for the inline text input when adding a new item. |
| 3 | Modify | `crates/clickup-tui/src/ui/task_detail.rs` | In checklist rendering: if `checklist_focus` matches the current checklist/item, highlight the row. Show `[x]`/`[ ]` with cursor indicator. When `add_item_input` is `Some`, render an inline text input at the bottom of the focused checklist. |
| 4 | Extend | `crates/clickup-tui/src/input.rs` | In `handle_task_detail()`: when cursor is on a checklist item, `Space` → toggle resolved (optimistic UI update → spawn `spawn_toggle_checklist_item()`), `d` → confirm delete → spawn `spawn_delete_checklist_item()`, `A` → set `add_item_input = Some(String::new())` to show inline input → Enter confirms → spawn `spawn_create_checklist_item()`. |
| 5 | Extend | `crates/clickup-tui/src/data.rs` | Add `spawn_toggle_checklist_item()`, `spawn_create_checklist_item()`, `spawn_delete_checklist_item()` following the existing spawn pattern. Each sends appropriate `DataPayload` variant on completion. |
| 6 | Extend | `crates/clickup-tui/src/event.rs` | Add `DataPayload::ChecklistUpdated(Box<Checklist>)` variant. Handle: replace the checklist in the current task's checklists vec. |
| 7 | Extend | `crates/clickup-tui/src/event.rs` | Add `DataPayload::ChecklistItemDeleted { checklist_id: String, item_id: String }` variant. Handle: remove the item from the checklist. |

#### Acceptance Criteria

- [ ] GIVEN the user is on TaskDetail viewing a checklist, WHEN they navigate to an item and press Space, THEN the item toggles between `[x]` and `[ ]` immediately (optimistic update) and an API call is made.
- [ ] GIVEN the user presses Space and the API call fails, WHEN the error arrives, THEN the optimistic update is reverted and an error message is shown.
- [ ] GIVEN the user navigates to a checklist and presses `A`, WHEN they type "New Step" and press Enter, THEN a new item "New Step" is added to the checklist via API.
- [ ] GIVEN the user navigates to a checklist item and presses `d`, WHEN confirmed, THEN the item is removed from the checklist via API.
- [ ] GIVEN the checklist section is not focused, WHEN Space/A/d are pressed, THEN they have no effect (these keys only work within the checklist section).
- [ ] GIVEN a checklist with 5 items, 3 resolved, WHEN rendered, THEN the progress bar shows 3/5 (60%).

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/task_detail.rs` (existing checklist rendering with progress bar), `crates/clickup-tui/src/input.rs` (key routing), `crates/clickup-tui/src/data.rs` (spawn pattern from comments CRUD), `crates/clickup-tui/src/app.rs` (state management)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-4: Integration Tests

**As a** maintainer, **I want** wiremock tests covering all 6 checklist endpoints, **so that** regressions are caught.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/tests/fixtures/checklist_created.json` | Fixture: checklist object with name and empty items array. |
| 2 | Create | `crates/clickup-api/tests/fixtures/checklist_with_items.json` | Fixture: checklist with 3 items, 1 resolved. |
| 3 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_create_checklist` — POST to `/task/{id}/checklist`, verify body and response. |
| 4 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_update_checklist_item_toggle` — PUT to `/checklist/{id}/checklist_item/{id}`, verify `resolved` field. |
| 5 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_delete_checklist` — DELETE, verify `Ok(())`. |
| 6 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_create_checklist_item` — POST, verify name in body. |

#### Acceptance Criteria

- [ ] GIVEN all 6 endpoint methods, WHEN tested with wiremock, THEN each sends the correct HTTP method, path, and body.
- [ ] GIVEN the create endpoints, WHEN the mock returns a valid checklist JSON, THEN the response deserializes into a `Checklist` struct.
- [ ] GIVEN the delete endpoints, WHEN the mock returns 200, THEN `Ok(())` is returned.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/tests/fixture_tests.rs` (existing test patterns), `crates/clickup-api/tests/fixtures/` (fixture dir)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: All 6 endpoints have wiremock tests
  - [ ] P2: `cargo clippy --workspace -- -D warnings` clean

---

## Execution Sequence

```
S-1 (API, 6 endpoints) ──┬──→ S-2 (CLI commands)
                          ├──→ S-3 (TUI interaction)
                          └──→ S-4 (tests)
```

- **Phase 1:** S-1 — All 6 API endpoints and request models
- **Phase 2 (parallel):** S-2 (CLI) + S-3 (TUI) + S-4 (integration tests)

## Assumptions

1. The ClickUp `POST /task/{id}/checklist` endpoint returns the created `Checklist` object (matching the existing `Checklist` model). **Risk if wrong:** May return a wrapper object requiring a new response struct.
2. The `PUT /checklist/{id}/checklist_item/{id}` endpoint returns the updated `Checklist` (parent), not just the item. **Risk if wrong:** May need to fetch the full task to refresh checklist state.
3. Optimistic UI updates for toggle are safe because the resolved boolean is idempotent — retrying on failure just sends the same value again. **Risk if wrong:** Minimal — worst case is a brief visual inconsistency.
4. Existing `Checklist` and `ChecklistItem` models from `crates/clickup-api/src/models/checklist.rs` are compatible with the create/update response format. **Risk if wrong:** May need minor field additions — low impact.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | 6 endpoints + CLI + TUI, but each is small and follows established patterns |
| Ambiguity | 1 | ClickUp checklist API is well-documented; existing models already fit |
| Dependencies | 2 | No feature dependencies, but builds on existing TaskDetail rendering and comment CRUD spawn pattern |
| Risk | 1 | All additive; existing models reduce deserialization risk |

**Total: 6/12** → Standard processing
