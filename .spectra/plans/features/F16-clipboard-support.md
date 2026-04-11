# F16 — Clipboard Support

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 4 — Navigation & Content
> **Complexity:** 3/12 | **Confidence:** 92%

---

## Problem Statement

TUI users frequently need to share a task URL or reference a task ID in other tools (Slack, Git commit messages, documentation). Currently, the task URL is displayed on the TaskDetail screen but there is no way to copy it to the system clipboard without manually selecting terminal text — a clumsy process that breaks the keyboard-driven workflow and doesn't work reliably in all terminal emulators (e.g., tmux, SSH sessions).

## Approach

Add `y` and `Y` keybindings on the TaskDetail screen:

- `y` → copy the current task's ClickUp URL to the system clipboard
- `Y` → copy the current task's ID (e.g., `abc123`) to the clipboard

Use the `arboard` crate for cross-platform clipboard access (macOS `pbcopy`, Linux `xclip`/`xsel`/`wl-clipboard`, Windows native). On clipboard write success, show a brief flash message in the bottom bar (e.g., `"✓ Copied task URL"`) that auto-dismisses after 2 seconds using the existing `error_message` + `error_set_at` infrastructure (repurposed as a general flash message).

### Rejected Alternatives

1. **`cli-clipboard` crate** — Less maintained than `arboard`, no Wayland support. Rejected for ecosystem maturity.

2. **OSC 52 escape sequence** — Terminal-native clipboard via ANSI escape codes. Works over SSH but is unsupported in many terminals (macOS Terminal.app, older iTerm versions). Rejected for inconsistent support. Could be added as a fallback later.

3. **Copy via subprocess (`pbcopy`/`xclip`)** — Manual subprocess management is fragile and requires detecting the platform and installed tools. `arboard` handles this internally. Rejected for maintenance burden.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `y` copies task URL to clipboard | Copy from CLI commands | OSC 52 fallback for SSH sessions |
| `Y` copies task ID to clipboard | Copy arbitrary text selection | Configurable key bindings |
| `arboard` crate dependency | Paste from clipboard | Copy from non-TaskDetail screens |
| Flash message in bottom bar | | Clipboard history |
| Auto-dismiss after 2 seconds | | |
| Cross-platform support (macOS, Linux, Windows) | | |

## Stories

### S-1: Clipboard Integration with arboard

**As a** TUI user viewing a task, **I want** to press `y` to copy the task URL and `Y` to copy the task ID to my system clipboard, **so that** I can quickly paste task references into other tools.

**Timebox:** ≤1d | **Risk:** Low — arboard is well-tested, simple API | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `arboard` dependency | `Cargo.toml` (workspace root) | `arboard = "3"` in `[workspace.dependencies]` |
| 2 | Reference in TUI crate | `crates/clickup-tui/Cargo.toml` | `arboard = { workspace = true }` |
| 3 | Add `y` key handler | `crates/clickup-tui/src/input.rs` | In TaskDetail, comment sidebar closed: `Char('y')` → get `app.current_task.url`, call `arboard::Clipboard::new()?.set_text(url)`, set flash message |
| 4 | Add `Y` key handler | `crates/clickup-tui/src/input.rs` | In TaskDetail, comment sidebar closed: `Char('Y')` → get `app.current_task.id`, call `arboard::Clipboard::new()?.set_text(id)`, set flash message |
| 5 | Handle clipboard errors gracefully | `crates/clickup-tui/src/input.rs` | If `arboard::Clipboard::new()` fails (e.g., no display server), show error message in bottom bar: `"✗ Clipboard unavailable"` |
| 6 | Update key hints | `crates/clickup-tui/src/ui/mod.rs` | Add `y Copy URL` and `Y Copy ID` to TaskDetail key hints (sidebar closed) |

#### Acceptance Criteria

- **GIVEN** the user is on TaskDetail with a task that has `url: "https://app.clickup.com/t/abc123"`, **WHEN** they press `y`, **THEN** the URL is in the system clipboard and the bottom bar shows `"✓ Copied task URL"`.
- **GIVEN** the user is on TaskDetail with a task that has `id: "abc123"`, **WHEN** they press `Y`, **THEN** `"abc123"` is in the system clipboard and the bottom bar shows `"✓ Copied task ID"`.
- **GIVEN** the system has no clipboard support (headless server), **WHEN** `y` is pressed, **THEN** the bottom bar shows `"✗ Clipboard unavailable"` (no panic).
- **GIVEN** `y` is pressed and the flash message is displayed, **WHEN** 2 seconds elapse, **THEN** the flash message disappears and normal key hints are restored.
- **GIVEN** the comment sidebar is open, **WHEN** `y` is pressed, **THEN** nothing happens (key is not intercepted — `y` has no meaning in comment browse mode).

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-polish`
- **Context:** `crates/clickup-tui/src/input.rs` (TaskDetail sidebar-closed keys, lines 693–710), `crates/clickup-tui/src/ui/mod.rs` (key hints builder lines 269–345)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new public items

---

### S-2: Flash Message System

**As a** TUI user, **I want** brief confirmation messages (like "✓ Copied task URL") to appear in the bottom bar and auto-dismiss, **so that** I get feedback on clipboard actions without persistent visual clutter.

**Timebox:** ≤1d | **Risk:** Low — extends existing error message infrastructure | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `flash_message: Option<(String, FlashKind)>` field | `crates/clickup-tui/src/app.rs` | `FlashKind` enum: `Success`, `Info`, `Error`. Replaces the overloaded use of `error_message` for success messages. |
| 2 | Add `flash_set_at: Option<Instant>` field | `crates/clickup-tui/src/app.rs` | Timestamp for auto-dismiss, 2-second timeout |
| 3 | Add `set_flash(&mut self, msg: &str, kind: FlashKind)` method | `crates/clickup-tui/src/app.rs` | Sets both `flash_message` and `flash_set_at` |
| 4 | Add auto-dismiss logic in tick handler | `crates/clickup-tui/src/app.rs` or event loop | Check `flash_set_at`; if elapsed > 2s, clear `flash_message` |
| 5 | Render flash message in bottom bar | `crates/clickup-tui/src/ui/mod.rs` | In `render_bottom_bar()`: if `flash_message` is set, render it instead of key hints. Color by `FlashKind`: green for Success, blue for Info, red for Error. |
| 6 | Migrate existing error display | `crates/clickup-tui/src/ui/mod.rs` | Update error display to use `set_flash(msg, FlashKind::Error)` pattern for consistency |
| 7 | Unit test flash timing | `crates/clickup-tui/src/app.rs` | Test `set_flash()` → message is set → after simulated 2s → message is cleared |

#### Acceptance Criteria

- **GIVEN** `set_flash("✓ Copied task URL", FlashKind::Success)` is called, **WHEN** the next frame renders, **THEN** the bottom bar shows the message in green.
- **GIVEN** a flash message was set, **WHEN** 2 seconds have elapsed, **THEN** the flash message is automatically cleared and key hints are restored.
- **GIVEN** a new flash message is set while one is already displayed, **WHEN** the new message arrives, **THEN** the old message is replaced immediately.
- **GIVEN** `FlashKind::Error`, **WHEN** rendered, **THEN** the message appears with red `✗` prefix (consistent with existing error display).

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-polish`
- **Context:** `crates/clickup-tui/src/app.rs` (error_message field ~line 259, error_set_at ~line 260), `crates/clickup-tui/src/ui/mod.rs` (render_bottom_bar lines 165–177)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on `FlashKind`, `set_flash()`

---

## Execution Sequence

```
S-1 (clipboard keybindings) ─┐
                              ├─→ Integration (S-1 uses S-2's flash messages)
S-2 (flash message system)  ─┘
```

S-1 and S-2 can be developed in parallel. S-1 will call `set_flash()` from S-2 for user feedback. If developed sequentially, S-2 should be implemented first so S-1 can use it.

## Assumptions

1. The `arboard` crate works correctly on macOS (pbcopy), Linux with X11 (xclip/xsel), and Linux with Wayland (wl-clipboard). **Risk if wrong:** Graceful error message; users can fall back to manual selection.
2. The task's `url` field is always populated in the API response. **Risk if wrong:** The `url` field is `Option<String>` in the model; handle `None` by showing `"✗ No URL available"`.
3. A 2-second dismiss timeout is appropriate for flash messages. **Risk if wrong:** Could be made configurable later, but 2s is standard UX practice.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 1 | Single crate (TUI only), 2 small changes |
| Ambiguity | 0 | Crystal-clear requirements |
| Dependencies | 1 | New external crate (`arboard`), no internal deps |
| Risk | 1 | Platform clipboard access can fail; handled gracefully |

**Total: 3/12** → Light processing
