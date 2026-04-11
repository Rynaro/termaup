# F15 — Attachment Handling

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 4 — Navigation & Content
> **Complexity:** 6/12 | **Confidence:** 83%

---

## Problem Statement

termaup currently displays attachments on the TaskDetail screen as a read-only list with type-based emoji icons (`crates/clickup-tui/src/ui/task_detail.rs`, lines 368–394), but users cannot interact with them. There is no way to:

1. **Upload** a new attachment to a task (the `POST /task/{task_id}/attachment` endpoint is not implemented)
2. **Open** an attachment in the default browser or external viewer
3. **Download** an attachment to the local filesystem
4. **List or upload attachments** from the CLI

Power users who manage tasks with specs, screenshots, or deliverables attached need these capabilities to avoid switching to the web app for file operations.

## Approach

1. **API layer:** Add `upload_attachment()` to `ClickUpClient` using `reqwest::multipart::Form` for the `POST /task/{task_id}/attachment` endpoint. The existing `Attachment` model already captures the response shape.

2. **CLI:** Two new subcommands under `clickup task`: `attachments` (list) and `attach` (upload). Listing reuses `get_task()` and reads the `attachments` field. Upload uses the new multipart endpoint.

3. **TUI:** On the TaskDetail screen when the attachment section is visible, add `o` to open the attachment URL in the system browser (`open` on macOS, `xdg-open` on Linux), and `S` to save/download the file to a user-specified or default directory via a streaming `reqwest::get()`.

### Rejected Alternatives

1. **Inline attachment preview in TUI** — Rendering images or PDFs in terminal is unreliable and crate-heavy (sixel, kitty protocol). Opening in the system viewer is more robust. Rejected for complexity and portability.

2. **Custom download manager with progress bar** — A progress bar widget for downloads adds significant complexity. A simple "downloading..." status message is sufficient for v1. Rejected for scope creep.

3. **Upload via TUI dialog** — A file picker in the terminal (e.g., using `rfd` or custom path input) adds significant UX complexity. CLI upload is sufficient; TUI upload is deferred. Rejected for phase scope.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `upload_attachment()` API endpoint (multipart) | Inline attachment preview in TUI | TUI upload dialog |
| CLI `clickup task attachments TASK_ID` | Attachment deletion (no API endpoint exists) | Download progress bar |
| CLI `clickup task attach TASK_ID --file ./path` | Drag-and-drop upload | Batch attachment download |
| TUI `o` to open attachment URL in browser | Image rendering in terminal | |
| TUI `S` to save attachment to disk | | |
| Platform detection for `open`/`xdg-open` | | |

## Stories

### S-1: API Upload Endpoint

**As a** developer using `clickup-api`, **I want** an `upload_attachment()` method on `ClickUpClient`, **so that** CLI and TUI can upload files to tasks.

**Timebox:** ≤2d | **Risk:** Medium — multipart upload is a new pattern for this client | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `post_multipart()` method to `ClickUpClient` | `crates/clickup-api/src/client.rs` | New private/pub(crate) method: builds `reqwest::multipart::Form`, attaches file as `Part::bytes()` with filename, sends POST. Returns `Result<T: DeserializeOwned>`. Must go through rate limiter. |
| 2 | Create `upload_attachment()` endpoint | `crates/clickup-api/src/endpoints/attachments.rs` | `pub async fn upload_attachment(&self, task_id: &str, file_name: &str, file_data: Vec<u8>) -> Result<Attachment>` — wraps `post_multipart()` to `POST /task/{task_id}/attachment` |
| 3 | Register module | `crates/clickup-api/src/endpoints/mod.rs` | Add `pub mod attachments;` |
| 4 | Add `AttachmentResponse` wrapper | `crates/clickup-api/src/models/task.rs` | ClickUp wraps the response: `{ "attachment": { ... } }` — need a wrapper struct for deserialization |
| 5 | Ensure `reqwest` multipart feature | `crates/clickup-api/Cargo.toml` | Verify `features = ["json", "rustls-tls", "multipart"]` on reqwest dependency |
| 6 | Add wiremock tests | `crates/clickup-api/tests/attachment_tests.rs` | Mock `POST /task/{id}/attachment`, verify multipart body, verify response deserialization |

#### Acceptance Criteria

- **GIVEN** a valid task ID and file bytes, **WHEN** `upload_attachment()` is called, **THEN** a multipart POST is sent to `/api/v2/task/{task_id}/attachment` with the file in the `attachment` form field.
- **GIVEN** the API returns 200 with attachment JSON, **WHEN** the response is received, **THEN** an `Attachment` struct is returned with `id`, `title`, `url`, and `extension` populated.
- **GIVEN** the API returns 401, **WHEN** the response is handled, **THEN** `ClickUpError::AuthError` is returned.
- **GIVEN** the file data is empty, **WHEN** `upload_attachment()` is called, **THEN** the request is still sent (ClickUp API will return an error; client does not validate file content).

#### Agent Hints

- **Class:** builder
- **Skill:** `api-endpoints`
- **Context:** `crates/clickup-api/src/client.rs` (HTTP methods, lines 66–180), `crates/clickup-api/src/models/task.rs` (`Attachment` struct line 143), `crates/clickup-api/src/endpoints/mod.rs`
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: wiremock test covers successful upload and error cases
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on `upload_attachment()` and `post_multipart()`

---

### S-2: CLI Attachment Commands

**As a** CLI user, **I want** `clickup task attachments TASK_ID` to list attachments and `clickup task attach TASK_ID --file ./path` to upload a file, **so that** I can manage task attachments from the command line.

**Timebox:** ≤2d | **Risk:** Low — follows existing CLI patterns | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `Attachments` variant to `TaskCommands` | `crates/clickup-cli/src/commands/tasks.rs` | `Attachments { task_id: String, #[arg(long)] format: Option<OutputFormat> }` |
| 2 | Add `Attach` variant to `TaskCommands` | `crates/clickup-cli/src/commands/tasks.rs` | `Attach { task_id: String, #[arg(long)] file: PathBuf }` |
| 3 | Implement `attachments` handler | `crates/clickup-cli/src/commands/tasks.rs` | Call `client.get_task(task_id)`, extract `attachments` field, format as table (columns: Title, Type, URL, Date) or JSON |
| 4 | Implement `attach` handler | `crates/clickup-cli/src/commands/tasks.rs` | Read file bytes with `tokio::fs::read()`, call `client.upload_attachment()`, print success with `✓ Uploaded {filename}` |
| 5 | Handle file-not-found error | `crates/clickup-cli/src/commands/tasks.rs` | Check file exists before reading; print `✗ File not found: {path}` to stderr |
| 6 | Register new subcommands in match arm | `crates/clickup-cli/src/commands/tasks.rs` | Add match arms for `Attachments` and `Attach` in the command handler |

#### Acceptance Criteria

- **GIVEN** a task with 3 attachments, **WHEN** running `clickup task attachments TASK_ID`, **THEN** a table with Title, Type, URL, Date columns is printed to stdout.
- **GIVEN** a task with attachments, **WHEN** running `clickup task attachments TASK_ID --format json`, **THEN** pretty-printed JSON array is output.
- **GIVEN** a valid file path, **WHEN** running `clickup task attach TASK_ID --file ./report.pdf`, **THEN** the file is uploaded and `✓ Uploaded report.pdf` is printed.
- **GIVEN** a non-existent file path, **WHEN** running `clickup task attach TASK_ID --file ./missing.pdf`, **THEN** `✗ File not found: ./missing.pdf` is printed to stderr and exit code is non-zero.

#### Agent Hints

- **Class:** builder
- **Skill:** `cli-data`
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (existing TaskCommands enum lines 8–38), `crates/clickup-cli/src/commands/tasks.rs` (existing handler pattern)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new public items

---

### S-3: TUI Open and Save Attachment Actions

**As a** TUI user viewing a task's attachments, **I want** to press `o` to open an attachment in my default browser and `S` to save it to disk, **so that** I can access attachment content without leaving the terminal workflow.

**Timebox:** ≤2d | **Risk:** Medium — platform-specific browser opening | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `selected_attachment_index: usize` field | `crates/clickup-tui/src/app.rs` | Track which attachment is selected in the attachment section |
| 2 | Add `attachment_section_focused: bool` field | `crates/clickup-tui/src/app.rs` | Track whether the attachment section has focus (vs. subtask/checklist sections) |
| 3 | Add `o` key handler for open | `crates/clickup-tui/src/input.rs` | When on TaskDetail with attachment focused: get `attachment.url`, call `open::that(url)` (or `std::process::Command::new("open")` on macOS / `xdg-open` on Linux) |
| 4 | Add `S` key handler for save | `crates/clickup-tui/src/input.rs` | When on TaskDetail with attachment focused: spawn download task |
| 5 | Create `spawn_download_attachment()` | `crates/clickup-tui/src/data.rs` | Async: `reqwest::get(url).await`, write bytes to `~/Downloads/{filename}` or current dir, send `AppEvent::DataLoaded(DataPayload::AttachmentDownloaded(path))` |
| 6 | Add `DataPayload::AttachmentDownloaded` variant | `crates/clickup-tui/src/app.rs` | Carries the saved file path string |
| 7 | Show flash message on success | `crates/clickup-tui/src/app.rs` | Set `app.error_message` (reuse as info message) to `"✓ Saved to ~/Downloads/{filename}"` with auto-dismiss |
| 8 | Add `open` crate dependency | `crates/clickup-tui/Cargo.toml` | `open = "5"` — cross-platform URL/file opener |
| 9 | Update key hints | `crates/clickup-tui/src/ui/mod.rs` | Add `o Open` and `S Save` hints when attachment section is focused |
| 10 | Highlight selected attachment | `crates/clickup-tui/src/ui/task_detail.rs` | Apply selection style to the currently focused attachment in the list |

#### Acceptance Criteria

- **GIVEN** a task with attachments on TaskDetail, **WHEN** the user navigates to the attachment section and presses `o`, **THEN** the attachment URL is opened in the system default browser.
- **GIVEN** a task with attachments, **WHEN** the user presses `S` on a selected attachment, **THEN** the file is downloaded and saved to `~/Downloads/{filename}`, and a flash message `"✓ Saved to ~/Downloads/{filename}"` appears in the bottom bar.
- **GIVEN** the download fails (network error), **WHEN** the download task completes, **THEN** an error message is displayed in the bottom bar (red `✗`).
- **GIVEN** macOS, **WHEN** `o` is pressed, **THEN** `open {url}` is invoked. **GIVEN** Linux, **WHEN** `o` is pressed, **THEN** `xdg-open {url}` is invoked.

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-tasks`
- **Context:** `crates/clickup-tui/src/ui/task_detail.rs` (attachment rendering lines 368–394), `crates/clickup-tui/src/input.rs` (TaskDetail keys lines 693–710), `crates/clickup-tui/src/data.rs`
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new public items

---

### S-4: Attachment Integration Tests

**As a** maintainer, **I want** comprehensive tests for attachment upload, download, and command behavior, **so that** regressions are caught automatically.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-1, S-2

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create upload fixture | `crates/clickup-api/tests/fixtures/attachment_upload_response.json` | Realistic ClickUp API response for `POST /task/{id}/attachment` |
| 2 | wiremock upload test | `crates/clickup-api/tests/attachment_tests.rs` | Mock multipart POST, verify request has `Content-Type: multipart/form-data`, verify response deserialization |
| 3 | wiremock error test | `crates/clickup-api/tests/attachment_tests.rs` | Mock 401 and 404 responses, verify correct `ClickUpError` variants |
| 4 | CLI attachment list test | `crates/clickup-cli/tests/` | Snapshot test for `attachments` subcommand output format |
| 5 | Deserialization edge cases | `crates/clickup-api/tests/attachment_tests.rs` | Test `Attachment` with missing optional fields (`thumbnail_small`, `extension`) |

#### Acceptance Criteria

- **GIVEN** a wiremock server, **WHEN** `upload_attachment()` is called, **THEN** the mock receives a multipart request with the correct path and file content.
- **GIVEN** a task fixture with attachments, **WHEN** the CLI `attachments` command runs against the mock, **THEN** the snapshot matches the expected table format.
- **GIVEN** an `Attachment` JSON with only `id` and `url`, **WHEN** deserialized, **THEN** optional fields default to `None` without error.

#### Agent Hints

- **Class:** builder
- **Skill:** `docs-release`
- **Context:** `crates/clickup-api/tests/` (existing test patterns), `crates/clickup-api/src/models/task.rs` (`Attachment` struct line 143)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: All new tests pass independently
  - [ ] P2: `cargo clippy` clean

---

## Execution Sequence

```
S-1 (API upload endpoint) ──→ S-2 (CLI commands) ──→ S-4 (tests)
                              S-3 (TUI open/save) ──┘
```

S-3 is independent of S-1 (TUI open/save doesn't need the upload endpoint).
S-2 depends on S-1 for the upload functionality.
S-4 tests should run after S-1 and S-2 are complete.

## Assumptions

1. The ClickUp API accepts multipart form upload with the file in a field named `attachment`. **Risk if wrong:** Field name is documented in ClickUp API v2 docs; low risk.
2. `attachment.url` fields contain direct-access URLs that don't require re-authentication. **Risk if wrong:** If URLs require auth headers, the download function would need to use the `ClickUpClient` instead of raw `reqwest::get()`.
3. The `open` crate handles macOS, Linux, and Windows browser opening correctly. **Risk if wrong:** Fallback to manual `std::process::Command` with platform detection.
4. Most attachments are small enough to download entirely into memory before writing to disk. **Risk if wrong:** For large files, streaming would be needed — deferred as enhancement.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Three crates affected (API, CLI, TUI) |
| Ambiguity | 1 | Clear API docs, well-defined actions |
| Dependencies | 1 | New endpoint, no conflicts with existing code |
| Risk | 2 | Multipart upload is a new HTTP pattern; platform-specific browser opening |

**Total: 6/12** → Standard processing
