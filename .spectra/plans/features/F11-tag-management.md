# F11 — Tag Management

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 3 — Task Enrichment
> **Complexity:** 4/12 | **Confidence:** 90%

---

## Problem Statement

Tags are a lightweight organizational primitive in ClickUp — colored labels that can be applied to tasks for cross-list filtering and visual grouping. termaup renders tags on the TaskDetail screen (displaying colored tag names) but provides no way to add or remove tags from tasks, nor to list which tags are available in a space. Since tag operations use the tag name (not ID) in the URL path, the API surface is minimal and well-defined.

## Approach

Add three API endpoints: `get_space_tags()` for listing available tags, `add_task_tag()` for tagging a task, and `remove_task_tag()` for untagging. The CLI provides `clickup tag list --space SPACE_ID` and `clickup task tag TASK_ID --add "bug" --remove "wontfix"`. The TUI presents a tag picker on TaskDetail (press `t`) showing available space tags with multi-select — the diff between current and selected tags drives add/remove API calls.

### Rejected Alternatives

1. **Tag CRUD (create/delete tags)** — Managing tag definitions (creating new tags, deleting tags from a space) is a workspace admin operation. Out of scope for MVP; users will use existing tags. Rejected for scope.

2. **Free-text tag input** — Let users type arbitrary tag names instead of picking from available tags. ClickUp API rejects tags that don't exist in the space, so free-text input would fail silently. Rejected; picker from available tags is safer.

3. **Inline tag editing on TaskDetail** — Edit tags directly in the tag display line. Awkward UX for multi-select; a picker overlay is more intuitive. Rejected for UX.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `GET /space/{id}/tag` endpoint | Tag creation/deletion (space admin) | Tag color customization |
| `POST /task/{id}/tag/{name}` endpoint | Tag renaming | Tag-based task filtering in TUI |
| `DELETE /task/{id}/tag/{name}` endpoint | Tag usage statistics | Bulk tag operations |
| CLI `tag list` and `task tag` commands | | |
| TUI tag picker with multi-select | | |
| `SpaceTagsResponse` model | | |

## Stories

### S-1: API Endpoints

**As a** developer using clickup-api, **I want** methods to list space tags, add a tag to a task, and remove a tag from a task, **so that** CLI and TUI can manage tags.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/src/endpoints/tags.rs` | Add `get_space_tags(&self, space_id: &str) -> Result<Vec<Tag>>`: `GET /space/{space_id}/tag`, deserialize `SpaceTagsResponse { tags: Vec<Tag> }`. Log with `tracing::debug!`. |
| 2 | Extend | `crates/clickup-api/src/endpoints/tags.rs` | Add `add_task_tag(&self, task_id: &str, tag_name: &str) -> Result<()>`: `POST /task/{task_id}/tag/{tag_name}` with empty body. Note: tag name must be URL-encoded. |
| 3 | Extend | `crates/clickup-api/src/endpoints/tags.rs` | Add `remove_task_tag(&self, task_id: &str, tag_name: &str) -> Result<()>`: `DELETE /task/{task_id}/tag/{tag_name}`. |
| 4 | Create | `crates/clickup-api/src/models/tag.rs` | Add `SpaceTagsResponse` struct: `tags: Vec<Tag>` with `#[serde(default, deserialize_with = "deserialize_null_as_default")]`. Note: `Tag` model already exists in `task.rs` with `name`, `tag_fg`, `tag_bg`. If Tag is defined inline in task.rs, extract to its own module or re-export. |
| 5 | Modify | `crates/clickup-api/src/endpoints/mod.rs` | Add `pub mod tags;` declaration. |
| 6 | Test | `crates/clickup-api/tests/fixtures/space_tags.json` | Create fixture: `{"tags":[{"name":"bug","tag_fg":"#fff","tag_bg":"#ff0000"},{"name":"feature","tag_fg":"#fff","tag_bg":"#00ff00"}]}`. |
| 7 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_get_space_tags` — verify deserialization returns 2 tags with correct names and colors. |
| 8 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_add_task_tag` — verify POST to correct path with URL-encoded tag name. |
| 9 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock: `test_remove_task_tag` — verify DELETE to correct path. |

#### Acceptance Criteria

- [ ] GIVEN a space with tags, WHEN `client.get_space_tags("space1")` is called, THEN `GET /space/space1/tag` is sent and a `Vec<Tag>` is returned with names and colors.
- [ ] GIVEN a task and tag name "bug", WHEN `client.add_task_tag("task1", "bug")` is called, THEN `POST /task/task1/tag/bug` is sent and `Ok(())` is returned.
- [ ] GIVEN a tag name with spaces "needs review", WHEN `client.add_task_tag("task1", "needs review")` is called, THEN the tag name is properly URL-encoded in the path.
- [ ] GIVEN a task and tag name "wontfix", WHEN `client.remove_task_tag("task1", "wontfix")` is called, THEN `DELETE /task/task1/tag/wontfix` is sent.
- [ ] GIVEN an empty space (no tags), WHEN `client.get_space_tags()` is called, THEN an empty `Vec` is returned.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/endpoints/comments.rs` (endpoint exemplar), `crates/clickup-api/src/models/task.rs` (existing Tag struct), `crates/clickup-api/src/client.rs` (delete and post methods)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Wiremock tests for all 3 endpoints
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean
  - [ ] P2: `///` doc comments on all public methods

---

### S-2: CLI Tag Commands

**As a** CLI user, **I want** `clickup tag list --space SPACE_ID` to see available tags and `clickup task tag TASK_ID --add "bug" --remove "wontfix"` to manage task tags, **so that** I can tag tasks from the command line.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-cli/src/commands/tags.rs` | Add `TagCommands` enum: `List { #[arg(long)] space: String }`. Implement: call `client.get_space_tags()` → display table with columns: Name, Foreground, Background. Support `--format` (table/json). |
| 2 | Extend | `crates/clickup-cli/src/commands/tasks.rs` | Add `Tag` variant to `TaskCommands`: `task_id: String`, `#[arg(long)] add: Vec<String>`, `#[arg(long)] remove: Vec<String>`. Implement: iterate `add` tags calling `client.add_task_tag()` for each, iterate `remove` tags calling `client.remove_task_tag()` for each, print ✓ summary. |
| 3 | Modify | `crates/clickup-cli/src/commands/mod.rs` | Add `pub mod tags;` declaration. |
| 4 | Modify | `crates/clickup-cli/src/main.rs` | Add `Tag(TagCommands)` to root `Commands` enum; wire `TaskCommands::Tag` in task dispatch. |

#### Acceptance Criteria

- [ ] GIVEN `clickup tag list --space space1`, WHEN executed, THEN a table is printed showing available tag names and their colors.
- [ ] GIVEN `clickup task tag abc123 --add "bug"`, WHEN executed, THEN `POST /task/abc123/tag/bug` is called and `✓ Tag "bug" added to task abc123` is printed.
- [ ] GIVEN `clickup task tag abc123 --add "bug" --add "urgent" --remove "wontfix"`, WHEN executed, THEN three API calls are made (2 adds + 1 remove) and a summary is printed.
- [ ] GIVEN `clickup task tag abc123` with no `--add` or `--remove`, WHEN executed, THEN an error message is printed.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (TaskCommands, Vec<String> arg pattern), `crates/clickup-cli/src/main.rs` (dispatch), `crates/clickup-cli/src/output.rs` (table formatting)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-3: TUI Tag Picker

**As a** TUI user on the TaskDetail screen, **I want** to press `t` to open a tag picker showing available space tags where I can toggle selections and apply changes, **so that** I can manage tags without leaving the TUI.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-tui/src/app.rs` | Add state: `tag_picker_open: bool`, `tag_picker_selected: usize`, `tag_picker_toggled: HashSet<String>` (tag names currently selected), `available_tags: Vec<Tag>` (loaded from space). |
| 2 | Extend | `crates/clickup-tui/src/data.rs` | Add `spawn_load_space_tags()`: calls `client.get_space_tags()` → sends `DataPayload::SpaceTags(Vec<Tag>)`. Called when tag picker opens (or cached from previous load). |
| 3 | Extend | `crates/clickup-tui/src/data.rs` | Add `spawn_add_task_tag()` and `spawn_remove_task_tag()`: each calls the corresponding API method → sends success/error event. |
| 4 | Extend | `crates/clickup-tui/src/event.rs` | Add `DataPayload::SpaceTags(Vec<Tag>)` variant. Add `DataPayload::TagsUpdated` variant (triggers task detail refresh). |
| 5 | Create | `crates/clickup-tui/src/ui/tag_picker.rs` | Render floating panel: list available tags with colored background chips. Each row: `[x]`/`[ ]` checkbox + tag name (with `tag_bg` color). Bottom bar: `Space: toggle | Enter: apply | Esc: cancel`. |
| 6 | Modify | `crates/clickup-tui/src/ui/mod.rs` | Add `pub mod tag_picker;` declaration. |
| 7 | Extend | `crates/clickup-tui/src/input.rs` | In `handle_task_detail()`: `t` key → load space tags (if not cached) → open tag picker. When picker is open: `j`/`↓`/`k`/`↑` navigate, `Space` toggle, `Enter` compute diff → call add/remove for each changed tag, `Esc` cancel. |
| 8 | Modify | `crates/clickup-tui/src/ui/task_detail.rs` | When `tag_picker_open`, render picker overlay on top. |

#### Acceptance Criteria

- [ ] GIVEN the user is on TaskDetail with a task tagged ["bug"], WHEN they press `t`, THEN the tag picker opens with "bug" pre-checked and all space tags listed.
- [ ] GIVEN the picker is open, WHEN the user checks "feature" and unchecks "bug" and presses Enter, THEN `POST /task/{id}/tag/feature` and `DELETE /task/{id}/tag/bug` are called.
- [ ] GIVEN the picker is open with no changes, WHEN Enter is pressed, THEN no API calls are made.
- [ ] GIVEN the picker is open, WHEN Esc is pressed, THEN the picker closes with no changes.
- [ ] GIVEN tags have colors (tag_bg), WHEN rendered in the picker, THEN each tag name is displayed with its background color.
- [ ] GIVEN the space has no tags, WHEN the picker opens, THEN an empty list message is shown: "No tags available in this space".

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/assignee_picker.rs` (from F08 — multi-select picker exemplar if available, otherwise `crates/clickup-tui/src/ui/comment_sidebar.rs` for floating panel), `crates/clickup-tui/src/data.rs` (spawn pattern), `crates/clickup-tui/src/app.rs` (tag color rendering from task detail)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P1: Render test for tag picker with colored tags
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

## Execution Sequence

```
S-1 (API, 3 endpoints) ──┬──→ S-2 (CLI commands)
                          └──→ S-3 (TUI tag picker)
```

- **Phase 1:** S-1 — API endpoints
- **Phase 2 (parallel):** S-2 (CLI) + S-3 (TUI)

## Assumptions

1. The `POST /task/{id}/tag/{tag_name}` endpoint accepts the tag name in the URL path (not in the request body). **Risk if wrong:** Move tag name to body — trivial change.
2. Tag names are unique within a space. **Risk if wrong:** May have duplicate display names — picker would show duplicates. Minimal impact.
3. The `POST` and `DELETE` tag endpoints return empty bodies (no response content). **Risk if wrong:** Add response struct — low impact.
4. Space tags are accessible from the TUI context — the current space ID is known from the navigation breadcrumb. **Risk if wrong:** Need to traverse from task → list → space to find the space ID.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 1 | 3 simple endpoints, CLI commands, TUI picker — all follow established patterns |
| Ambiguity | 1 | Tag API is simple and well-documented; name-in-path pattern is clear |
| Dependencies | 1 | No feature dependencies; reuses existing Tag model |
| Risk | 1 | Low risk; simple string-based operations with no complex serialization |

**Total: 4/12** → Fast-track processing
