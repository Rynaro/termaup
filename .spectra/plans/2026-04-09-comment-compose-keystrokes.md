# SPEC: Comment Compose Keystroke Rework

> SPECTRA v4.2.0 — SPEC-2026-04-09-001
> Confidence: 92% — AUTO_PROCEED

---

## 🎯 SCOPE ANALYSIS

**Intent Type:** CHANGE
**Complexity Score:** 5/12 (Scope 1 + Ambiguity 1 + Dependencies 1 + Risk 2)
**Thinking Budget:** Standard

**WHO:** TUI user composing/editing/replying to comments
**WHAT:** Remap comment compose keystrokes — Enter submits, Alt+Enter inserts newline, fix `?` and `q` being swallowed by global handlers during compose
**WHY:** Current Ctrl+D submit is unreliable and unintuitive; `?` (and `q`) are intercepted by global key handlers before reaching the compose box, making those characters untypeable in comments
**CONSTRAINTS:** crossterm 0.28, no Kitty keyboard protocol, Alt+Enter chosen for universal terminal compatibility

**Boundaries:**

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| Enter = submit comment | Kitty keyboard protocol enablement | Ctrl+C behavior in compose mode (matches existing filter-mode pattern) |
| Alt+Enter = newline in compose | Shift+Enter support (terminal limitation) | Multi-cursor / rich text editing |
| Fix `?` swallowed in compose | Reworking global key dispatch architecture | |
| Fix `q` swallowed in compose | Delete-confirm modal key handling | |
| Update UI hint text | | |
| Update help overlay | | |
| Update bottom bar key hints | | |

**Assumptions:**
1. Alt+Enter is reliably detected by crossterm on all target terminals — Risk if wrong: users can't insert newlines (LOW — Alt+Enter is universally supported in raw mode)
2. The mention picker's Enter/Tab handling (confirm selection) takes priority over submit — Risk if wrong: mention selection broken (LOW — picker already returns before normal compose handling)
3. Users rarely type `q` and `?` in comments so the existing bugs are low-severity — Risk if wrong: N/A, we're fixing them regardless

---

## 📚 PATTERN ANALYSIS

**Query:** "text input mode bypassing global keys"
**Matches:** 1 direct pattern

| ID | Pattern | Similarity | Decision |
|----|---------|------------|----------|
| P1 | `filter_active` early-return bypass (input.rs:28-31) | 95% | USE_TEMPLATE |
| P2 | `filter_panel_open` early-return bypass (input.rs:22-26) | 90% | CONTEXT_ONLY |

**Strategy:** USE_TEMPLATE — the `filter_active` pattern is an exact match: check a "text input is active" condition before global keys, route to the appropriate handler, and return.

---

## 🌳 EXPLORATION SUMMARY

**Hypotheses:** 3 generated, top 2 expanded

| # | Name | Feas | Value | Risk | Pattern | Timebox | Total |
|---|------|------|-------|------|---------|---------|-------|
| 1 | Early-return compose bypass (like filter_active) | 3 | 3 | 3 | 3 | 3 | 15 |
| 2 | Conditional global keys (check "text input active" flag) | 3 | 3 | 2 | 2 | 2 | 12 |
| 3 | Refactor into input mode state machine | 2 | 3 | 1 | 1 | 1 | 8 |

**Selected:** H1 — Early-return compose bypass
**Rationale:** Follows the exact pattern already proven in the codebase (filter_active). Minimal diff. No new abstractions. Fixes both `q` and `?` automatically.
**Rejected:**
- H2: Adds a new `text_input_active` flag to App that must be kept in sync — more surface area for bugs
- H3: Over-engineered for a 3-key fix; refactoring the entire input dispatch is valuable but out of scope

---

## Feature: Comment Compose Keystroke Rework

### 📋 STORY: S-1 — Bypass global keys during comment compose mode

> 🔴 P0 — Fixes characters being swallowed during compose

**Description:** As a TUI user composing a comment, I want `q`, `?`, and all other character keys to be typed into the compose box instead of triggering global shortcuts, so that I can write any text in my comments.

**Timebox:** ≤1d
**Risk:** P0

#### Action Plan:
1. **Modify:** `handle_key()` in `input.rs` — add early-return check after `filter_active` (line ~32): if `app.screen == Screen::TaskDetail` and `app.comment_input_mode != CommentInputMode::Browse`, route directly to `handle_task_detail()` and return. Keep `Ctrl+C` as a universal quit ABOVE this check.
2. **Test:** Verify that typing `q`, `?`, `/`, and all printable characters in compose mode inserts them into the comment text.
3. **Test:** Verify that `Esc` in compose mode still cancels (handled within compose handler).
4. **Test:** Verify that `Ctrl+C` still quits the app even during compose mode.

#### Acceptance Criteria:
- [ ] GIVEN the comment sidebar is open AND the user is in compose mode (new/edit/reply) WHEN they type `q` THEN `q` is appended to the comment text AND the app does NOT quit
- [ ] GIVEN the comment sidebar is open AND the user is in compose mode WHEN they type `?` THEN `?` is appended to the comment text AND the help overlay does NOT appear
- [ ] GIVEN the comment sidebar is open AND the user is in compose mode WHEN they press `Ctrl+C` THEN the app quits (universal escape hatch preserved)

#### Technical Context:
- **Pattern:** `filter_active` early-return (input.rs:28-31)
- **Files:** `crates/clickup-tui/src/input.rs`
- **Dependencies:** None

#### Agent Hints:
- **Class:** builder
- **Context:** `crates/clickup-tui/src/input.rs` (lines 22-48, 528-544)
- **Gates:**
  - [ ] P0: `cargo build --workspace` succeeds
  - [ ] P0: Typing `q` and `?` in compose mode does NOT trigger global shortcuts
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` passes

---

### 📋 STORY: S-2 — Remap Enter to submit, Alt+Enter to newline

> 🔴 P0 — Core keystroke change

**Description:** As a TUI user composing a comment, I want Enter to submit my comment and Alt+Enter to insert a newline, so that the submit action matches common chat application conventions and works reliably across all terminals.

**Timebox:** ≤1d
**Risk:** P0

#### Action Plan:
1. **Modify:** `handle_comment_compose()` in `input.rs` — replace the `KeyCode::Char('d') if CONTROL` submit block (lines 820-896) with `KeyCode::Enter` (no modifiers / without ALT modifier) using the same submit logic.
2. **Modify:** `handle_comment_compose()` in `input.rs` — replace the `KeyCode::Enter` newline block (line 897-899) with `KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT)` for newline insertion.
3. **Verify:** Mention picker `Enter` handling (line 771) runs before normal compose handling due to the early-return guard — no change needed there.
4. **Test:** Verify Enter submits a new comment, reply, and edit.
5. **Test:** Verify Alt+Enter inserts a newline.
6. **Test:** Verify Enter with empty text does not submit (existing guard at line 823).

#### Acceptance Criteria:
- [ ] GIVEN the user is composing a comment with non-empty text WHEN they press Enter THEN the comment is submitted (new/reply/edit dispatched correctly)
- [ ] GIVEN the user is composing a comment WHEN they press Alt+Enter THEN a newline character is inserted into the compose text
- [ ] GIVEN the user is composing a comment with empty/whitespace-only text WHEN they press Enter THEN nothing is submitted (no empty comments)
- [ ] GIVEN the mention picker is open WHEN the user presses Enter THEN the selected mention is confirmed (NOT submit)

#### Technical Context:
- **Pattern:** Existing submit logic in `handle_comment_compose` (lines 820-896)
- **Files:** `crates/clickup-tui/src/input.rs`
- **Dependencies:** S-1 (compose mode must receive key events first)

#### Agent Hints:
- **Class:** builder
- **Context:** `crates/clickup-tui/src/input.rs` (lines 810-925, 771-790)
- **Gates:**
  - [ ] P0: `cargo build --workspace` succeeds
  - [ ] P0: Enter submits, Alt+Enter inserts newline
  - [ ] P0: Mention picker Enter still confirms selection
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` passes

---

### 📋 STORY: S-3 — Update UI text and help overlay

> 🟡 P1 — User-facing documentation of new keystrokes

**Description:** As a TUI user, I want the compose prompt, bottom bar hints, and help overlay to reflect the new Enter/Alt+Enter keystrokes, so that I can discover the correct keys without guessing.

**Timebox:** ≤1d
**Risk:** P1

#### Action Plan:
1. **Modify:** `render_input_area()` in `comment_sidebar.rs` (line 408) — change `"(Ctrl+D to send)"` to `"(Enter to send, Alt+Enter for newline)"`.
2. **Modify:** `key_hints()` in `ui/mod.rs` (lines 314-331) — when compose mode is active on TaskDetail, show compose-specific hints: `("Enter", "Send")`, `("Alt+Enter", "Newline")`, `("Esc", "Cancel")`.
3. **Modify:** `render()` in `ui/help.rs` (lines 76-81) — add a "Comments" section to the TaskDetail help showing: `Enter` = Send, `Alt+Enter` = Newline, `Esc` = Cancel, `n` = New comment, `e` = Edit, `d` = Delete, `r` = Reply.

#### Acceptance Criteria:
- [ ] GIVEN the user is in compose mode WHEN the compose area renders THEN the prompt shows "(Enter to send, Alt+Enter for newline)"
- [ ] GIVEN the user is in compose mode WHEN they look at the bottom bar THEN they see compose-specific key hints (Send, Newline, Cancel)
- [ ] GIVEN the user presses `?` on the TaskDetail screen (not in compose mode) WHEN the help overlay renders THEN it includes a "Comments" section with accurate keybindings

#### Technical Context:
- **Pattern:** Existing `key_hints()` screen-based dispatch (ui/mod.rs:269-337)
- **Files:** `crates/clickup-tui/src/ui/comment_sidebar.rs`, `crates/clickup-tui/src/ui/mod.rs`, `crates/clickup-tui/src/ui/help.rs`
- **Dependencies:** S-2 (must reflect the new keystrokes, not old ones)

#### Agent Hints:
- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/comment_sidebar.rs` (lines 400-412), `crates/clickup-tui/src/ui/mod.rs` (lines 268-337), `crates/clickup-tui/src/ui/help.rs` (lines 76-81)
- **Gates:**
  - [ ] P0: `cargo build --workspace` succeeds
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` passes
  - [ ] P2: All UI strings reference Enter/Alt+Enter (not Ctrl+D)

---

## ✅ VERIFICATION REPORT

| Layer | Check | Status |
|-------|-------|--------|
| Structural | 3 stories, linear dependency chain, no orphaned tasks | ✓ |
| Self-Consistency | Single viable decomposition — input fix → keystroke remap → UI update | ✓ |
| Dependency | All 4 affected files identified: `input.rs`, `comment_sidebar.rs`, `ui/mod.rs`, `ui/help.rs` | ✓ |
| Constraint | Alt+Enter universally supported without protocol changes; timeboxes conservative | ✓ |
| Process Reward | S-1 unblocks safe compose → S-2 changes keystrokes → S-3 updates docs; each step independently testable | ✓ |
| Adversarial | See notes below | ✓ |

**Adversarial notes:**
- **Ctrl+C in compose:** With S-1's early routing, `Ctrl+C` during compose goes to compose handler, matching `Char('c')` which types 'c'. This matches existing filter-mode behavior. If desired later, compose handler can add explicit `Ctrl+C` → cancel support. Out of scope.
- **Alt+Enter in mention picker:** When the mention picker is active, `Alt+Enter` would dismiss the picker (falls through to `_ =>` at line 803) then the compose handler would see Enter with ALT modifier → newline. This is reasonable — Alt+Enter means "I don't want to pick, just give me a newline."
- **Empty submit protection:** Existing `!text.is_empty()` guard (line 823) prevents empty submits — Enter on empty compose is a no-op.

**Gate:** PASS → Assemble

---

## 📊 CONFIDENCE ASSESSMENT

| Factor | Score |
|--------|-------|
| Pattern Match | 3/3 — exact `filter_active` pattern exists |
| Requirement Clarity | 3/3 — user confirmed Alt+Enter, all gaps resolved |
| Decomposition Stability | 3/3 — single obvious decomposition |
| Constraint Compliance | 2/3 — Alt+Enter universally works; minor Ctrl+C edge case noted |

**Weighted Confidence:** 92%
**Decision:** AUTO_PROCEED

**Known limitations (out of scope):**
- Ctrl+C during compose types 'c' instead of quitting (matches filter-mode precedent)
- Shift+Enter not distinguishable from Enter without Kitty protocol (user chose Alt+Enter)

---

## Execution Sequence

```
S-1 (fix global key bypass) → S-2 (remap Enter/Alt+Enter) → S-3 (update UI text)
```

All stories touch `crates/clickup-tui/src/` only. No API or CLI changes.
