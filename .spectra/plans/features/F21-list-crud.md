# F21 — List CRUD

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 5 — Workspace Management
> **Complexity:** 5/12 | **Confidence:** 88%

---

## Problem Statement

termaup currently supports read-only list browsing via `get_lists_in_folder()`, `get_folderless_lists()`, and `get_list()`. Users cannot create new lists, rename existing ones, or delete unused lists without switching to the ClickUp web app. This is a more frequent operation than space or folder management — users regularly create new lists for sprints, feature tracks, or ad-hoc task collections.

The ClickUp API v2 provides four relevant endpoints:
- `POST /folder/{folder_id}/list` — create a list inside a folder
- `POST /space/{space_id}/list` — create a folderless list in a space
- `PUT /list/{list_id}` — update list properties (name, content, etc.)
- `DELETE /list/{list_id}` — delete a list

## Approach

1. **API layer:** Add four endpoint methods: `create_list_in_folder()`, `create_folderless_list()`, `update_list()`, `delete_list()`. Two create methods are needed because ClickUp distinguishes between folder lists and folderless (space-level) lists with different API paths.

2. **CLI layer:** Add `create`, `update`, and `delete` subcommands to the existing `clickup list` command group. The create command accepts either `--folder` or `--space` to determine the parent.

3. **TUI layer:** On the SpaceContent screen, add `n` to create a new list (name input + parent selection: folder or folderless) and `d` to delete the selected list (with confirmation). Consistent with F20's `N`/`D` for folders — lowercase for lists, uppercase for folders.

### Rejected Alternatives

1. **Single `create_list()` method with parent enum** — Combining folder and folderless creation into one method with a `ListParent::Folder(id) | ListParent::Space(id)` enum. Adds abstraction without simplifying the API calls (different URLs). Rejected for over-abstraction.

2. **List move between folders** — `PUT /list/{id}` can't move lists between folders; requires delete + recreate. Complex and lossy. Rejected for MVP scope.

3. **List settings management** — Due dates, time tracking, assignees settings per list. Out of scope for CRUD MVP. Rejected for scope.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `create_list_in_folder()` API endpoint | List status management (active/archived) | List reordering |
| `create_folderless_list()` API endpoint | List-level settings (due dates, time tracking) | List content/description editing |
| `update_list()` API endpoint (name only for MVP) | Moving lists between folders | TUI list rename dialog |
| `delete_list()` API endpoint | List duplication | List templates |
| CLI `clickup list create/update/delete` | | |
| TUI `n` to create list on SpaceContent | | |
| TUI `d` to delete list on SpaceContent | | |
| Parent selection (folder vs. folderless) in CLI and TUI | | |

## Stories

### S-1: API Endpoints and Models

**As a** developer using `clickup-api`, **I want** list CRUD methods on `ClickUpClient`, **so that** CLI and TUI can create, rename, and delete lists.

**Timebox:** ≤2d | **Risk:** Low — mirrors folder CRUD pattern | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create `CreateListRequest` struct | `crates/clickup-api/src/models/list.rs` | `pub struct CreateListRequest { pub name: String, #[serde(skip_serializing_if = "Option::is_none")] pub content: Option<String>, #[serde(skip_serializing_if = "Option::is_none")] pub status: Option<String> }` |
| 2 | Create `UpdateListRequest` struct | `crates/clickup-api/src/models/list.rs` | `pub struct UpdateListRequest { pub name: String }` |
| 3 | Add `create_list_in_folder()` | `crates/clickup-api/src/endpoints/lists.rs` | `pub async fn create_list_in_folder(&self, folder_id: &str, request: &CreateListRequest) -> Result<List>` — `POST /folder/{folder_id}/list` |
| 4 | Add `create_folderless_list()` | `crates/clickup-api/src/endpoints/lists.rs` | `pub async fn create_folderless_list(&self, space_id: &str, request: &CreateListRequest) -> Result<List>` — `POST /space/{space_id}/list` |
| 5 | Add `update_list()` | `crates/clickup-api/src/endpoints/lists.rs` | `pub async fn update_list(&self, list_id: &str, request: &UpdateListRequest) -> Result<List>` — `PUT /list/{list_id}` |
| 6 | Add `delete_list()` | `crates/clickup-api/src/endpoints/lists.rs` | `pub async fn delete_list(&self, list_id: &str) -> Result<()>` — `DELETE /list/{list_id}` |
| 7 | Add wiremock tests | `crates/clickup-api/tests/list_crud_tests.rs` | Test all four endpoints: verify URL paths, request bodies, response deserialization, error cases (401, 404) |
| 8 | Add fixtures | `crates/clickup-api/tests/fixtures/create_list_response.json` | Realistic List response after creation |

#### Acceptance Criteria

- **GIVEN** a folder ID and `CreateListRequest { name: "Backlog" }`, **WHEN** `create_list_in_folder()` is called, **THEN** `POST /folder/{folder_id}/list` is sent with `{"name": "Backlog"}`.
- **GIVEN** a space ID and `CreateListRequest { name: "Ad-hoc" }`, **WHEN** `create_folderless_list()` is called, **THEN** `POST /space/{space_id}/list` is sent with `{"name": "Ad-hoc"}`.
- **GIVEN** a list ID, **WHEN** `update_list()` is called with `{ name: "Renamed" }`, **THEN** `PUT /list/{list_id}` is sent.
- **GIVEN** a list ID, **WHEN** `delete_list()` is called, **THEN** `DELETE /list/{list_id}` is sent.
- **GIVEN** a non-existent list ID, **WHEN** `delete_list()` is called, **THEN** `ClickUpError::NotFound` is returned.
- **GIVEN** `CreateListRequest` with `content: None, status: None`, **WHEN** serialized, **THEN** only `name` appears in the JSON body (skip_serializing_if works).

#### Agent Hints

- **Class:** builder
- **Skill:** `api-endpoints`
- **Context:** `crates/clickup-api/src/endpoints/lists.rs` (existing methods lines 7–23), `crates/clickup-api/src/models/list.rs` (existing `List` struct line 10)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: wiremock tests for all four operations
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on all new public items

---

### S-2: CLI List Commands

**As a** CLI user, **I want** `clickup list create`, `clickup list update`, and `clickup list delete` commands, **so that** I can manage lists from the terminal.

**Timebox:** ≤2d | **Risk:** Low — extends existing `list` command group | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `Create` variant to `ListCommands` | `crates/clickup-cli/src/commands/lists.rs` | `Create { #[arg(long)] name: String, #[arg(long, group = "parent")] folder: Option<String>, #[arg(long, group = "parent")] space: Option<String> }` — `folder` and `space` are mutually exclusive via clap groups |
| 2 | Add `Update` variant to `ListCommands` | `crates/clickup-cli/src/commands/lists.rs` | `Update { list_id: String, #[arg(long)] name: String }` |
| 3 | Add `Delete` variant to `ListCommands` | `crates/clickup-cli/src/commands/lists.rs` | `Delete { list_id: String, #[arg(long)] yes: bool }` |
| 4 | Implement create handler | `crates/clickup-cli/src/commands/lists.rs` | If `--folder` provided: `create_list_in_folder()`. If `--space` provided: `create_folderless_list()`. If neither: error `"✗ Specify --folder or --space"`. Print `"✓ Created list: {name} ({id})"` |
| 5 | Implement update handler | `crates/clickup-cli/src/commands/lists.rs` | Call `update_list()`, print `"✓ Updated list: {name}"` |
| 6 | Implement delete handler | `crates/clickup-cli/src/commands/lists.rs` | Confirmation prompt (unless `--yes`): `"Delete list '{name}'? All tasks will be deleted."`. Call `delete_list()`, print `"✓ Deleted list: {id}"` |

#### Acceptance Criteria

- **GIVEN** `--folder FOLDER_ID`, **WHEN** running `clickup list create --name "Backlog" --folder FOLDER_ID`, **THEN** a list is created in the folder.
- **GIVEN** `--space SPACE_ID`, **WHEN** running `clickup list create --name "Ad-hoc" --space SPACE_ID`, **THEN** a folderless list is created in the space.
- **GIVEN** both `--folder` and `--space` provided, **WHEN** the command is parsed, **THEN** clap reports a conflict error (mutually exclusive args).
- **GIVEN** neither `--folder` nor `--space`, **WHEN** the command runs, **THEN** `"✗ Specify --folder or --space"` is printed.
- **GIVEN** a list ID, **WHEN** running `clickup list delete LIST_ID --yes`, **THEN** the list is deleted without confirmation.

#### Agent Hints

- **Class:** builder
- **Skill:** `cli-data`
- **Context:** `crates/clickup-cli/src/commands/lists.rs` (existing ListCommands), `crates/clickup-cli/src/main.rs` (command routing)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new variants

---

### S-3: TUI List Create and Delete Actions

**As a** TUI user on the SpaceContent screen, **I want** to press `n` to create a new list and `d` to delete the selected list, **so that** I can manage lists without leaving the TUI.

**Timebox:** ≤2d | **Risk:** Medium — parent selection (folder vs. folderless) adds dialog complexity | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add list dialog state | `crates/clickup-tui/src/app.rs` | `list_create_dialog_open: bool`, `list_create_input: String`, `list_create_parent: ListParent` (enum: `Folder(String)`, `Space(String)`), `list_delete_confirm: Option<String>` |
| 2 | Add `ListParent` enum | `crates/clickup-tui/src/app.rs` | `pub enum ListParent { Folder(String), Space(String) }` |
| 3 | Add `n` key handler on SpaceContent | `crates/clickup-tui/src/input.rs` | Open list create dialog. Determine parent: if selected item is inside a folder, default to `ListParent::Folder(folder_id)`. If at space level, default to `ListParent::Space(space_id)`. Show dialog with name input. |
| 4 | Handle create dialog input | `crates/clickup-tui/src/input.rs` | Text input for name. Enter triggers create based on `list_create_parent`. Esc cancels. |
| 5 | Create `spawn_create_list()` | `crates/clickup-tui/src/data.rs` | Based on `ListParent`: calls `create_list_in_folder()` or `create_folderless_list()`. Sends `DataPayload::ListCreated(list)`. |
| 6 | Add `DataPayload::ListCreated` variant | `crates/clickup-tui/src/app.rs` | On receipt: add list to appropriate section, rebuild `space_content`, show flash `"✓ Created list: {name}"` |
| 7 | Add `d` key handler on SpaceContent | `crates/clickup-tui/src/input.rs` | When selected item is a list: set `list_delete_confirm = Some(list_id)` |
| 8 | Handle delete confirmation | `crates/clickup-tui/src/input.rs` | `y`/`Y` confirms: call `spawn_delete_list()`. `n`/`Esc` cancels. |
| 9 | Create `spawn_delete_list()` | `crates/clickup-tui/src/data.rs` | Calls `client.delete_list(list_id)`, sends `DataPayload::ListDeleted(list_id)` |
| 10 | Add `DataPayload::ListDeleted` variant | `crates/clickup-tui/src/app.rs` | On receipt: remove list, rebuild `space_content`, show flash `"✓ Deleted list"` |
| 11 | Render list create dialog | `crates/clickup-tui/src/ui/mod.rs` | Small popup: "New List in {parent_name}", text input |
| 12 | Render delete confirmation bar | `crates/clickup-tui/src/ui/mod.rs` | Bottom bar: `"Delete list '{name}'? All tasks will be deleted. (y/n)"` |
| 13 | Update key hints | `crates/clickup-tui/src/ui/mod.rs` | Add `n New list` and `d Delete list` to SpaceContent key hints |

#### Acceptance Criteria

- **GIVEN** the user is on SpaceContent with a folder selected, **WHEN** `n` is pressed, **THEN** a "New List in {folder_name}" dialog appears.
- **GIVEN** the dialog is open and the user types "Backlog" and presses Enter, **WHEN** the API succeeds, **THEN** the list appears under the selected folder and a flash confirms creation.
- **GIVEN** the user is at space level (no folder selected), **WHEN** `n` is pressed, **THEN** a "New Folderless List" dialog appears and creates via `create_folderless_list()`.
- **GIVEN** the user selects a list and presses `d`, **WHEN** the confirmation shows, **THEN** `y` deletes and `n` cancels.
- **GIVEN** the selected item is a folder (not a list), **WHEN** `d` is pressed, **THEN** nothing happens (lowercase `d` is for lists; uppercase `D` is for folders per F20).

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-workspaces`
- **Context:** `crates/clickup-tui/src/input.rs` (SpaceContent handling lines 267–326), `crates/clickup-tui/src/app.rs` (`SpaceContentItem` enum, `space_content` field), `crates/clickup-tui/src/data.rs`
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on `ListParent`, `spawn_create_list()`, `spawn_delete_list()`

---

## Execution Sequence

```
S-1 (API endpoints + models) ──┬──→ S-2 (CLI commands)
                                └──→ S-3 (TUI create/delete)
```

S-2 and S-3 are independent, both depend on S-1.

## Assumptions

1. `POST /folder/{id}/list` and `POST /space/{id}/list` share the same request body format (only `name` required). **Risk if wrong:** API docs confirm; low risk.
2. `PUT /list/{id}` returns the updated `List` object. **Risk if wrong:** Change return type to `Result<()>` if empty response.
3. `DELETE /list/{id}` cascades to all tasks in the list. The confirmation prompt must communicate this. **Risk if wrong:** This is documented ClickUp behavior.
4. The SpaceContent screen distinguishes between folder items and list items in its `SpaceContentItem` enum, allowing targeted `d` (lists) vs `D` (folders) keybindings. **Risk if wrong:** Need to verify the enum variant structure in `app.rs`.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Three crates (API, CLI, TUI) with 4 API endpoints |
| Ambiguity | 1 | Clear CRUD pattern, two parent types are straightforward |
| Dependencies | 1 | No cross-feature dependencies |
| Risk | 1 | Standard CRUD; parent selection logic is the only nuance |

**Total: 5/12** → Standard processing
