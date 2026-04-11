# F08 — Assignee Management

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 2 — Task Editing
> **Complexity:** 6/12 | **Confidence:** 85%

---

## Problem Statement

Users cannot assign or unassign people from tasks without leaving the terminal. The ClickUp API uses a unique `assignees` pattern in `PUT /task/{id}` where the body contains `{ "assignees": { "add": [USER_ID], "rem": [USER_ID] } }` — an object with separate `add` and `rem` arrays rather than a flat replacement list. This requires a dedicated `AssigneeUpdate` serialization model. The TUI currently displays assignees on TaskDetail but provides no way to modify them.

## Approach

Create an `AssigneeUpdate` struct with `add` and `rem` fields, integrate it into `UpdateTaskRequest`, and build CLI/TUI interfaces. The CLI provides explicit `--add` and `--remove` flags for user IDs. The TUI presents a floating member picker (similar to the existing `@mention` picker pattern from comment compose) where workspace members are listed with checkboxes — Space toggles selection, Enter applies changes, calculating the diff between current and selected assignees to produce the `add`/`rem` arrays.

### Rejected Alternatives

1. **Flat assignee list replacement** — Send the full desired assignee list and let the API figure out the diff. The ClickUp API does not support this format for `PUT /task/{id}` — it requires explicit `add`/`rem` arrays. Rejected for API incompatibility.

2. **Separate assign/unassign API calls** — Make individual `POST /task/{id}/assignee` calls for each add/remove. These endpoints don't exist in the ClickUp API v2; assignee changes go through the main task update endpoint. Rejected for API mismatch.

3. **Username-based assignment (CLI)** — Allow `--add "username"` instead of user IDs. Requires an extra API call to resolve username → user ID, and usernames may not be unique across workspaces. Rejected for complexity; user IDs are shown in `clickup workspace list` output.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `AssigneeUpdate` model with `add` and `rem` arrays | Username → user ID resolution | Watcher management (separate API) |
| Integration into `UpdateTaskRequest.assignees` | Assignee role permissions (viewer vs editor) | Assignee suggestions based on history |
| CLI `clickup task assign TASK_ID --add ID --remove ID` | Team/group assignment | Bulk assign across multiple tasks |
| TUI member picker with multi-select | Member invitation | |
| Diff calculation (current vs selected → add/rem) | | |

## Stories

### S-1: AssigneeUpdate Model

**As a** developer using clickup-api, **I want** an `AssigneeUpdate` struct that serializes to `{ "add": [...], "rem": [...] }`, **so that** the `UpdateTaskRequest` can correctly express assignee changes.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** F06/S-1 (UpdateTaskRequest exists)

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/src/models/task.rs` | Add `AssigneeUpdate` struct: `add: Vec<i64>`, `rem: Vec<i64>` — both derive `Serialize, Deserialize, Debug, Clone, Default`. Use `#[serde(skip_serializing_if = "Vec::is_empty")]` on both fields. |
| 2 | Extend | `crates/clickup-api/src/models/task.rs` | Add `assignees: Option<AssigneeUpdate>` to `UpdateTaskRequest` with `#[serde(skip_serializing_if = "Option::is_none")]`. |
| 3 | Test | `crates/clickup-api/src/models/task.rs` | Unit test: `AssigneeUpdate { add: vec![123], rem: vec![456] }` serializes to `{"add":[123],"rem":[456]}`. |
| 4 | Test | `crates/clickup-api/src/models/task.rs` | Unit test: `AssigneeUpdate { add: vec![123], rem: vec![] }` serializes to `{"add":[123]}` (empty `rem` omitted). |
| 5 | Test | `crates/clickup-api/src/models/task.rs` | Unit test: `UpdateTaskRequest` with only `assignees` set serializes correctly; all other fields are absent. |

#### Acceptance Criteria

- [ ] GIVEN an `AssigneeUpdate` with `add: [123, 456]` and `rem: [789]`, WHEN serialized, THEN the JSON is `{"add":[123,456],"rem":[789]}`.
- [ ] GIVEN an `AssigneeUpdate` with `add: [123]` and `rem: []`, WHEN serialized, THEN the JSON is `{"add":[123]}` — empty arrays are omitted.
- [ ] GIVEN an `UpdateTaskRequest` with only `assignees` set, WHEN serialized, THEN the JSON contains only the `assignees` object and no other fields.
- [ ] GIVEN an `UpdateTaskRequest` with no fields set (including `assignees: None`), WHEN serialized, THEN `assignees` is absent from the JSON.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/models/task.rs` (UpdateTaskRequest, existing serde patterns), `crates/clickup-api/src/models/comment.rs` (skip_serializing_if exemplar)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Serialization tests for add-only, rem-only, both, empty
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean
  - [ ] P2: `///` doc comments on `AssigneeUpdate`

---

### S-2: CLI Assign Command

**As a** CLI user, **I want** to run `clickup task assign TASK_ID --add 123 --add 456 --remove 789`, **so that** I can manage task assignees from the command line.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-cli/src/commands/tasks.rs` | Add `Assign` variant to `TaskCommands` enum: `task_id: String`, `#[arg(long)] add: Vec<i64>`, `#[arg(long)] remove: Vec<i64>`. Both `add` and `remove` accept repeated flags. |
| 2 | Create | `crates/clickup-cli/src/commands/tasks.rs` | Add `assign_task()` handler: validate at least one `--add` or `--remove` is provided → build `UpdateTaskRequest` with `assignees: Some(AssigneeUpdate { add, rem: remove })` → call `client.update_task()` → print success listing added/removed IDs with ✓ prefix. |
| 3 | Modify | `crates/clickup-cli/src/main.rs` | Wire `TaskCommands::Assign` in command dispatch. |
| 4 | Test | `crates/clickup-cli/src/commands/tasks.rs` | Verify clap parsing: `--add 1 --add 2 --remove 3` produces `add: [1, 2], remove: [3]`. |

#### Acceptance Criteria

- [ ] GIVEN `clickup task assign abc123 --add 100 --add 200`, WHEN executed, THEN a `PUT /task/abc123` is sent with `{"assignees":{"add":[100,200]}}`.
- [ ] GIVEN `clickup task assign abc123 --remove 100`, WHEN executed, THEN a `PUT /task/abc123` is sent with `{"assignees":{"rem":[100]}}`.
- [ ] GIVEN `clickup task assign abc123 --add 100 --remove 200`, WHEN executed, THEN both `add` and `rem` arrays are present in the request body.
- [ ] GIVEN no `--add` or `--remove` flags, WHEN executed, THEN the CLI prints an error: "At least one --add or --remove is required".
- [ ] GIVEN a successful API response, WHEN displayed, THEN the CLI prints `✓ Task abc123 assignees updated`.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (TaskCommands enum, Vec arg pattern), `crates/clickup-cli/src/main.rs` (dispatch)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-3: TUI Assignee Picker

**As a** TUI user on the TaskDetail screen, **I want** to press `a` to open a member picker where I can add/remove assignees with Space to toggle and Enter to apply, **so that** I can manage task assignees without leaving the TUI.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-tui/src/app.rs` | Add state fields: `assignee_picker_open: bool`, `assignee_picker_selected: usize`, `assignee_picker_toggled: HashSet<i64>` (user IDs currently toggled on). Initialize `toggled` from the current task's `assignees` list when the picker opens. |
| 2 | Create | `crates/clickup-tui/src/ui/assignee_picker.rs` | Render a floating centered panel listing workspace members. Each row: `[x]` or `[ ]` checkbox + user display name + username. Current selection highlighted. Bottom bar: `Space: toggle | Enter: apply | Esc: cancel`. |
| 3 | Modify | `crates/clickup-tui/src/ui/mod.rs` | Add `pub mod assignee_picker;` declaration. |
| 4 | Extend | `crates/clickup-tui/src/input.rs` | In `handle_task_detail()`: press `a` → open assignee picker (populate toggled set from current task assignees). When picker is open, route keys to `handle_assignee_picker()`: `j`/`↓` next, `k`/`↑` prev, `Space` toggle, `Enter` compute diff (compare toggled vs original → build `AssigneeUpdate`), `Esc` cancel. |
| 5 | Extend | `crates/clickup-tui/src/data.rs` | Reuse `spawn_update_task()` from F06/S-3 to send the `UpdateTaskRequest` with assignees. On `TaskUpdated` response, refresh task detail and close picker. |
| 6 | Modify | `crates/clickup-tui/src/ui/task_detail.rs` | When `assignee_picker_open`, render the picker overlay on top of the task detail using a `Clear` widget + floating `Block`. |
| 7 | Test | `crates/clickup-tui/src/ui/assignee_picker.rs` | Render test using `TestBackend`: verify picker shows member list with checkboxes, currently assigned members are pre-checked. |

#### Acceptance Criteria

- [ ] GIVEN the user is on TaskDetail with a task assigned to users [100, 200], WHEN they press `a`, THEN the picker opens with users 100 and 200 pre-checked.
- [ ] GIVEN the picker is open and user 300 is unchecked, WHEN the user navigates to user 300 and presses Space, THEN user 300 becomes checked.
- [ ] GIVEN the user has checked user 300 and unchecked user 200 (originally assigned), WHEN they press Enter, THEN a `PUT /task/{id}` is sent with `{"assignees":{"add":[300],"rem":[200]}}`.
- [ ] GIVEN no changes were made in the picker, WHEN Enter is pressed, THEN no API call is made and the picker closes.
- [ ] GIVEN the picker is open, WHEN Esc is pressed, THEN the picker closes with no changes.
- [ ] GIVEN the workspace has members loaded in `app.workspaces[current].members`, WHEN the picker opens, THEN all workspace members are listed.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/app.rs` (App struct, member data access pattern), `crates/clickup-tui/src/ui/comment_sidebar.rs` (mention picker as exemplar for floating picker UI), `crates/clickup-tui/src/input.rs` (modal input routing pattern from CommentInputMode)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P1: Render tests for picker widget
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

## Execution Sequence

```
S-1 (model) ──┬──→ S-2 (CLI)
              └──→ S-3 (TUI picker)
```

- **Phase 1:** S-1 — AssigneeUpdate model
- **Phase 2 (parallel):** S-2 (CLI assign command) + S-3 (TUI assignee picker)

## Assumptions

1. Workspace members are already loaded and accessible via `app.workspaces[current_workspace_index].members` in the TUI. The `Workspace` model includes a `members` field with `Vec<WorkspaceMember>` containing user IDs and display info. **Risk if wrong:** Must add a `spawn_load_members()` call and a `GET /team/{id}/member` endpoint — adds ~1d.
2. The ClickUp API user IDs are `i64` integers, consistent with the `User.id` type in the existing model. **Risk if wrong:** May need `String` IDs with string-or-number deserialization.
3. The `add` and `rem` arrays in the `assignees` object are processed atomically by the ClickUp API — adding and removing in the same request works correctly. **Risk if wrong:** May need to split into two sequential API calls.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | 3 crates affected but small surface per crate; model is simple |
| Ambiguity | 1 | ClickUp API docs clearly specify the add/rem pattern; well-defined |
| Dependencies | 2 | Depends on F06's UpdateTaskRequest and update_task(); reuses TUI picker pattern |
| Risk | 1 | Additive changes; the assignee add/rem serialization is the only tricky part |

**Total: 6/12** → Standard processing
