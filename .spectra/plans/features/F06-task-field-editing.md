# F06 — Task Field Editing

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 2 — Task Editing
> **Complexity:** 8/12 | **Confidence:** 82%

---

## Problem Statement

Users can view task details in both the CLI and TUI but cannot modify basic task fields — name, priority, due date, start date, or time estimate — without leaving the terminal and opening the ClickUp web app. F03 introduces the `update_task()` endpoint and a minimal `UpdateTaskRequest` struct (for status changes only), but it does not cover the full range of editable fields. This feature extends the update capability to all core task fields, creating the foundation that subsequent editing features (F07, F08, F09) build upon.

## Approach

Extend the existing `UpdateTaskRequest` model with `Option<T>` fields for each editable property, using `#[serde(skip_serializing_if = "Option::is_none")]` so only changed fields are sent in the PUT body. The CLI adds flags to the `task update` command. The TUI introduces an "edit mode" on the TaskDetail screen where pressing `e` toggles fields to editable state — Tab/Shift+Tab cycles fields, Enter confirms a field, Esc cancels, and Ctrl+S saves all changes via a single `PUT /task/{id}` call.

### Rejected Alternatives

1. **Separate endpoint per field** — Individual `update_task_name()`, `update_task_priority()`, etc. methods. This multiplies network requests for multi-field edits, increases API surface area, and violates the ClickUp API design which expects a single PUT with all changed fields. Rejected for inefficiency and API mismatch.

2. **Full-screen edit form (TUI)** — Replace the TaskDetail screen with a dedicated edit form screen. This loses the context of the task detail view and requires duplicating the layout. Rejected in favor of inline edit mode that preserves the existing TaskDetail layout while making fields editable in place.

3. **Interactive prompt workflow (CLI)** — Walk the user through each field with `dialoguer` prompts. Slower than flag-based updates for scripting and automation. Rejected as the primary interface; flags are more composable and scriptable.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| Extend `UpdateTaskRequest` with name, priority, due_date, start_date, time_estimate | Status field editing (handled by F03) | Assignee editing (F08) |
| CLI `task update` with `--name`, `--priority`, `--due-date`, `--start-date`, `--estimate` flags | Description editing (handled by F07) | Custom field editing (F09) |
| TUI edit mode on TaskDetail with inline field editing | Assignee editing (handled by F08) | Bulk field editing across multiple tasks |
| TUI field-specific input widgets (text, priority picker, date input, duration input) | Task creation (handled by F02) | Undo/redo for edits |
| Duration parser for time estimates (e.g., "2h30m") | Notification of changes to other users | |
| Validation of input values (priority 1-4, valid dates) | | |

## Stories

### S-1: Extend UpdateTaskRequest Model

**As a** developer using clickup-api, **I want** `UpdateTaskRequest` to support name, priority, due_date, start_date, and time_estimate fields, **so that** CLI and TUI code can construct partial update payloads for task field editing.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** F03 (provides base `UpdateTaskRequest` + `update_task()`)

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-api/src/models/task.rs` | Add fields to `UpdateTaskRequest`: `name: Option<String>`, `priority: Option<u8>` (1-4, serialized as `{"id": "N", "priority": "label"}`), `due_date: Option<i64>` (Unix ms), `start_date: Option<i64>` (Unix ms), `time_estimate: Option<u64>` (ms). All fields use `#[serde(skip_serializing_if = "Option::is_none")]`. |
| 2 | Create | `crates/clickup-api/src/models/task.rs` | Add helper method `UpdateTaskRequest::builder()` returning a builder or `Default` impl so callers can set only the fields they want. |
| 3 | Test | `crates/clickup-api/src/models/task.rs` | Unit test: serializing `UpdateTaskRequest` with only `name` set produces `{"name": "New Title"}` — no other fields. |
| 4 | Test | `crates/clickup-api/src/models/task.rs` | Unit test: serializing with all fields set produces the full JSON body. |
| 5 | Test | `crates/clickup-api/src/models/task.rs` | Unit test: serializing with `priority: Some(2)` produces the correct ClickUp priority format (integer or object depending on API expectation). |
| 6 | Test | `crates/clickup-api/src/models/task.rs` | Verify `due_date_millis` and `start_date_millis` fields serialize as integers (ClickUp expects Unix timestamps in milliseconds as numbers, NOT strings). |

#### Acceptance Criteria

- [ ] GIVEN an `UpdateTaskRequest` with only `name` set, WHEN serialized to JSON, THEN the output contains only `{"name":"New Title"}` with no null or default fields.
- [ ] GIVEN an `UpdateTaskRequest` with `priority` set to 2, WHEN serialized, THEN the output contains the correct priority representation expected by `PUT /task/{id}`.
- [ ] GIVEN an `UpdateTaskRequest` with `due_date` set to a Unix ms timestamp, WHEN serialized, THEN `due_date` is an integer (not a string).
- [ ] GIVEN an `UpdateTaskRequest` with `time_estimate` set to 9000000 (2h30m in ms), WHEN serialized, THEN `time_estimate` is an integer in milliseconds.
- [ ] GIVEN an empty `UpdateTaskRequest` (all fields `None`), WHEN serialized, THEN the output is `{}` or contains only the status field from F03.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/models/task.rs` (existing Task struct), `crates/clickup-api/src/models/comment.rs` (UpdateCommentRequest as exemplar for skip_serializing_if pattern)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected — no CLI/TUI deps in clickup-api
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Serialization round-trip tests for each field
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-2: CLI Task Update Command

**As a** CLI user, **I want** to run `clickup task update TASK_ID --name "New Title" --priority 2 --due-date "2026-05-01"`, **so that** I can update task fields without opening the web browser.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-cli/src/commands/tasks.rs` | Add `Update` variant to `TaskCommands` enum with args: `task_id: String`, `--name: Option<String>`, `--priority: Option<u8>`, `--due-date: Option<String>` (YYYY-MM-DD parsed to Unix ms), `--start-date: Option<String>`, `--estimate: Option<String>` (duration string parsed to ms). |
| 2 | Create | `crates/clickup-cli/src/commands/tasks.rs` | Add `update_task()` async handler: validate inputs → build `UpdateTaskRequest` → call `client.update_task()` → print success with green ✓ prefix or error with red ✗. |
| 3 | Create | `crates/clickup-cli/src/commands/tasks.rs` | Add `parse_duration()` helper: parse "2h30m", "1.5h", "90m", "1d" → milliseconds. |
| 4 | Create | `crates/clickup-cli/src/commands/tasks.rs` | Add `parse_date()` helper: parse "YYYY-MM-DD" → Unix timestamp in milliseconds using `chrono`. |
| 5 | Modify | `crates/clickup-cli/src/main.rs` | Wire `TaskCommands::Update` variant in the command dispatch match arm. |
| 6 | Test | `crates/clickup-cli/src/commands/tasks.rs` | Unit tests for `parse_duration()`: "2h30m"→9000000, "1.5h"→5400000, "90m"→5400000, "1d"→86400000, invalid→error. |
| 7 | Test | `crates/clickup-cli/src/commands/tasks.rs` | Unit tests for `parse_date()`: "2026-05-01"→valid ms, "invalid"→error. |

#### Acceptance Criteria

- [ ] GIVEN the command `clickup task update abc123 --name "New Title"`, WHEN executed, THEN the CLI calls `PUT /task/abc123` with `{"name":"New Title"}` and prints `✓ Task abc123 updated`.
- [ ] GIVEN the command `clickup task update abc123 --priority 2`, WHEN executed, THEN the CLI sends the correct priority value in the PUT body.
- [ ] GIVEN `--due-date "2026-05-01"`, WHEN parsed, THEN the date is converted to Unix milliseconds timestamp at start of day UTC.
- [ ] GIVEN `--estimate "2h30m"`, WHEN parsed, THEN the duration is converted to 9000000 milliseconds.
- [ ] GIVEN `--priority 5` (out of range 1-4), WHEN executed, THEN the CLI prints an error and exits without making an API call.
- [ ] GIVEN no optional flags provided, WHEN executed, THEN the CLI prints a help message indicating at least one field must be specified.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (existing TaskCommands enum), `crates/clickup-cli/src/commands/comments.rs` (exemplar for CRUD command pattern), `crates/clickup-cli/src/main.rs` (command dispatch)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P1: Duration and date parser unit tests
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-3: TUI Edit Mode Framework

**As a** TUI user on the TaskDetail screen, **I want** to press `e` to enter edit mode where fields become editable, **so that** I can modify task fields without leaving the TUI.

**Timebox:** ≤3d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-tui/src/app.rs` | Add `EditMode` enum: `Off`, `Editing { field_index: usize, fields: Vec<EditableField> }`. Add `edit_mode: EditMode` to `App` struct. Add `EditableField` struct: `label: String`, `value: String`, `original: String`, `field_type: EditFieldType`. |
| 2 | Extend | `crates/clickup-tui/src/app.rs` | Add `EditFieldType` enum: `Text`, `Priority`, `Date`, `Duration`. This determines which input widget renders for each field. |
| 3 | Extend | `crates/clickup-tui/src/app.rs` | Add methods: `App::enter_edit_mode()` (populates `EditableField` vec from current task), `App::cancel_edit_mode()` (restores to `EditMode::Off`), `App::save_edits()` (builds `UpdateTaskRequest` from changed fields). |
| 4 | Extend | `crates/clickup-tui/src/input.rs` | In `handle_task_detail()`, intercept `e` key to call `app.enter_edit_mode()`. When `edit_mode` is `Editing`, route all keys to `handle_edit_mode()`: Tab/Shift+Tab cycle `field_index`, Esc calls `cancel_edit_mode()`, Ctrl+S calls `save_edits()` then spawns the update. |
| 5 | Extend | `crates/clickup-tui/src/data.rs` | Add `spawn_update_task()` function following the existing `spawn_*` pattern: takes `ClickUpClient`, `mpsc::UnboundedSender<AppEvent>`, `task_id`, `UpdateTaskRequest` → calls `client.update_task()` → sends `DataPayload::TaskUpdated`. |
| 6 | Extend | `crates/clickup-tui/src/event.rs` | Add `DataPayload::TaskUpdated(Box<Task>)` variant. Handle in main loop: replace current task detail, show success message, exit edit mode. |
| 7 | Modify | `crates/clickup-tui/src/ui/task_detail.rs` | When `app.edit_mode` is `Editing`, render the current field with a cursor/highlight indicator instead of plain text. Non-focused fields show their edited (or original) value. Show bottom bar hint: `Tab: next field | Esc: cancel | Ctrl+S: save`. |

#### Acceptance Criteria

- [ ] GIVEN the user is on TaskDetail screen, WHEN they press `e`, THEN edit mode activates and the first editable field (name) is highlighted with a cursor.
- [ ] GIVEN edit mode is active, WHEN the user presses Tab, THEN focus moves to the next editable field; Shift+Tab moves to the previous field.
- [ ] GIVEN edit mode is active, WHEN the user presses Esc, THEN all changes are discarded and edit mode exits.
- [ ] GIVEN the user has modified some fields, WHEN they press Ctrl+S, THEN only the changed fields are sent in the PUT request (unchanged fields are `None`).
- [ ] GIVEN Ctrl+S is pressed and the API returns success, WHEN the response arrives, THEN the task detail refreshes with updated data and edit mode exits.
- [ ] GIVEN Ctrl+S is pressed and the API returns an error, WHEN the error arrives, THEN an error message is displayed and edit mode remains active (no data loss).
- [ ] GIVEN edit mode is active, WHEN global keys like `q` or `?` are pressed, THEN they are NOT processed (edit mode captures all input).

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/app.rs` (App struct, CommentInputMode as exemplar for modal state), `crates/clickup-tui/src/input.rs` (key routing), `crates/clickup-tui/src/data.rs` (spawn pattern), `crates/clickup-tui/src/event.rs` (DataPayload enum)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-4: TUI Field-Specific Input Widgets

**As a** TUI user in edit mode, **I want** field-appropriate input controls (text input for name, picker for priority, date input for dates, duration input for estimates), **so that** each field type has an intuitive editing experience.

**Timebox:** ≤3d | **Risk:** P2 | **Depends on:** S-3

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-tui/src/ui/edit_widgets.rs` | Create module with rendering functions for each field type: `render_text_input(text: &str, cursor_pos: usize, area: Rect, frame: &mut Frame)`, `render_priority_picker(current: u8, area: Rect, frame: &mut Frame)`, `render_date_input(text: &str, cursor_pos: usize, area: Rect, frame: &mut Frame)`, `render_duration_input(text: &str, cursor_pos: usize, area: Rect, frame: &mut Frame)`. |
| 2 | Modify | `crates/clickup-tui/src/ui/task_detail.rs` | In edit mode rendering, dispatch to the appropriate widget function based on `EditFieldType` of the focused field. |
| 3 | Extend | `crates/clickup-tui/src/input.rs` | In `handle_edit_mode()`, dispatch key events to field-type-specific handlers: text fields accept character input with cursor movement (Left/Right/Home/End/Backspace/Delete); priority picker accepts 1-4 or Up/Down cycling; date input validates format as typed; duration input accepts alphanumeric for "2h30m" format. |
| 4 | Modify | `crates/clickup-tui/src/ui/mod.rs` | Add `pub mod edit_widgets;` declaration. |
| 5 | Test | `crates/clickup-tui/src/ui/edit_widgets.rs` | Render tests using ratatui `TestBackend`: verify text input shows cursor at correct position, priority picker highlights current selection, date input shows placeholder format. |

#### Acceptance Criteria

- [ ] GIVEN edit mode on the name field, WHEN the user types characters, THEN the text updates inline with a visible cursor.
- [ ] GIVEN edit mode on the priority field, WHEN the user presses Up/Down or types 1-4, THEN the priority cycles through Urgent(1)/High(2)/Normal(3)/Low(4) with color-coded labels.
- [ ] GIVEN edit mode on a date field, WHEN the user types a date, THEN it shows a "YYYY-MM-DD" placeholder/mask and validates the format.
- [ ] GIVEN edit mode on the estimate field, WHEN the user types "2h30m", THEN the input is accepted; typing "abc" shows a validation indicator.
- [ ] GIVEN a text input with "Hello World" and cursor at position 5, WHEN the user presses Backspace, THEN the text becomes "Hell World" and cursor moves to position 4.
- [ ] GIVEN the priority picker showing "Normal (3)", WHEN the user presses Up, THEN it changes to "High (2)".

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/task_detail.rs` (rendering context), `crates/clickup-tui/src/ui/comment_sidebar.rs` (text input exemplar for compose mode), `crates/clickup-tui/src/app.rs` (EditMode state)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P1: Render tests for each widget type using `TestBackend`
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-5: Integration Tests and Fixtures

**As a** maintainer, **I want** wiremock-based integration tests covering task field updates, **so that** regressions in the update flow are caught automatically.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1, S-2

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/tests/fixtures/update_task_response.json` | Fixture JSON for `PUT /task/{id}` response: a full Task object reflecting the updated fields (name, priority, due_date, start_date, time_estimate). |
| 2 | Extend | `crates/clickup-api/tests/fixture_tests.rs` | Add wiremock test: `test_update_task_name_only` — mock `PUT /task/abc123` returning fixture, call `client.update_task("abc123", &req)`, assert request body contains only `name` field, assert response Task has updated name. |
| 3 | Extend | `crates/clickup-api/tests/fixture_tests.rs` | Add wiremock test: `test_update_task_all_fields` — all fields set, assert all are present in request body. |
| 4 | Extend | `crates/clickup-api/tests/fixture_tests.rs` | Add wiremock test: `test_update_task_invalid_priority` — verify priority outside 1-4 is rejected (if validation is in the model) or handled gracefully. |
| 5 | Test | `crates/clickup-cli/src/commands/tasks.rs` | Unit tests for duration parser edge cases: "0m"→0, "1d2h30m"→95400000, empty string→error, "5"→error (no unit). |
| 6 | Test | `crates/clickup-cli/src/commands/tasks.rs` | Unit tests for date parser edge cases: "2026-12-31"→valid, "2026-13-01"→error, "2026-02-30"→error. |

#### Acceptance Criteria

- [ ] GIVEN a wiremock server configured for `PUT /task/abc123`, WHEN `client.update_task()` is called with only `name` set, THEN the request body contains `{"name":"New Title"}` and no other fields.
- [ ] GIVEN a wiremock server returning a full task response, WHEN `client.update_task()` completes, THEN the returned `Task` struct has the updated field values.
- [ ] GIVEN the duration string "2h30m", WHEN parsed, THEN the result is 9000000 milliseconds.
- [ ] GIVEN the duration string "1d", WHEN parsed, THEN the result is 86400000 milliseconds.
- [ ] GIVEN an invalid date "2026-13-01", WHEN parsed, THEN an error is returned.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/tests/fixture_tests.rs` (existing test patterns), `crates/clickup-api/tests/fixtures/` (existing fixture files), `crates/clickup-api/src/endpoints/comments.rs` (update endpoint exemplar)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: New endpoints have wiremock tests
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy --workspace -- -D warnings` clean

---

## Execution Sequence

```
S-1 (model) ──┬──→ S-2 (CLI) ──→ S-5 (tests)
              └──→ S-3 (TUI edit framework) ──→ S-4 (input widgets)
```

- **Phase 1:** S-1 — Extend `UpdateTaskRequest` model (prerequisite for all)
- **Phase 2 (parallel):** S-2 (CLI update command) + S-3 (TUI edit mode framework)
- **Phase 3:** S-4 (field input widgets, after S-3)
- **Phase 4:** S-5 (integration tests, after S-1 and S-2)

## Assumptions

1. F03 has already delivered `UpdateTaskRequest` with at least a `status` field and the `update_task()` endpoint method on `ClickUpClient`. **Risk if wrong:** S-1 must create the entire struct and endpoint from scratch — adds ~1d to timebox.
2. The ClickUp API accepts `priority` as an integer (1-4) in the PUT body, not as the `{"id": "2", "priority": "high", "color": "#..."}` object format used in GET responses. **Risk if wrong:** Must add a custom serializer for the priority field.
3. The ClickUp API accepts `due_date` and `start_date` as Unix millisecond timestamps (integers) in the PUT body. **Risk if wrong:** May need string format — adjust serialization.
4. Users will want to clear a field (e.g., remove due date). Need `--clear-due-date` flag or special value like `"none"`. **Risk if wrong:** Deferred to follow-up if not handled in S-2.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 3 | 3 crates affected (API model + CLI command + TUI edit mode with widgets), 5 stories |
| Ambiguity | 2 | ClickUp priority serialization format unclear; edit mode UX details need design decisions |
| Dependencies | 2 | Hard dependency on F03 for `update_task()` endpoint; TUI edit mode is net-new UI pattern |
| Risk | 1 | Additive changes; no breaking API changes; text input is the riskiest widget but well-understood |

**Total: 8/12** → Careful processing with phased delivery
