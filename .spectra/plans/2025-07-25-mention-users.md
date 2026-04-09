# Plan: @Mention Users in Comments

> **Spec ID:** SPEC-2025-07-25-MENTIONS
> **Confidence:** 92% — AUTO_PROCEED
> **Complexity:** 8/12 (Extended)
> **SPECTRA Version:** 4.2.0

---

## Overview

Add @mention support for comments across the TUI and CLI crates. When users
include `@username` in a comment, the system resolves usernames against
workspace members and sends the ClickUp structured `comment` array (with
`type: "tag"` items) so that mentioned users receive notifications.

### Approach & Rationale

**Selected:** Dual-Field + Submit-Time Resolution (H1)

- Keep `comment_text` as the plain-text field (backward compatible).
- Add `comment: Vec<CommentContentItem>` alongside for structured content.
- At submit time, scan text for `@username` tokens at word boundaries →
  resolve against workspace members → build the structured array.
- The TUI mention picker inserts `@username` *text*; resolution is decoupled
  from insertion, keeping the model simple.
- Shared `resolve_mentions()` utility in `clickup-api` serves both TUI and CLI.

**Rejected alternatives:**

| # | Name | Why Rejected |
|---|------|-------------|
| H2 | Builder + Offset Tracking | Offset tracking is fragile — shifts on mid-text edits. Submit-time resolution is simpler and more robust. |
| H3 | Unified Content-First Model | Breaking change to `CreateCommentRequest.comment_text`; high blast radius; the API itself uses both fields. |

### Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Insertion model | Display text + submit-time resolution | No offset tracking needed; same code path for picker and manual typing; graceful degradation for unresolved mentions |
| Member freshness | Use startup-loaded members (TUI); fetch on demand (CLI) | Simple, predictable; no automatic polling complexity |
| Serde strategy | `#[serde(untagged)]` with Tag → Text → Unknown order | Tag requires `user` field (differentiates from Text); Unknown catch-all prevents deserialization failures on emoticons/attachments |
| `@` trigger | Word-boundary only (preceded by whitespace or at pos 0) | Prevents false triggers on `user@example.com` |

---

## Story Hierarchy

```
PROJECT: Comment Mentions
└── FEATURE: @Mention Users in Comments
    ├── S-1: Structured Comment Content Models (API)     ≤3d  P0
    ├── S-2: TUI Mention Picker                          ≤3d  P1
    ├── S-3: TUI Mention Submit & Display                ≤2d  P0
    └── S-4: CLI @Mention Parsing                        ≤2d  P1
```

### Dependency Graph

```
S-1 (API models) ─────┬──→ S-3 (TUI submit & display)
                       └──→ S-4 (CLI mentions)

S-2 (TUI picker) ← independent (uses existing WorkspaceMember)
```

### Execution Sequence

| Phase | Stories | Parallel? | Gate |
|-------|---------|-----------|------|
| 1 | S-1 + S-2 | Yes | S-1: serde tests pass; S-2: picker renders |
| 2 | S-3 + S-4 | Yes (after S-1) | Structured comments sent to API; CLI mentions resolve |

---

## S-1: Structured Comment Content Models

> **As a** library consumer (TUI/CLI),
> **I want** serde models for the ClickUp structured comment format with a
> shared mention-resolution utility,
> **so that** comments with @mentions can be correctly serialized,
> deserialized, and built from plain text.

**Timebox:** ≤3d | **Risk:** P0

### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | **Create** `TaggedUser` struct | `crates/clickup-api/src/models/comment.rs` | `{ id: i64, username: Option<String>, email: Option<String> }` — `skip_serializing_if = "Option::is_none"` on optional fields. Derive `Debug, Clone, Serialize, Deserialize`. |
| 2 | **Create** `CommentContentItem` enum | `crates/clickup-api/src/models/comment.rs` | `#[serde(untagged)]` with three variants in order: **Tag** `{ #[serde(rename = "type")] content_type: String, user: TaggedUser, text: Option<String> }`, **Text** `{ text: String }`, **Unknown** `(serde_json::Value)`. |
| 3 | **Create** `MentionResolution` struct | `crates/clickup-api/src/models/comment.rs` | `{ comment_content: Vec<CommentContentItem>, resolved: Vec<String>, unresolved: Vec<String> }`. |
| 4 | **Create** `resolve_mentions()` function | `crates/clickup-api/src/models/comment.rs` | `pub fn resolve_mentions(text: &str, members: &[WorkspaceMember]) -> MentionResolution`. Scans for `@word` tokens at word boundaries. Resolved → Tag item, unresolved → plain Text. |
| 5 | **Extend** `Comment` struct | `crates/clickup-api/src/models/comment.rs` | Add `#[serde(default)] pub comment: Vec<CommentContentItem>`. |
| 6 | **Extend** `CreateCommentRequest` | `crates/clickup-api/src/models/comment.rs` | Add `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub comment: Vec<CommentContentItem>`. |
| 7 | **Extend** docs | `docs/clickup-api-quirks.md` | Document structured comment format. |
| 8 | **Test** all new types and functions | `crates/clickup-api/src/models/comment.rs` | Serde round-trip tests + resolve_mentions tests (see acceptance criteria). |

### Acceptance Criteria

- [ ] GIVEN a JSON response with `"comment": [{"text": "Hi "}, {"type": "tag", "user": {"id": 123, "username": "alice"}, "text": "@alice"}, {"text": " check this"}]` WHEN deserialized into `Comment` THEN `comment` field contains `[Text, Tag, Text]` items with correct values
- [ ] GIVEN a JSON response with no `comment` field WHEN deserialized into `Comment` THEN `comment` defaults to empty vec (backward compatible)
- [ ] GIVEN `CreateCommentRequest` with non-empty `comment` vec WHEN serialized THEN JSON includes `"comment"` key with structured array
- [ ] GIVEN `CreateCommentRequest` with empty `comment` vec WHEN serialized THEN JSON omits `"comment"` key entirely
- [ ] GIVEN text `"Please review @alice and @bob"` with matching members WHEN `resolve_mentions()` called THEN returns structured array with two Tag items and resolved=`["alice","bob"]`
- [ ] GIVEN text `"Email user@example.com"` WHEN `resolve_mentions()` called THEN no mention resolved (@ not at word boundary)
- [ ] GIVEN text `"@unknown please help"` with no matching member WHEN `resolve_mentions()` called THEN unresolved=`["unknown"]`, content is plain text
- [ ] GIVEN an unknown JSON item type (e.g., emoticon) WHEN deserialized THEN matches `Unknown` variant (no error)
- [ ] `cargo build --workspace` compiles; `cargo test --workspace` passes

### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/models/comment.rs`, `crates/clickup-api/src/models/workspace.rs`, `crates/clickup-api/src/serde_helpers.rs`
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles with no errors
  - [ ] P0: No `.unwrap()` outside test code; no `panic!()` in library code
  - [ ] P0: Crate boundary respected — `clickup-api` has no CLI/TUI deps
  - [ ] P1: `cargo test --workspace` — all tests pass (existing + new)
  - [ ] P1: Serde tests cover full, minimal, null-heavy, and type-mixed JSON
  - [ ] P1: `resolve_mentions` tested with boundary conditions
  - [ ] P2: `cargo fmt --all -- --check` passes
  - [ ] P2: `cargo clippy --workspace -- -D warnings` passes
  - [ ] P2: All public items have `///` doc comments

---

## S-2: TUI Mention Picker

> **As a** TUI user composing a comment,
> **I want** a floating member picker triggered by `@` that lets me search
> and select workspace members,
> **so that** I can quickly insert `@username` mentions without remembering
> exact spellings.

**Timebox:** ≤3d | **Risk:** P1

### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | **Extend** `App` struct | `crates/clickup-tui/src/app.rs` | Add `mention_picker_active: bool`, `mention_picker_filter: String`, `mention_picker_selected: usize`. |
| 2 | **Create** `filtered_members()` | `crates/clickup-tui/src/app.rs` | Helper returning members matching filter (case-insensitive substring). |
| 3 | **Modify** `@` trigger | `crates/clickup-tui/src/input.rs` | Intercept `@` in `KeyCode::Char(c)` handler; activate picker only at word boundary. |
| 4 | **Modify** input routing | `crates/clickup-tui/src/input.rs` | When picker active: intercept Char/Backspace/Up/Down/Enter/Esc before normal handling. |
| 5 | **Create** selection logic | `crates/clickup-tui/src/input.rs` | On Enter: replace `@{filter}` with `@{username} ` in input text, dismiss picker. |
| 6 | **Create** picker overlay | `crates/clickup-tui/src/ui/comment_sidebar.rs` | `render_mention_picker()`: floating box above input area, up to 5 rows, selected row highlighted. |
| 7 | **Modify** `render()` | `crates/clickup-tui/src/ui/comment_sidebar.rs` | Call picker overlay when `mention_picker_active`. |
| 8 | **Test** filtered_members | `crates/clickup-tui/src/app.rs` | Test filter with various strings, empty filter, no workspace. |

### Acceptance Criteria

- [ ] GIVEN user types `@` at start of text or after whitespace THEN picker opens showing workspace members
- [ ] GIVEN user types `@` after non-whitespace (e.g., `user@`) THEN picker does NOT open
- [ ] GIVEN picker open and user types filter THEN list narrows to matching members
- [ ] GIVEN picker with selection WHEN Enter pressed THEN `@username ` inserted and picker closes
- [ ] GIVEN picker open WHEN Esc pressed THEN picker closes, typed text remains
- [ ] GIVEN picker open with empty filter WHEN Backspace pressed THEN picker closes and `@` removed
- [ ] GIVEN picker open WHEN Down/Up pressed THEN selection moves with wrapping
- [ ] GIVEN picker renders THEN floating box above input with rounded border, up to 5 visible rows

### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/input.rs` (line 794-808), `crates/clickup-tui/src/ui/comment_sidebar.rs`, `crates/clickup-tui/src/app.rs`
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside test code
  - [ ] P0: Crate boundary — `clickup-tui` depends only on `clickup-api`
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: Picker doesn't trigger on email-like patterns
  - [ ] P2: `cargo fmt --all -- --check` passes
  - [ ] P2: `cargo clippy --workspace -- -D warnings` passes
  - [ ] P2: All public items have `///` doc comments

---

## S-3: TUI Mention Submit & Display

> **As a** TUI user,
> **I want** my `@username` mentions to be sent as structured tags
> (triggering ClickUp notifications) and displayed with visual highlighting,
> **so that** mentioned users are notified and mentions are visually distinct.

**Timebox:** ≤2d | **Risk:** P0

### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | **Modify** `spawn_create_comment` | `crates/clickup-tui/src/data.rs` | Add `comment_content: Vec<CommentContentItem>` parameter; include in `CreateCommentRequest`. |
| 2 | **Modify** `spawn_create_comment_reply` | `crates/clickup-tui/src/data.rs` | Same change. |
| 3 | **Modify** submit handler | `crates/clickup-tui/src/input.rs` | Call `resolve_mentions()` before spawning; pass `resolution.comment_content` to spawn function. Warn on unresolved. |
| 4 | **Modify** reply submit handler | `crates/clickup-tui/src/input.rs` | Same resolution logic. |
| 5 | **Modify** comment rendering | `crates/clickup-tui/src/ui/comment_sidebar.rs` | If `comment.comment` is non-empty: render Tag items as cyan+bold `@username` spans. Fall back to `comment_text`. |
| 6 | **Modify** input display | `crates/clickup-tui/src/ui/comment_sidebar.rs` | Highlight `@username` tokens in input text (cyan for resolved members, dim for unresolved). |

### Acceptance Criteria

- [ ] GIVEN `"@alice"` in comment text (alice is member id=123) WHEN submitted THEN API request contains `"comment"` array with tag `{"type":"tag","user":{"id":123}}`
- [ ] GIVEN `"@nonexistent"` WHEN submitted THEN plain `comment_text` sent (no structured array), tracing warning emitted
- [ ] GIVEN received comment with structured `comment` array containing tags WHEN rendered THEN `@username` shown in cyan/bold
- [ ] GIVEN received comment with empty `comment` array WHEN rendered THEN `comment_text` displayed as before
- [ ] Existing comment creation without mentions works unchanged

### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/data.rs` (line 154), `crates/clickup-tui/src/input.rs`, `crates/clickup-tui/src/ui/comment_sidebar.rs`
- **Dependencies:** S-1
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside test code
  - [ ] P0: Existing comment creation still works
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt --all -- --check` passes
  - [ ] P2: `cargo clippy --workspace -- -D warnings` passes

---

## S-4: CLI @Mention Parsing

> **As a** CLI user creating or replying to a comment,
> **I want** `@username` tokens in my message to be automatically resolved
> to ClickUp user tags,
> **so that** mentioned users receive notifications without me needing to
> know their numeric IDs.

**Timebox:** ≤2d | **Risk:** P1

### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | **Create** helper | `crates/clickup-cli/src/commands/comments.rs` | `async fn resolve_mentions_for_message(client, config, message) -> Result<(String, Vec<CommentContentItem>)>`. Only calls `get_workspaces()` when `@` patterns detected. |
| 2 | **Modify** `create_comment()` | `crates/clickup-cli/src/commands/comments.rs` | Call helper; use structured content in request; print resolved/unresolved info. |
| 3 | **Modify** `reply_comment()` | `crates/clickup-cli/src/commands/comments.rs` | Same integration. |
| 4 | **Modify** comment display | `crates/clickup-cli/src/commands/comments.rs` | In `list_comments`: if `comment.comment` non-empty, render Tag items as cyan `@username`. |
| 5 | **Test** integration | `crates/clickup-cli/src/commands/comments.rs` | Verify helper compiles and basic resolution path. |

### Acceptance Criteria

- [ ] GIVEN `--message "Hey @alice check this"` with alice as member WHEN executed THEN API request includes structured tag, CLI prints `ℹ Mentioned: @alice`
- [ ] GIVEN `@unknown` in message WHEN executed THEN CLI warns about unresolved mention, sends plain text
- [ ] GIVEN message with no `@` patterns WHEN executed THEN no `get_workspaces()` call (zero overhead)
- [ ] GIVEN no `default_workspace_id` configured WHEN message has mentions THEN CLI warns, sends as plain text
- [ ] GIVEN `list` command with structured comments THEN mentions show as `@username` in cyan

### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/comments.rs`, `crates/clickup-cli/src/client_factory.rs`, `crates/clickup-api/src/endpoints/workspaces.rs`
- **Dependencies:** S-1
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside test code
  - [ ] P0: Existing commands without mentions still work
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: No `get_workspaces()` call when message has no `@` patterns
  - [ ] P2: `cargo fmt --all -- --check` passes
  - [ ] P2: `cargo clippy --workspace -- -D warnings` passes
  - [ ] P2: All public items have `///` doc comments

---

## Files Affected

| File | Stories | Change Type |
|------|---------|-------------|
| `crates/clickup-api/src/models/comment.rs` | S-1 | Extend (new types + fields + function + tests) |
| `crates/clickup-api/src/models/mod.rs` | S-1 | Verify (glob re-export covers new types) |
| `docs/clickup-api-quirks.md` | S-1 | Extend (new section) |
| `crates/clickup-tui/src/app.rs` | S-2 | Extend (picker state + helper) |
| `crates/clickup-tui/src/input.rs` | S-2, S-3 | Modify (@ trigger, picker nav, submit logic) |
| `crates/clickup-tui/src/ui/comment_sidebar.rs` | S-2, S-3 | Extend (overlay, mention highlighting) |
| `crates/clickup-tui/src/data.rs` | S-3 | Modify (spawn function signatures) |
| `crates/clickup-cli/src/commands/comments.rs` | S-4 | Modify (mention resolution + display) |

---

## Confidence Report

| Factor | Score | Rationale |
|--------|-------|-----------|
| Pattern Match | 3/3 | All patterns exist; extending not inventing |
| Requirement Clarity | 3/3 | All decisions pre-answered; API format documented |
| Decomposition Stability | 2/3 | ~80% overlap across alternative decompositions |
| Constraint Compliance | 3/3 | Crate boundaries, timeboxes, conventions all met |

**Weighted Confidence:** 92% — **AUTO_PROCEED**
