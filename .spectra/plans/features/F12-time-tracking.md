# F12 — Time Tracking

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 3 — Task Enrichment
> **Complexity:** 6/12 | **Confidence:** 83%

---

## Problem Statement

Time tracking is essential for teams that bill by the hour or measure velocity. ClickUp's web app provides rich time tracking — logging time entries with duration, description, start/end times, and billable flags. termaup currently displays `time_estimate` and `time_spent` fields on tasks (as raw millisecond values) but cannot list time entries, log new time, or delete entries. Additionally, there is no duration parser to convert human-friendly strings like "2h30m" into milliseconds, which is needed by both this feature and F06 (time estimate editing).

## Approach

Add three API endpoints for time entry CRUD, create a reusable duration parser utility in `clickup-api`, build CLI commands for listing and logging time, and add a time section to the TUI TaskDetail screen. The duration parser is the linchpin — it's extracted as a shared utility since F06 also needs it for the `--estimate` flag.

### Rejected Alternatives

1. **Live timer (start/stop)** — Implement a running timer in the TUI that tracks wall-clock time and submits on stop. Complex state management (timer persists across screen changes, needs to survive app crashes), and the ClickUp API doesn't natively support "running timers" via the v2 API. Rejected for complexity; manual time entry is MVP.

2. **Calendar-based time input** — Let users pick start/end times from a calendar widget. Overly complex for the common case (logging a duration). Most users log "I spent 2 hours" not "I worked from 10:00 to 12:00". Rejected; duration-first input with optional start time is simpler.

3. **Duration parser in each binary crate** — Duplicate the parser in `clickup-cli` and `clickup-tui`. Violates DRY and risks inconsistency. Rejected; shared parser in `clickup-api` serves both.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `GET /task/{id}/time` endpoint | Running timers (start/stop) | Time entry editing |
| `POST /task/{id}/time` endpoint | Billable time tracking UI | Time reports and summaries |
| `DELETE /task/{id}/time/{interval_id}` endpoint | Time entry approval workflows | Weekly timesheet view |
| Duration parser ("2h30m", "1.5h", "90m", "1d") | Calendar-based time input | Time entry tags |
| `TimeEntry` model | | |
| CLI `time list` and `time log` commands | | |
| TUI time section display + log dialog | | |

## Stories

### S-1: API Endpoints and Models

**As a** developer using clickup-api, **I want** methods to list, create, and delete time entries for a task, **so that** CLI and TUI can manage time tracking.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/src/models/time_entry.rs` | Add `TimeEntry` struct (Debug, Clone, Serialize, Deserialize): `id: String` (with `deserialize_string_or_number`), `task: Option<TimeEntryTask>` (`#[serde(default)]`), `wid: Option<String>`, `user: Option<User>`, `duration: i64` (ms, with `deserialize_string_or_number` as ClickUp may send string), `description: Option<String>`, `start: Option<String>` (Unix ms timestamp), `end: Option<String>` (Unix ms timestamp), `billable: Option<bool>`, `tags: Vec<Tag>` (with `#[serde(default, deserialize_with = "deserialize_null_as_default")]`). |
| 2 | Create | `crates/clickup-api/src/models/time_entry.rs` | Add `TimeEntryTask` struct: `id: String`, `name: Option<String>`, `status: Option<TaskStatus>`. Minimal reference to avoid circular deps. |
| 3 | Create | `crates/clickup-api/src/models/time_entry.rs` | Add `TimeEntriesResponse` wrapper: `data: Vec<TimeEntry>` with `#[serde(default, deserialize_with = "deserialize_null_as_default")]`. |
| 4 | Create | `crates/clickup-api/src/models/time_entry.rs` | Add `CreateTimeEntryRequest` struct: `duration: i64` (ms), `description: Option<String>`, `start: Option<i64>` (Unix ms), `end: Option<i64>` (Unix ms), `billable: Option<bool>`. Use `#[serde(skip_serializing_if = "Option::is_none")]` on optional fields. |
| 5 | Modify | `crates/clickup-api/src/models/mod.rs` | Add `pub mod time_entry;` and re-export key types. |
| 6 | Create | `crates/clickup-api/src/endpoints/time_tracking.rs` | Add `get_time_entries(&self, task_id: &str) -> Result<Vec<TimeEntry>>`: `GET /task/{task_id}/time`, deserialize `TimeEntriesResponse`. |
| 7 | Extend | `crates/clickup-api/src/endpoints/time_tracking.rs` | Add `create_time_entry(&self, task_id: &str, req: &CreateTimeEntryRequest) -> Result<TimeEntry>`: `POST /task/{task_id}/time`. |
| 8 | Extend | `crates/clickup-api/src/endpoints/time_tracking.rs` | Add `delete_time_entry(&self, task_id: &str, interval_id: &str) -> Result<()>`: `DELETE /task/{task_id}/time/{interval_id}`. |
| 9 | Modify | `crates/clickup-api/src/endpoints/mod.rs` | Add `pub mod time_tracking;` declaration. |
| 10 | Test | `crates/clickup-api/tests/fixtures/time_entries.json` | Create fixture: `{"data":[...]}` with 2-3 time entries of varying completeness (one with all fields, one minimal). |
| 11 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_get_time_entries` — verify deserialization with full and minimal entries. |
| 12 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_create_time_entry` — verify POST body contains duration and optional fields. |

#### Acceptance Criteria

- [ ] GIVEN a task with time entries, WHEN `client.get_time_entries("task1")` is called, THEN a `Vec<TimeEntry>` is returned with id, duration, description, and user.
- [ ] GIVEN a `CreateTimeEntryRequest` with `duration: 9000000` and `description: "Code review"`, WHEN serialized, THEN the body is `{"duration":9000000,"description":"Code review"}`.
- [ ] GIVEN a `TimeEntry` response where `duration` is a string `"9000000"`, WHEN deserialized, THEN `duration` is parsed correctly (string-or-number deserializer).
- [ ] GIVEN a `TimeEntry` response with `tags: null`, WHEN deserialized, THEN `tags` is an empty `Vec`.
- [ ] GIVEN `client.delete_time_entry("task1", "interval1")`, WHEN called, THEN `DELETE /task/task1/time/interval1` is sent.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/models/comment.rs` (model with user reference), `crates/clickup-api/src/endpoints/comments.rs` (CRUD endpoint exemplar), `crates/clickup-api/src/models/task.rs` (defensive serde patterns, deserialize_string_or_number)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Wiremock tests for all 3 endpoints
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean
  - [ ] P2: `///` doc comments on all public types and methods

---

### S-2: Duration Parser Utility

**As a** developer in the workspace, **I want** a shared `parse_duration()` function that converts human-friendly strings like "2h30m", "1.5h", "90m", "1d" to milliseconds, **so that** both CLI (F06 + F12) and TUI time inputs can use consistent parsing.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/src/duration.rs` | Add `pub fn parse_duration(input: &str) -> Result<u64>`: parse patterns: `Nd` (days, ×86400000), `Nh` (hours, ×3600000), `Nm` (minutes, ×60000), combinations like `NdNhNm`, `N.Nh` (fractional hours). Return milliseconds. |
| 2 | Extend | `crates/clickup-api/src/duration.rs` | Add `pub fn format_duration(ms: u64) -> String`: convert milliseconds to human-readable "2h 30m" format. Used for display in CLI output and TUI rendering. |
| 3 | Modify | `crates/clickup-api/src/lib.rs` | Add `pub mod duration;` declaration. |
| 4 | Test | `crates/clickup-api/src/duration.rs` | Unit tests: `"2h30m"`→9000000, `"1.5h"`→5400000, `"90m"`→5400000, `"1d"`→86400000, `"1d2h30m"`→95400000, `"0m"`→0, `"45m"`→2700000, `""`→error, `"abc"`→error, `"5"`→error (no unit), `"h"`→error (no number). |
| 5 | Test | `crates/clickup-api/src/duration.rs` | Unit tests for `format_duration`: 9000000→"2h 30m", 86400000→"1d", 5400000→"1h 30m", 60000→"1m", 0→"0m". |

#### Acceptance Criteria

- [ ] GIVEN input "2h30m", WHEN `parse_duration()` is called, THEN it returns `Ok(9000000)`.
- [ ] GIVEN input "1.5h", WHEN `parse_duration()` is called, THEN it returns `Ok(5400000)`.
- [ ] GIVEN input "1d2h30m", WHEN `parse_duration()` is called, THEN it returns `Ok(95400000)`.
- [ ] GIVEN input "90m", WHEN `parse_duration()` is called, THEN it returns `Ok(5400000)`.
- [ ] GIVEN input "1d", WHEN `parse_duration()` is called, THEN it returns `Ok(86400000)`.
- [ ] GIVEN empty string, WHEN `parse_duration()` is called, THEN it returns an `Err`.
- [ ] GIVEN input "5" (no unit), WHEN `parse_duration()` is called, THEN it returns an `Err`.
- [ ] GIVEN 9000000 ms, WHEN `format_duration()` is called, THEN it returns "2h 30m".
- [ ] GIVEN 86400000 ms, WHEN `format_duration()` is called, THEN it returns "1d".

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/lib.rs` (module structure), `crates/clickup-api/src/error.rs` (ClickUpError for parse errors)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Comprehensive parse/format unit tests
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean
  - [ ] P2: `///` doc comments on both public functions

---

### S-3: CLI Time Commands

**As a** CLI user, **I want** `clickup time list --task TASK_ID` to see logged time and `clickup time log --task TASK_ID --duration "2h30m"` to log time, **so that** I can manage time tracking from the command line.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1, S-2

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-cli/src/commands/time.rs` | Add `TimeCommands` enum: `List { #[arg(long)] task: String }`, `Log { #[arg(long)] task: String, #[arg(long)] duration: String, #[arg(long)] description: Option<String> }`. |
| 2 | Create | `crates/clickup-cli/src/commands/time.rs` | Implement `list_time()`: call `client.get_time_entries()` → display table with columns: ID, Duration (formatted via `format_duration()`), Description, User, Date. Support `--format`. |
| 3 | Create | `crates/clickup-cli/src/commands/time.rs` | Implement `log_time()`: parse duration string via `parse_duration()` → build `CreateTimeEntryRequest` with `start` defaulting to now (Unix ms) → call `client.create_time_entry()` → print `✓ Logged {formatted_duration} to task {id}`. |
| 4 | Modify | `crates/clickup-cli/src/commands/mod.rs` | Add `pub mod time;` declaration. |
| 5 | Modify | `crates/clickup-cli/src/main.rs` | Add `Time(TimeCommands)` to root `Commands` enum and wire dispatch. |

#### Acceptance Criteria

- [ ] GIVEN `clickup time list --task abc123`, WHEN executed, THEN a table is printed showing time entries with human-readable durations.
- [ ] GIVEN `clickup time list --task abc123 --format json`, WHEN executed, THEN raw JSON time entries are printed.
- [ ] GIVEN `clickup time log --task abc123 --duration "2h30m"`, WHEN executed, THEN `POST /task/abc123/time` is sent with `{"duration":9000000,"start":<now_ms>}` and `✓ Logged 2h 30m to task abc123` is printed.
- [ ] GIVEN `clickup time log --task abc123 --duration "2h30m" --description "Code review"`, WHEN executed, THEN the description is included in the POST body.
- [ ] GIVEN an invalid duration "abc", WHEN executed, THEN an error is printed: `✗ Invalid duration format: ...`.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (command group exemplar), `crates/clickup-api/src/duration.rs` (parse/format functions from S-2), `crates/clickup-cli/src/main.rs` (dispatch)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-4: TUI Time Display and Log Dialog

**As a** TUI user on the TaskDetail screen, **I want** to see logged time entries in a dedicated section and press `T` to open a time log dialog, **so that** I can view and log time without leaving the TUI.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1, S-2

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-tui/src/app.rs` | Add state: `time_entries: Vec<TimeEntry>` (loaded when task detail opens), `time_log_open: bool`, `time_log_duration: String`, `time_log_description: String`, `time_log_field: usize` (0=duration, 1=description). |
| 2 | Extend | `crates/clickup-tui/src/data.rs` | Add `spawn_load_time_entries()`: calls `client.get_time_entries()` → sends `DataPayload::TimeEntries(Vec<TimeEntry>)`. Call alongside `spawn_load_task_detail()`. |
| 3 | Extend | `crates/clickup-tui/src/data.rs` | Add `spawn_create_time_entry()`: calls `client.create_time_entry()` → sends `DataPayload::TimeEntryCreated(Box<TimeEntry>)`. |
| 4 | Extend | `crates/clickup-tui/src/event.rs` | Add `DataPayload::TimeEntries(Vec<TimeEntry>)` and `DataPayload::TimeEntryCreated(Box<TimeEntry>)` variants. Handle: store in app state, append to list. |
| 5 | Modify | `crates/clickup-tui/src/ui/task_detail.rs` | Add time entries section after the existing metadata: header "⏱ Time Tracked", total time (sum of durations, formatted), then list of entries showing duration, description, user, and date. Use `format_duration()` for display. |
| 6 | Create | `crates/clickup-tui/src/ui/time_dialog.rs` | Render floating dialog: two fields — Duration (text input with "2h30m" format hint) and Description (optional text input). Tab switches fields. Enter submits (parses duration → creates entry). Esc cancels. |
| 7 | Modify | `crates/clickup-tui/src/ui/mod.rs` | Add `pub mod time_dialog;` declaration. |
| 8 | Extend | `crates/clickup-tui/src/input.rs` | In `handle_task_detail()`: `T` key (shift+t) → open time log dialog. When dialog is open: route keys to dialog handler (Tab switch fields, character input, Enter submit, Esc cancel). |

#### Acceptance Criteria

- [ ] GIVEN a task with time entries, WHEN TaskDetail is opened, THEN a "Time Tracked" section shows the total time and individual entries.
- [ ] GIVEN time entries loaded, WHEN rendered, THEN each entry shows duration in "2h 30m" format, description, user name, and date.
- [ ] GIVEN the user presses `T` on TaskDetail, WHEN the dialog opens, THEN it shows two input fields: Duration and Description.
- [ ] GIVEN the user types "2h30m" in duration and "Code review" in description and presses Enter, WHEN submitted, THEN a time entry is created via API with correct millisecond duration.
- [ ] GIVEN an invalid duration (e.g., "abc"), WHEN Enter is pressed, THEN a validation error is shown in the dialog (no API call).
- [ ] GIVEN the dialog is open, WHEN Esc is pressed, THEN the dialog closes with no changes.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/task_detail.rs` (section rendering), `crates/clickup-tui/src/ui/comment_sidebar.rs` (floating dialog/input exemplar), `crates/clickup-tui/src/data.rs` (spawn patterns), `crates/clickup-api/src/duration.rs` (parse/format)
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
S-1 (API) ──────┐
                 ├──→ S-3 (CLI) 
S-2 (parser) ───┤
                 └──→ S-4 (TUI)
```

- **Phase 1 (parallel):** S-1 (API endpoints) + S-2 (duration parser) — both are independent
- **Phase 2 (parallel):** S-3 (CLI) + S-4 (TUI) — both depend on S-1 and S-2

## Assumptions

1. The ClickUp `GET /task/{id}/time` endpoint returns `{"data": [...]}` wrapper format. **Risk if wrong:** May use a different wrapper key — adjust `TimeEntriesResponse`.
2. The `POST /task/{id}/time` endpoint requires `duration` in milliseconds as an integer, not a string. **Risk if wrong:** Adjust to string — low impact.
3. The `duration` field in `TimeEntry` response may be a string or number (ClickUp API quirk). **Risk if wrong:** Already mitigated with `deserialize_string_or_number`.
4. The `start` field defaults to "now" if not provided by the API client. **Risk if wrong:** May need to always send explicit start time — use `chrono::Utc::now().timestamp_millis()`.
5. F06 S-2 currently has its own `parse_duration()` in the CLI crate. After S-2 ships, F06 should migrate to use the shared utility. **Risk if wrong:** Temporary duplication until migration.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | 3 crates + shared utility; 3 endpoints, CLI commands, TUI section + dialog |
| Ambiguity | 1 | Time entry format is straightforward; duration parser is well-defined |
| Dependencies | 2 | Duration parser is shared; time entries need task detail integration |
| Risk | 1 | Additive changes; time entry model follows established defensive serde patterns |

**Total: 6/12** → Standard processing
