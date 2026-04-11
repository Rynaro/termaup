# F07 — Task Description Editing

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 2 — Task Editing
> **Complexity:** 7/12 | **Confidence:** 80%

---

## Problem Statement

Task descriptions in ClickUp are rich markdown documents that users frequently update. Currently, termaup renders descriptions in read-only mode (markdown-to-terminal in CLI via `termimad`, markdown-to-ratatui spans in TUI via `pulldown-cmark`). Editing a multi-line markdown document inside a TUI text widget is impractical — the UX would be terrible for anything beyond trivial changes. Power users expect the `$EDITOR` pattern (used by `git commit`, `kubectl edit`, `crontab -e`) where the terminal suspends, the user's preferred editor opens, and changes are captured on exit.

## Approach

Implement the `$EDITOR` workflow for both CLI and TUI. The CLI provides `clickup task edit TASK_ID` (opens editor) and `clickup task update TASK_ID --description "inline text"` (non-interactive). The TUI uses `E` on TaskDetail to: (1) exit raw mode and alternate screen, (2) write current `markdown_description` to a temporary file, (3) launch `$EDITOR` (or `$VISUAL`, fallback to `vi`), (4) on editor exit, read the file, (5) send `PUT /task/{id}` with updated `description` + `markdown_description`, (6) re-enter alternate screen and raw mode, (7) refresh the task detail.

The `update_task()` endpoint from F03/F06 is reused — `UpdateTaskRequest` already includes optional `description` and `markdown_description` fields (or will be extended in S-1).

### Rejected Alternatives

1. **Inline TUI text editor** — Build a multi-line text editor widget inside the TUI. Extremely complex (undo/redo, line wrapping, scrolling, selection), poor UX compared to real editors like vim/nano, and contradicts terminal power user expectations. Rejected for complexity and poor UX.

2. **Clipboard-based workflow** — Copy description to clipboard, let user paste into external editor, then paste back. Fragile (clipboard access varies by OS/terminal), multi-step manual process, no way to detect when user is "done." Rejected for poor UX.

3. **Web browser redirect** — Open the ClickUp web task URL in the browser for editing. Defeats the purpose of a terminal client. Rejected entirely.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `$EDITOR` / `$VISUAL` integration for description editing | Rich text formatting helpers | Checklist editing via `$EDITOR` |
| CLI `clickup task edit TASK_ID` (interactive) | Image/attachment embedding in description | Comment editing via `$EDITOR` |
| CLI `clickup task update TASK_ID --description "inline"` (non-interactive) | Conflict detection (concurrent edits) | Description templates |
| TUI terminal suspend/restore for editor launch | Description preview before save | Collaborative editing |
| `UpdateTaskRequest` extension for `description` + `markdown_description` | | |
| Editor fallback chain: `$VISUAL` → `$EDITOR` → `vi` | | |
| Temp file cleanup on all code paths (success, error, signal) | | |

## Stories

### S-1: Editor Integration Utility

**As a** developer in the workspace, **I want** a reusable editor integration module that launches `$EDITOR` with file content and returns the edited result, **so that** both CLI and TUI can use the same editor workflow.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/src/editor.rs` | Create `pub async fn edit_in_editor(initial_content: &str, file_extension: &str) -> Result<String>`: (a) resolve editor from `$VISUAL` → `$EDITOR` → `vi`, (b) write `initial_content` to a temporary file in the config dir (`~/.config/clickup-rs/.edit-buffer.md`), (c) spawn editor process via `tokio::process::Command` with `.status().await`, (d) read file contents after editor exits, (e) return edited content, (f) clean up file. |
| 2 | Create | `crates/clickup-api/src/editor.rs` | Add `pub fn resolve_editor() -> Result<String>`: check `$VISUAL`, then `$EDITOR`, then hardcoded `"vi"`. Return error if none found and `vi` is not available. |
| 3 | Modify | `crates/clickup-api/src/lib.rs` | Add `pub mod editor;` declaration. |
| 4 | Extend | `crates/clickup-api/src/models/task.rs` | Ensure `UpdateTaskRequest` has `description: Option<String>` and `markdown_description: Option<String>` fields with `#[serde(skip_serializing_if = "Option::is_none")]`. |
| 5 | Test | `crates/clickup-api/src/editor.rs` | Unit test for `resolve_editor()`: mock env vars, verify fallback chain. |
| 6 | Test | `crates/clickup-api/src/editor.rs` | Integration test: set `$EDITOR` to `cat` (no-op), call `edit_in_editor("hello", "md")`, verify returns "hello" unchanged. |

#### Acceptance Criteria

- [ ] GIVEN `$VISUAL` is set to `"code --wait"`, WHEN `resolve_editor()` is called, THEN it returns `"code --wait"`.
- [ ] GIVEN `$VISUAL` is unset and `$EDITOR` is `"vim"`, WHEN `resolve_editor()` is called, THEN it returns `"vim"`.
- [ ] GIVEN both `$VISUAL` and `$EDITOR` are unset, WHEN `resolve_editor()` is called, THEN it returns `"vi"`.
- [ ] GIVEN content "# Hello\n\nWorld", WHEN `edit_in_editor()` is called and the editor makes no changes, THEN the returned content equals the original.
- [ ] GIVEN the editor exits with a non-zero status code, WHEN `edit_in_editor()` returns, THEN an error is returned indicating the editor failed.
- [ ] GIVEN `edit_in_editor()` completes (success or error), WHEN checked, THEN the temporary file has been deleted.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/config.rs` (config dir path resolution), `crates/clickup-api/src/models/task.rs` (UpdateTaskRequest)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected — editor module in clickup-api is acceptable since both binaries need it
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Env var fallback tests
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean
  - [ ] P2: `///` doc comments on `edit_in_editor()` and `resolve_editor()`

---

### S-2: TUI Terminal Suspend/Restore for Editor

**As a** TUI user viewing a task, **I want** to press `E` to open my preferred editor with the task description, edit it, and return to the TUI with changes saved, **so that** I can use familiar editing tools for complex description changes.

**Timebox:** ≤2d | **Risk:** P0 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-tui/src/terminal.rs` | Add `pub fn suspend_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()>`: (a) `terminal.clear()`, (b) `crossterm::terminal::disable_raw_mode()`, (c) `crossterm::execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)`. |
| 2 | Create | `crates/clickup-tui/src/terminal.rs` | Add `pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()>`: (a) `crossterm::terminal::enable_raw_mode()`, (b) `crossterm::execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)`, (c) `terminal.clear()`. |
| 3 | Extend | `crates/clickup-tui/src/input.rs` | In `handle_task_detail()`, handle `E` key (shift+e): extract `markdown_description` from current task → call `suspend_terminal()` → call `edit_in_editor()` blocking (use `tokio::task::spawn_blocking` wrapping the synchronous editor wait) → call `restore_terminal()` → if content changed, spawn `spawn_update_task()` with `description` and `markdown_description` set. |
| 4 | Extend | `crates/clickup-tui/src/event.rs` | Add `AppEvent::EditorComplete { task_id: String, description: String }` variant if needed, or handle the editor flow synchronously within the key handler by blocking the event loop during editor execution. |
| 5 | Modify | `crates/clickup-tui/src/main.rs` | Ensure the terminal handle is accessible from the input handler (may need to pass `&mut Terminal` or use a shared reference). Verify panic hook still restores terminal after editor suspend/restore cycle. |

#### Acceptance Criteria

- [ ] GIVEN the user is on TaskDetail with a task that has a markdown description, WHEN they press `E`, THEN the TUI exits alternate screen and raw mode, and `$EDITOR` opens with the description content.
- [ ] GIVEN the editor is open, WHEN the user saves and exits, THEN the TUI re-enters alternate screen and raw mode and renders correctly.
- [ ] GIVEN the user modified the description in the editor, WHEN the editor exits, THEN a `PUT /task/{id}` is sent with the updated `markdown_description` field.
- [ ] GIVEN the user did NOT modify the description (content unchanged), WHEN the editor exits, THEN NO API call is made (skip unnecessary update).
- [ ] GIVEN the editor exits with an error (non-zero status), WHEN control returns to the TUI, THEN the terminal is properly restored AND an error message is shown.
- [ ] GIVEN a task with no description (`markdown_description` is `None`), WHEN the user presses `E`, THEN the editor opens with an empty file.
- [ ] GIVEN the TUI is suspended for the editor, WHEN a panic occurs during restore, THEN the panic hook still restores the terminal to a usable state.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/main.rs` (terminal setup/teardown, panic hook), `crates/clickup-tui/src/input.rs` (key handling in TaskDetail), `crates/clickup-tui/src/data.rs` (spawn_update_task from F06)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: Terminal is ALWAYS restored — even on editor crash or panic
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` clean
  - [ ] P2: `///` doc comments on suspend/restore functions

---

### S-3: CLI Edit Command

**As a** CLI user, **I want** to run `clickup task edit TASK_ID` to open my editor with the task description, or `clickup task update TASK_ID --description "text"` for non-interactive updates, **so that** I can modify descriptions from the command line.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-cli/src/commands/tasks.rs` | Add `Edit` variant to `TaskCommands` enum: `Edit { task_id: String }`. |
| 2 | Create | `crates/clickup-cli/src/commands/tasks.rs` | Add `edit_task()` handler: fetch task via `client.get_task()` → extract `markdown_description` (or empty string) → call `edit_in_editor()` → if changed, call `client.update_task()` with `description` + `markdown_description` → print `✓ Description updated for task {id}`. |
| 3 | Extend | `crates/clickup-cli/src/commands/tasks.rs` | Add `--description` flag to the existing `Update` variant (from F06 S-2): `description: Option<String>`. When provided, directly update without opening editor. |
| 4 | Modify | `crates/clickup-cli/src/main.rs` | Wire `TaskCommands::Edit` in the command dispatch match arm. |
| 5 | Test | `crates/clickup-cli/src/commands/tasks.rs` | Unit test: verify `edit_task()` skips API call when content is unchanged. |

#### Acceptance Criteria

- [ ] GIVEN the command `clickup task edit abc123`, WHEN executed, THEN the CLI fetches the task, opens `$EDITOR` with the description, and on editor exit sends the updated description via `PUT /task/abc123`.
- [ ] GIVEN `clickup task edit abc123` and the user makes no changes, WHEN the editor exits, THEN the CLI prints `ℹ No changes detected` and makes no API call.
- [ ] GIVEN `clickup task update abc123 --description "New description"`, WHEN executed, THEN the description is updated directly without opening an editor.
- [ ] GIVEN a task with no description, WHEN `clickup task edit` is executed, THEN the editor opens with an empty file and saves work correctly.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (TaskCommands enum, existing get_task handler), `crates/clickup-api/src/editor.rs` (editor utility from S-1)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

## Execution Sequence

```
S-1 (editor utility) ──┬──→ S-2 (TUI suspend/restore)
                        └──→ S-3 (CLI edit command)
```

- **Phase 1:** S-1 — Editor integration utility (shared by both binaries)
- **Phase 2 (parallel):** S-2 (TUI terminal management) + S-3 (CLI edit command)

## Assumptions

1. The `$EDITOR` invocation is synchronous — the TUI event loop blocks while the editor is open. This is acceptable because the terminal is fully yielded to the editor process. **Risk if wrong:** If async editor spawning is needed, the architecture changes significantly.
2. ClickUp API accepts both `description` (plain text) and `markdown_description` (markdown) in the PUT body. Setting `markdown_description` may auto-populate `description` on the server side. **Risk if wrong:** May need to strip markdown for the `description` field or only send `markdown_description`.
3. The temporary file is written to `~/.config/clickup-rs/` (not system temp) to avoid permission issues and ensure the file persists if the editor crashes. **Risk if wrong:** Minimal — fallback to current directory.
4. Terminal restore after `$EDITOR` will work correctly on all supported terminals (iTerm2, Terminal.app, Alacritty, kitty, Windows Terminal). **Risk if wrong:** May need terminal-specific workarounds for edge cases.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | 3 crates affected but relatively small surface area per crate |
| Ambiguity | 2 | Terminal suspend/restore is well-understood but edge cases exist (panic during editor, signal handling) |
| Dependencies | 2 | Depends on F06's `update_task()` endpoint and `UpdateTaskRequest`; editor utility is new code |
| Risk | 1 | `$EDITOR` pattern is proven (git, kubectl); terminal restore is the main risk point but mitigated by panic hook |

**Total: 7/12** → Standard processing with careful attention to terminal management
