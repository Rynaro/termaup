# F02 — Task Creation

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 1 — Critical MVP Gaps
> **Complexity:** 7/12 | **Confidence:** 82%

---

## Problem Statement

Users cannot create tasks from the terminal. The ClickUp API supports `POST /list/{list_id}/task`, but the `clickup-api` crate has no `create_task()` endpoint, no `CreateTaskRequest` model, and neither the CLI nor the TUI offers a creation workflow. For power users who live in the terminal, switching to the ClickUp web app to create a task breaks flow and defeats the purpose of the tool.

## Approach

Build task creation bottom-up: API model + endpoint first, then CLI command, then TUI creation dialog. The `CreateTaskRequest` model uses `#[serde(skip_serializing_if)]` on every optional field to keep the POST body minimal. The CLI exposes all fields as `--flags`. The TUI adds a modal form triggered by `n` on the TaskList screen, with name as the only required field.

### Rejected Alternatives

1. **Template-based creation** (`clickup task create --template FILE`) — Useful for complex tasks with many fields, but adds YAML/TOML parsing, template schema design, and validation. Over-scoped for MVP when flags cover the same ground. Deferred to a future enhancement.

2. **Interactive wizard in CLI** (`dialoguer` step-by-step prompts) — Nice UX but forces sequential interaction for something that works fine with flags. Users who want interactive creation can use the TUI. Rejected for duplicating TUI capabilities in a less capable medium.

3. **Full-form TUI with tab navigation** (multi-field form widget with field-level validation) — Significantly more complex; requires building a generic form widget, tab-stop management, field validation framework. A simpler modal with input fields and picker overlays achieves MVP goals. Deferred.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `CreateTaskRequest` model with all standard fields | Custom field values in create request | Bulk task creation |
| `create_task()` API endpoint | Recurring task configuration | Task templates |
| CLI `task create` subcommand with flags | File attachment during creation | Interactive CLI wizard |
| TUI creation modal on TaskList (`n` key) | Subtask creation from TUI | Full form widget with validation |
| Name (required), description, status, priority, assignee, due date | Time estimate, start date, tags | Custom field values |
| `wiremock` tests for API endpoint | | |

## Stories

### S-1: CreateTaskRequest model and API endpoint

**As a** developer using clickup-api, **I want** a `create_task()` method that POSTs to `/list/{list_id}/task`, **so that** both CLI and TUI can create tasks through a shared, tested interface.

**Timebox:** ≤2d | **Risk:** P0 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/src/models/task.rs` | Add `CreateTaskRequest` struct with: `name: String` (required), all other fields `Option<T>` with `#[serde(skip_serializing_if = "Option::is_none")]` — fields: `description`, `markdown_description`, `status`, `priority` (integer), `assignees` (`Option<Vec<i64>>`), `due_date` (ms timestamp string), `start_date`, `tags` (`Option<Vec<String>>`), `notify_all` (bool), `parent` (subtask creation) |
| 2 | Create | `crates/clickup-api/src/endpoints/tasks.rs` | Add `pub async fn create_task(&self, list_id: &str, request: &CreateTaskRequest) -> Result<Task>` — call `self.post(&format!("/list/{list_id}/task"), request)` with `tracing::debug!` logging, following `create_task_comment()` pattern from `endpoints/comments.rs` (line 29-37) |
| 3 | Test | `crates/clickup-api/tests/fixtures/create_task_response.json` | Create fixture with a realistic task response from the ClickUp API after creation |
| 4 | Test | `crates/clickup-api/src/endpoints/tasks.rs` or `crates/clickup-api/tests/` | Add `wiremock` test: mock `POST /list/123/task` → assert request body contains `name`, assert response deserializes to `Task`. Test with minimal request (name only) and full request (all fields populated) |
| 5 | Test | `crates/clickup-api/src/models/task.rs` | Add serialization tests: verify `skip_serializing_if` omits `None` fields, verify `name` is always present, verify `assignees` serializes as integer array |

#### Acceptance Criteria

- [ ] GIVEN a `CreateTaskRequest` with only `name` set WHEN serialized to JSON THEN the output contains `{"name": "..."}` with no other fields
- [ ] GIVEN a `CreateTaskRequest` with all fields populated WHEN serialized THEN all fields are present in the JSON output with correct types (priority as integer, due_date as string, assignees as integer array)
- [ ] GIVEN a valid list ID and request WHEN `create_task()` is called THEN it POSTs to `/list/{list_id}/task` and returns a deserialized `Task`
- [ ] GIVEN the API returns a 401 WHEN `create_task()` is called THEN it returns `ClickUpError::AuthError`
- [ ] GIVEN the API returns a 404 WHEN `create_task()` is called THEN it returns `ClickUpError::NotFound`

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/endpoints/comments.rs` (lines 29-37 `create_task_comment` as pattern), `crates/clickup-api/src/models/comment.rs` (lines 207-221 `CreateCommentRequest` as model pattern), `crates/clickup-api/src/models/task.rs` (lines 8-139 `Task` struct for response type)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code; no `panic!()` in library code
  - [ ] P0: Crate boundary respected — `clickup-api` has no CLI/TUI deps
  - [ ] P1: `cargo test -p clickup-api` passes with new wiremock tests
  - [ ] P1: New endpoint method has `wiremock` test
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` passes
  - [ ] P2: `///` doc comment on `create_task()` and `CreateTaskRequest`

---

### S-2: CLI `task create` command

**As a** CLI user, **I want** to run `clickup task create --list LIST_ID --name "My Task"` with optional flags for description, status, priority, assignee, and due date, **so that** I can create tasks without leaving the terminal.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify | `crates/clickup-cli/src/commands/tasks.rs` | Add `Create` variant to `TaskCommands` enum (after line 38): `Create { #[arg(long)] list: String, #[arg(long)] name: String, #[arg(long)] description: Option<String>, #[arg(long)] status: Option<String>, #[arg(long)] priority: Option<u8>, #[arg(long)] assignee: Vec<i64>, #[arg(long)] due_date: Option<String> }` |
| 2 | Modify | `crates/clickup-cli/src/commands/tasks.rs` | Add match arm in `run()` for `Self::Create { .. }` → call `create_task()` handler function |
| 3 | Create | `crates/clickup-cli/src/commands/tasks.rs` | Implement `create_task()` handler: build `CreateTaskRequest` from args, parse `--due-date` string ("YYYY-MM-DD") to millisecond timestamp, call `client.create_task(list_id, &request)`, format output (table: show created task ID + name + status + URL; json: full task; markdown: rich detail) |
| 4 | Test | `crates/clickup-cli/src/commands/tasks.rs` | Test that due-date parsing converts "2026-04-11" to correct millisecond timestamp. Test output formatting with `insta` snapshot if applicable |

#### Acceptance Criteria

- [ ] GIVEN a valid list ID WHEN the user runs `clickup task create --list LIST_ID --name "Test Task"` THEN a task is created and the output shows the task ID, name, and URL with a green `✓` prefix
- [ ] GIVEN all optional flags provided WHEN the user runs `clickup task create --list L --name "T" --description "D" --status "open" --priority 1 --assignee 123 --due-date "2026-12-31"` THEN the `CreateTaskRequest` includes all fields with correct types
- [ ] GIVEN `--format json` WHEN the task is created THEN the full task JSON is pretty-printed to stdout
- [ ] GIVEN an invalid list ID WHEN the user runs `clickup task create --list BAD --name "T"` THEN a red `✗` error message is displayed on stderr
- [ ] GIVEN `--due-date "2026-04-11"` WHEN building the request THEN the due_date field is the Unix millisecond timestamp for midnight UTC on that date

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (lines 9-38 `TaskCommands` enum pattern), `crates/clickup-cli/src/commands/comments.rs` (lines 68-88 `run()` dispatch pattern), `crates/clickup-cli/src/output.rs` (output formatting utilities)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo fmt --all -- --check` passes
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` passes

---

### S-3: TUI task creation dialog

**As a** TUI user browsing a task list, **I want** to press `n` to open a creation dialog where I fill in a task name and optional fields, **so that** I can create tasks without switching to the CLI or web app.

**Timebox:** ≤3d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify | `crates/clickup-tui/src/app.rs` | Add `CreateTaskDialog` state struct to `App`: `create_dialog_open: bool`, `create_name: String`, `create_description: String`, `create_status: Option<String>`, `create_priority: Option<u8>`, `create_focused_field: usize` (tracks which field has focus). Add `open_create_dialog()` and `close_create_dialog()` methods |
| 2 | Modify | `crates/clickup-tui/src/event.rs` | Add `DataPayload::TaskCreated(Box<Task>)` variant for handling the async creation response |
| 3 | Modify | `crates/clickup-tui/src/data.rs` | Add `pub fn spawn_create_task(client: &ClickUpClient, tx: &mpsc::UnboundedSender<AppEvent>, list_id: &str, request: CreateTaskRequest)` following `spawn_load_task_detail()` pattern (lines 107-128) |
| 4 | Modify | `crates/clickup-tui/src/input.rs` | Handle `n` key on TaskList screen (when no dialog/filter/sidebar is open): call `app.open_create_dialog()`. Add `handle_create_dialog()` function: `Tab`/`Shift+Tab` cycle fields, `Enter` on last field submits, `Esc` cancels, character input goes to focused field |
| 5 | Create | `crates/clickup-tui/src/ui/create_task.rs` | Render function `pub fn render(app: &App, frame: &mut Frame, area: Rect)`: centered modal overlay (60% width, 50% height), bordered with title "Create Task", fields: Name (text input, highlighted when focused), Description (text input), Status (show current value, explain that it uses list default), Priority (1-4 picker or None). Show `[Tab] Next  [Enter] Create  [Esc] Cancel` in bottom bar |
| 6 | Modify | `crates/clickup-tui/src/ui/mod.rs` | Add `pub mod create_task;` export, render the dialog overlay when `app.create_dialog_open` is true (after main screen render, before help overlay) |
| 7 | Modify | `crates/clickup-tui/src/main.rs` | Handle `DataPayload::TaskCreated` event: close dialog, show success message, refresh task list |

#### Acceptance Criteria

- [ ] GIVEN the user is on the TaskList screen WHEN they press `n` THEN a centered modal dialog appears with Name, Description, Status, and Priority fields
- [ ] GIVEN the create dialog is open WHEN the user types a name and presses `Enter` (or Tab to last field + Enter) THEN `spawn_create_task()` is called with the list ID and a `CreateTaskRequest` containing the entered name
- [ ] GIVEN the create dialog is open WHEN the user presses `Esc` THEN the dialog closes without creating a task
- [ ] GIVEN a task is successfully created WHEN the async response arrives THEN the dialog closes, the task list refreshes, and a success message is briefly shown
- [ ] GIVEN the create dialog is open WHEN the user presses `Tab` THEN focus moves to the next field (wrapping from last to first)
- [ ] GIVEN the create dialog is open with no name entered WHEN the user attempts to submit THEN nothing happens (name is required, cannot submit empty)

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/help.rs` (modal overlay rendering pattern), `crates/clickup-tui/src/input.rs` (lines 536-541 `handle_task_detail` as handler pattern), `crates/clickup-tui/src/data.rs` (lines 107-128 `spawn_load_task_detail` as spawn pattern), `crates/clickup-tui/src/app.rs` (lines 148-276 App struct for state additions)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code; no `panic!()` in library code
  - [ ] P0: Crate boundary respected — `clickup-api` has no CLI/TUI deps
  - [ ] P0: Main render thread is never blocked — creation happens via `tokio::spawn`
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` passes

---

### S-4: Integration tests for task creation

**As a** developer, **I want** comprehensive tests for the task creation flow across all three crates, **so that** regressions are caught before they reach users.

**Timebox:** ≤1d | **Risk:** P2 | **Depends on:** S-1, S-2, S-3

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Test | `crates/clickup-api/src/models/task.rs` | Serialization round-trip tests for `CreateTaskRequest`: minimal (name only), full (all fields), verify `skip_serializing_if` behavior |
| 2 | Test | `crates/clickup-api/tests/` or inline | `wiremock` integration test: mock `POST /list/123/task`, verify request body, verify response deserialization |
| 3 | Test | `crates/clickup-cli/src/commands/tasks.rs` | Test due-date string parsing edge cases: valid date, invalid format, leap year |
| 4 | Create | `crates/clickup-api/tests/fixtures/create_task_response.json` | Realistic fixture based on actual ClickUp API response for task creation |

#### Acceptance Criteria

- [ ] GIVEN a `CreateTaskRequest` with only `name` set WHEN serialized THEN no optional fields appear in the JSON
- [ ] GIVEN a `wiremock` mock for `POST /list/123/task` WHEN `create_task("123", &request)` is called THEN the request body contains `"name"` and the response deserializes to `Task`
- [ ] GIVEN the date string "2026-02-29" (invalid — not a leap year) WHEN parsed THEN an appropriate error is returned

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/tests/fixtures/task_detail.json` (existing fixture pattern), `crates/clickup-api/src/models/task.rs` (existing deserialization tests)
- **Gates:**
  - [ ] P1: All new tests pass with `cargo test --workspace`
  - [ ] P1: `wiremock` tests cover success and error paths
  - [ ] P2: Test names follow `test_<what>_<condition>_<expected>` convention

## Execution Sequence

```
S-1 (API model + endpoint)
├── S-2 (CLI command)          [parallel]
└── S-3 (TUI creation dialog)  [parallel]
    └── S-4 (integration tests) [after all]
```

S-2 and S-3 can proceed in parallel once S-1 is complete. S-4 runs last to cover the full integration.

## Assumptions

1. The ClickUp API returns a full `Task` object from `POST /list/{list_id}/task` — Risk if wrong: need a separate `GET /task/{id}` call after creation to get the full response (LOW — documented behavior)
2. Priority values are integers 1-4 matching `TaskPriority` model — Risk if wrong: validation logic needs adjustment (LOW — documented in ClickUp API)
3. Due date as millisecond timestamp (string) is the correct format for the API — Risk if wrong: date field is silently ignored or rejected (MEDIUM — some ClickUp endpoints accept seconds, test will catch this)
4. The TUI `n` key is not already bound on the TaskList screen — Risk if wrong: key conflict (LOW — verified from input.rs analysis, `n` is unbound on TaskList)

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Touches 3 crates, creates new model + endpoint + CLI command + TUI dialog |
| Ambiguity | 2 | API request format is documented but TUI dialog design requires decisions |
| Dependencies | 2 | S-2 and S-3 depend on S-1; TUI dialog is a new UI pattern (no existing form widget) |
| Risk | 1 | All patterns have exemplars in the codebase; API endpoint is well-documented |
