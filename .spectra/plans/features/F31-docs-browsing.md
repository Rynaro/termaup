# F31 — ClickUp Docs Browsing

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 7 — Advanced Features
> **Complexity:** 7/12 | **Confidence:** 55% → VALIDATE

---

## Problem Statement

ClickUp Docs are a core collaboration feature used for wikis, meeting notes, specs, and knowledge bases. termaup users cannot access Docs content at all — they must switch to the web UI to read or reference any Doc. For power users who treat the terminal as their primary workspace, this is a significant gap.

However, the ClickUp API v2 has **limited and inconsistent Docs support**. Docs may be surfaced as special `type: "doc"` views via the Views API, but a dedicated Docs listing endpoint and content retrieval endpoint may not exist in v2. The upcoming v3 API is expected to have full Docs support, but its timeline is uncertain.

## Approach

Take a conservative, discovery-first approach:

1. **Use the Views API** to find docs: `GET /team/{team_id}/view` and filter by `type == "doc"` to discover Doc entities.
2. **Probe the Docs API** for content: test `GET /doc/{doc_id}` or `GET /view/{view_id}` for doc content. The ClickUp API v2 may return page content as markdown or HTML through undocumented endpoints.
3. **Graceful degradation**: If content retrieval is not available, show doc metadata (name, creator, date) and open in browser on Enter.

The first story explicitly validates API availability before building features. If the API proves insufficient, the feature is scoped down to "Docs discovery + browser open" rather than full content rendering.

### Rejected Alternatives

1. **Wait for v3 API** — Delays indefinitely; v3 timeline is unknown. A partial implementation with v2 provides value today and positions the codebase for easy v3 adoption. Rejected for indefinite delay.

2. **Scrape web UI** — Load the ClickUp web UI in a headless browser to extract Doc content. Fragile, slow, and potentially against ClickUp's ToS. Rejected for unreliability and policy concerns.

3. **Treat Docs as tasks** — Map Doc pages to tasks for browsing. Conceptually wrong — Docs are a separate entity type with different behavior. Rejected for semantic incorrectness.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| API exploration + endpoint validation | Doc creation/editing | Full Doc CRUD (waiting for v3 API) |
| Doc listing via Views API (`type: "doc"`) | Doc collaboration (real-time editing) | Doc search within content |
| Doc content display (if API supports it) | Doc page hierarchy (sub-pages) | Doc export (PDF, Markdown file) |
| Fallback: open Doc in browser | Doc comments | Doc version history |
| CLI `doc list` and `doc view` commands | | |
| TUI Docs screen with markdown rendering | | |

## Stories

### S-1: API Exploration + Endpoints

**As a** developer, **I want** to validate ClickUp's Docs API availability and implement the best available endpoints, **so that** the feature can be built on a reliable foundation or scoped down early.

**Timebox:** ≤2d | **Risk:** High (API may not exist) | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Validate Views-based discovery | Manual testing | Call `GET /team/{team_id}/view` on a workspace with Docs. Verify that docs appear as `type: "doc"` views. Document the response shape |
| 2 | Probe doc content endpoint | Manual testing | Test `GET /view/{view_id}` where the view is a doc. Check if content is returned. Test `GET /doc/{doc_id}` if such an endpoint exists. Try query params like `?include_content=true` |
| 3 | Document findings | `docs/clickup-api-quirks.md` | Add "Docs API v2 limitations" section documenting which endpoints work, response format, and content availability |
| 4 | Create `Doc` model | `crates/clickup-api/src/models/doc.rs` | If content is available: `pub struct Doc { pub id: String, pub name: String, pub content: Option<String>, pub date_created: Option<String>, pub creator: Option<User>, pub parent: Option<DocParent>, pub workspace_id: Option<String> }`. If not: reuse `View` model filtered by type |
| 5 | Create response wrapper | `crates/clickup-api/src/models/doc.rs` | `pub struct DocsResponse { pub docs: Vec<Doc> }` or reuse `ViewsResponse` |
| 6 | Register in models/mod.rs | `crates/clickup-api/src/models/mod.rs` | `pub mod doc;` |
| 7 | Implement listing endpoint | `crates/clickup-api/src/endpoints/docs.rs` | `pub async fn get_docs(&self, team_id: &str) -> Result<Vec<Doc>>` — uses Views API filtered by `type: "doc"`, or dedicated endpoint if available |
| 8 | Implement content endpoint | `crates/clickup-api/src/endpoints/docs.rs` | `pub async fn get_doc(&self, doc_id: &str) -> Result<Doc>` — fetches doc metadata + content if available |
| 9 | Implement browser fallback | `crates/clickup-api/src/endpoints/docs.rs` | `pub fn doc_web_url(workspace_id: &str, doc_id: &str) -> String` — returns `https://app.clickup.com/{workspace_id}/docs/{doc_id}` for browser opening |
| 10 | Register in endpoints/mod.rs | `crates/clickup-api/src/endpoints/mod.rs` | `pub mod docs;` |
| 11 | Unit tests | `crates/clickup-api/src/models/doc.rs` | Deserialization tests with mock data; test with and without content field |

#### Acceptance Criteria

- **GIVEN** a workspace with Docs, **WHEN** `get_docs(team_id)` is called, **THEN** it returns a list of doc entities with at least `id` and `name`.
- **GIVEN** a doc ID where content IS available, **WHEN** `get_doc(doc_id)` is called, **THEN** `doc.content` is `Some(markdown_string)`.
- **GIVEN** a doc ID where content is NOT available, **WHEN** `get_doc(doc_id)` is called, **THEN** `doc.content` is `None` and `doc_web_url()` can be used as fallback.
- **GIVEN** any API response shape, **WHEN** deserialized, **THEN** no panic occurs — all non-ID fields are `Option<T>`.
- **GIVEN** API exploration results, **WHEN** documented, **THEN** `docs/clickup-api-quirks.md` has a section on Docs API v2 limitations.

#### Agent Hints

- **Class:** explorer + builder
- **Context:** This story has an explicit EXPLORATION phase. Start with manual API testing before writing code. If the Views API is the only way to discover docs, the `get_docs()` endpoint will internally call `get_workspace_views()` and filter by type. The `content` field may be markdown, HTML, or a ClickUp-proprietary format — use `Option<String>` and render whatever is available. If content is entirely unavailable, document this and adjust S-2/S-3 scope.
- **Decision gate:** After step 3, evaluate: if content retrieval is not possible, reduce S-3 to "list + browser open" only (no content rendering).
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: API exploration findings documented
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P2: `cargo clippy` clean

---

### S-2: CLI Doc Commands

**As a** CLI user, **I want** `clickup doc list` and `clickup doc view DOC_ID`, **so that** I can discover and read Docs from the terminal.

**Timebox:** ≤1d | **Risk:** Low (if S-1 validated API) | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create command module | `crates/clickup-cli/src/commands/docs.rs` | `DocCommands` enum with `List { space_id: Option<String> }` and `View { doc_id: String }` |
| 2 | Implement `list` handler | `crates/clickup-cli/src/commands/docs.rs` | Fetch docs via `client.get_docs()`, display as table (name, creator, date created) |
| 3 | Implement `view` handler (content available) | `crates/clickup-cli/src/commands/docs.rs` | If `doc.content` is available, render with `termimad` for markdown output or print raw for JSON format |
| 4 | Implement `view` handler (content unavailable) | `crates/clickup-cli/src/commands/docs.rs` | If content is `None`, print doc metadata and offer to open in browser: `"Opening in browser: {url}"`, then call `open::that(url)` or print the URL |
| 5 | JSON + Markdown format support | `crates/clickup-cli/src/commands/docs.rs` | Support `--format json` and `--format markdown` |
| 6 | Register in mod.rs | `crates/clickup-cli/src/commands/mod.rs` | `pub mod docs;` |
| 7 | Register in main.rs | `crates/clickup-cli/src/main.rs` | Add `Doc { command: DocCommands }` to `Commands` enum |

#### Acceptance Criteria

- **GIVEN** `clickup doc list`, **WHEN** run, **THEN** a table of docs is displayed with name, creator, and creation date.
- **GIVEN** `clickup doc list --space SPACE_ID`, **WHEN** run, **THEN** only docs in that space are listed.
- **GIVEN** `clickup doc view DOC_ID` with content available, **WHEN** run, **THEN** the doc content is rendered as formatted markdown in the terminal.
- **GIVEN** `clickup doc view DOC_ID` with NO content available, **WHEN** run, **THEN** the doc metadata is printed and the browser URL is shown.
- **GIVEN** `--format json`, **WHEN** any doc command is run, **THEN** output is pretty-printed JSON.

#### Agent Hints

- **Class:** builder
- **Context:** Follow `crates/clickup-cli/src/commands/tasks.rs` for command structure. Use `termimad` (already a dependency) for markdown rendering of doc content. The `view` subcommand name conflicts with the View feature (F30) — the clap subcommand is scoped under `doc` so `clickup doc view` vs `clickup view list` are distinct.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo clippy` clean

---

### S-3: TUI Docs Screen

**As a** TUI user, **I want** a Docs screen showing available docs with content rendering, **so that** I can read documentation and notes without leaving the terminal.

**Timebox:** ≤2d | **Risk:** Medium (content rendering quality depends on API) | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `Docs` and `DocDetail` screen variants | `crates/clickup-tui/src/app.rs` | Add to `Screen` enum |
| 2 | Add doc state to `App` | `crates/clickup-tui/src/app.rs` | `pub docs: Vec<Doc>`, `pub selected_doc_index: usize`, `pub selected_doc: Option<Doc>` |
| 3 | Add data payload variants | `crates/clickup-tui/src/event.rs` | `DataPayload::Docs(Vec<Doc>)`, `DataPayload::DocDetail(Doc)` |
| 4 | Implement data loader | `crates/clickup-tui/src/data.rs` | `pub fn spawn_load_docs(client, tx, team_id)` |
| 5 | Create docs list screen | `crates/clickup-tui/src/ui/docs.rs` | List of docs with name, creator, date. Highlight selected. |
| 6 | Create doc detail screen | `crates/clickup-tui/src/ui/docs.rs` | If content available: render markdown using `pulldown-cmark` → ratatui spans (reuse pattern from task description rendering). If not: show metadata + "Press Enter to open in browser" |
| 7 | Register in ui/mod.rs | `crates/clickup-tui/src/ui/mod.rs` | `pub mod docs;` |
| 8 | Add navigation from WorkspaceSelect | `crates/clickup-tui/src/input.rs` | Press `d` to navigate to Docs screen from WorkspaceSelect |
| 9 | Add input handlers | `crates/clickup-tui/src/input.rs` | `handle_docs()` — j/k navigation, Enter to view doc content or open in browser, Esc to go back |
| 10 | Handle data events | `crates/clickup-tui/src/app.rs` | Handle `DataPayload::Docs` and `DataPayload::DocDetail` |
| 11 | Browser open on Enter (no content) | `crates/clickup-tui/src/input.rs` | If selected doc has no content, open `doc_web_url()` in browser and show feedback message |

#### Acceptance Criteria

- **GIVEN** the WorkspaceSelect screen, **WHEN** `d` is pressed, **THEN** the TUI navigates to the Docs screen and loads docs for the selected workspace.
- **GIVEN** the Docs screen with loaded docs, **WHEN** rendered, **THEN** each doc shows its name, creator, and creation date.
- **GIVEN** a doc selected with Enter and content available, **WHEN** the doc detail is shown, **THEN** markdown content is rendered with proper formatting (headers, lists, code blocks).
- **GIVEN** a doc selected with Enter and NO content available, **WHEN** the doc detail is shown, **THEN** the doc opens in the user's browser.
- **GIVEN** the Docs screen, **WHEN** `Esc` is pressed, **THEN** the user returns to WorkspaceSelect.
- **GIVEN** no docs exist in the workspace, **WHEN** the Docs screen loads, **THEN** a "No docs found" message is displayed.

#### Agent Hints

- **Class:** builder
- **Context:** Follow `crates/clickup-tui/src/ui/workspace_select.rs` for screen list pattern. For markdown rendering, the TaskDetail screen already renders markdown descriptions — check `crates/clickup-tui/src/ui/task_detail.rs` for the existing approach using `pulldown-cmark`. The browser open function can reuse the same utility from F23-S-3 (`open_browser()`), or use `std::process::Command` directly. The `d` shortcut should only be active on WorkspaceSelect.
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo clippy` clean

---

## Execution Sequence

```
S-1 (API exploration + endpoints) ──┬──→ S-2 (CLI commands)
                                    └──→ S-3 (TUI docs screen)
```

**CRITICAL:** S-1 has a decision gate after API exploration. If content retrieval is not possible:
- S-2 scope reduces to: `list` + `view` (metadata + browser open only)
- S-3 scope reduces to: list screen + browser open on Enter (no content rendering)

S-2 and S-3 can execute in parallel after S-1.

## Assumptions

1. **Docs appear as `type: "doc"` in the Views API**. Risk if wrong: HIGH — the feature's discovery mechanism fails. Mitigation: manual API testing in S-1 will validate this before building features.
2. **Doc content (if available) is returned as markdown or plain text**. Risk if wrong: may be ClickUp-proprietary format requiring custom parsing. Mitigation: render as-is and iterate.
3. **The ClickUp v3 API will provide full Docs support**. Risk if wrong: v2 limitations may persist. Mitigation: this feature plan works with or without content — the browser fallback provides value regardless.
4. **`GET /doc/{doc_id}` may not exist in v2**. Risk if wrong: actually EXPECTED — the Views API is the primary discovery mechanism. This is a known limitation, not an assumption violation.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Models, endpoints, CLI commands, TUI screen — but content rendering may be scoped down |
| Ambiguity | 3 | HIGH — API coverage uncertain, content format unknown, endpoint existence unconfirmed |
| Dependencies | 1 | Standard crate boundaries; reuses existing markdown rendering in TUI |
| Risk | 1 | Browser fallback ensures feature delivers value even if API is limited |

**Total: 7/12** → Standard processing with VALIDATE confidence (55%)

> **⚠️ VALIDATE CONFIDENCE:** This feature has 55% confidence due to API ambiguity. S-1 must complete API exploration before committing to S-2/S-3 scope. Be prepared to reduce scope to "list + browser open" if content retrieval is not possible.
