# F20 — Folder CRUD

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 5 — Workspace Management
> **Complexity:** 5/12 | **Confidence:** 89%

---

## Problem Statement

termaup currently supports read-only folder browsing via `get_folders()` in the API crate. The TUI displays folders as expandable tree nodes on the SpaceContent screen, and the CLI has no folder-specific commands at all. Users who need to create organizational structure (new folders for sprints, projects, or teams) or clean up unused folders must switch to the ClickUp web app.

The ClickUp API v2 provides `POST /space/{space_id}/folder`, `PUT /folder/{folder_id}`, and `DELETE /folder/{folder_id}`.

## Approach

1. **API layer:** Add `create_folder()`, `update_folder()`, and `delete_folder()` endpoint methods plus `CreateFolderRequest` and `UpdateFolderRequest` model structs.

2. **CLI layer:** Add a new `folder` command group (`clickup folder create`, `clickup folder update`, `clickup folder delete`). This is a new top-level command since no `folder` command exists yet.

3. **TUI layer:** On the SpaceContent screen, add `N` to create a new folder (name input dialog) and `D` to delete the selected folder (with confirmation). Folder renaming is deferred (less common operation).

### Rejected Alternatives

1. **Nest folder commands under `clickup space`** — `clickup space folder create` is too deeply nested. A top-level `clickup folder` is more ergonomic and consistent with the `clickup list` pattern. Rejected for UX.

2. **Full folder editing in TUI (rename + move)** — Moving folders between spaces is rare and complex. Rename can be added later. MVP covers create and delete. Rejected for scope.

3. **Folder templates** — Pre-configured folder structures (e.g., "Sprint Folder" with standard lists). Over-engineered. Rejected for YAGNI.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `create_folder()` API endpoint | Moving folders between spaces | TUI folder rename |
| `update_folder()` API endpoint | Folder visibility/hidden management | Folder archival |
| `delete_folder()` API endpoint | Folder duplication | Folder ordering |
| `CreateFolderRequest`, `UpdateFolderRequest` models | Nested folder creation (batch) | |
| CLI `clickup folder create/update/delete` | | |
| TUI `N` to create folder on SpaceContent | | |
| TUI `D` to delete folder on SpaceContent (with confirmation) | | |

## Stories

### S-1: API Endpoints and Models

**As a** developer using `clickup-api`, **I want** `create_folder()`, `update_folder()`, and `delete_folder()` methods, **so that** CLI and TUI can manage folders programmatically.

**Timebox:** ≤2d | **Risk:** Low — standard CRUD, mirrors space CRUD pattern | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create `CreateFolderRequest` struct | `crates/clickup-api/src/models/folder.rs` | `pub struct CreateFolderRequest { pub name: String }` — derive `Debug, Clone, Serialize, Deserialize` |
| 2 | Create `UpdateFolderRequest` struct | `crates/clickup-api/src/models/folder.rs` | `pub struct UpdateFolderRequest { pub name: String }` |
| 3 | Add `create_folder()` | `crates/clickup-api/src/endpoints/folders.rs` | `pub async fn create_folder(&self, space_id: &str, request: &CreateFolderRequest) -> Result<Folder>` — `POST /space/{space_id}/folder` |
| 4 | Add `update_folder()` | `crates/clickup-api/src/endpoints/folders.rs` | `pub async fn update_folder(&self, folder_id: &str, request: &UpdateFolderRequest) -> Result<Folder>` — `PUT /folder/{folder_id}` |
| 5 | Add `delete_folder()` | `crates/clickup-api/src/endpoints/folders.rs` | `pub async fn delete_folder(&self, folder_id: &str) -> Result<()>` — `DELETE /folder/{folder_id}` |
| 6 | Add wiremock tests | `crates/clickup-api/tests/folder_crud_tests.rs` | Test create (verify path and JSON body), update, delete, 404, 401 |
| 7 | Add fixture | `crates/clickup-api/tests/fixtures/create_folder_response.json` | Realistic Folder response |

#### Acceptance Criteria

- **GIVEN** a valid space ID and `CreateFolderRequest { name: "Sprint 42" }`, **WHEN** `create_folder()` is called, **THEN** `POST /space/{space_id}/folder` is sent with `{"name": "Sprint 42"}` body, and a `Folder` with the new name is returned.
- **GIVEN** a valid folder ID, **WHEN** `update_folder()` is called with `{ name: "Renamed" }`, **THEN** `PUT /folder/{folder_id}` is sent.
- **GIVEN** a valid folder ID, **WHEN** `delete_folder()` is called, **THEN** `DELETE /folder/{folder_id}` is sent and `Result<()>` is returned.
- **GIVEN** a non-existent folder ID, **WHEN** `delete_folder()` is called, **THEN** `ClickUpError::NotFound` is returned.

#### Agent Hints

- **Class:** builder
- **Skill:** `api-endpoints`
- **Context:** `crates/clickup-api/src/endpoints/folders.rs` (existing `get_folders` line 7), `crates/clickup-api/src/models/folder.rs` (existing `Folder` struct line 10), `crates/clickup-api/src/client.rs` (`post` line 87, `put` line 103, `delete` line 132)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: wiremock tests for all three operations + error cases
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on all new public items

---

### S-2: CLI Folder Commands

**As a** CLI user, **I want** `clickup folder create`, `clickup folder update`, and `clickup folder delete` commands, **so that** I can manage folders from the terminal.

**Timebox:** ≤2d | **Risk:** Low — new command group following existing patterns | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create `crates/clickup-cli/src/commands/folders.rs` | `crates/clickup-cli/src/commands/folders.rs` | New file with `FolderCommands` enum: `Create { #[arg(long)] space: String, #[arg(long)] name: String }`, `Update { folder_id: String, #[arg(long)] name: String }`, `Delete { folder_id: String, #[arg(long)] yes: bool }` |
| 2 | Register module | `crates/clickup-cli/src/commands/mod.rs` | Add `pub mod folders;` |
| 3 | Add `Folder` variant to top-level `Commands` | `crates/clickup-cli/src/main.rs` | `Folder { #[command(subcommand)] command: FolderCommands }` |
| 4 | Add match arm | `crates/clickup-cli/src/main.rs` | Route `Commands::Folder { command }` to handler |
| 5 | Implement create handler | `crates/clickup-cli/src/commands/folders.rs` | Build `CreateFolderRequest`, call `create_folder()`, print `"✓ Created folder: {name} ({id})"` |
| 6 | Implement update handler | `crates/clickup-cli/src/commands/folders.rs` | Call `update_folder()`, print `"✓ Updated folder: {name}"` |
| 7 | Implement delete handler | `crates/clickup-cli/src/commands/folders.rs` | Confirmation prompt (unless `--yes`), call `delete_folder()`, print `"✓ Deleted folder: {id}"` |

#### Acceptance Criteria

- **GIVEN** a space ID, **WHEN** running `clickup folder create --space SPACE_ID --name "Sprint 42"`, **THEN** the folder is created and `"✓ Created folder: Sprint 42 ({id})"` is printed.
- **GIVEN** a folder ID, **WHEN** running `clickup folder update FOLDER_ID --name "Sprint 43"`, **THEN** the folder is renamed.
- **GIVEN** a folder ID, **WHEN** running `clickup folder delete FOLDER_ID`, **THEN** a confirmation prompt appears. On `y`, the folder is deleted.
- **GIVEN** `--yes` flag, **WHEN** running `clickup folder delete FOLDER_ID --yes`, **THEN** the folder is deleted without prompting.
- **WHEN** running `clickup folder --help`, **THEN** subcommands `create`, `update`, `delete` are listed.

#### Agent Hints

- **Class:** builder
- **Skill:** `cli-data`
- **Context:** `crates/clickup-cli/src/commands/mod.rs` (module registry lines 1–6), `crates/clickup-cli/src/main.rs` (Commands enum lines 18–58), `crates/clickup-cli/src/commands/spaces.rs` (pattern reference)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on `FolderCommands`

---

### S-3: TUI Folder Create and Delete Actions

**As a** TUI user on the SpaceContent screen, **I want** to press `N` to create a new folder and `D` to delete the selected folder, **so that** I can organize my space without leaving the TUI.

**Timebox:** ≤2d | **Risk:** Medium — new input dialog pattern in TUI | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add folder dialog state | `crates/clickup-tui/src/app.rs` | `folder_create_dialog_open: bool`, `folder_create_input: String`, `folder_delete_confirm: Option<String>` (folder ID pending confirmation) |
| 2 | Add `N` key handler on SpaceContent | `crates/clickup-tui/src/input.rs` | Open folder create dialog: set `folder_create_dialog_open = true`, clear input |
| 3 | Handle create dialog input | `crates/clickup-tui/src/input.rs` | `Char(c)` appends to `folder_create_input`, `Backspace` removes, `Enter` triggers create, `Esc` cancels |
| 4 | Create `spawn_create_folder()` | `crates/clickup-tui/src/data.rs` | Calls `client.create_folder(space_id, &request)`, sends `DataPayload::FolderCreated(folder)` |
| 5 | Add `DataPayload::FolderCreated` variant | `crates/clickup-tui/src/app.rs` | On receipt: add folder to `folders` list, rebuild `space_content`, show flash `"✓ Created folder: {name}"` |
| 6 | Add `D` key handler on SpaceContent | `crates/clickup-tui/src/input.rs` | When selected item is a folder: set `folder_delete_confirm = Some(folder_id)` |
| 7 | Handle delete confirmation | `crates/clickup-tui/src/input.rs` | `y`/`Y` confirms: call `spawn_delete_folder()`. `n`/`Esc` cancels. |
| 8 | Create `spawn_delete_folder()` | `crates/clickup-tui/src/data.rs` | Calls `client.delete_folder(folder_id)`, sends `DataPayload::FolderDeleted(folder_id)` |
| 9 | Add `DataPayload::FolderDeleted` variant | `crates/clickup-tui/src/app.rs` | On receipt: remove folder from `folders` list, rebuild `space_content`, show flash `"✓ Deleted folder"` |
| 10 | Render create dialog | `crates/clickup-tui/src/ui/mod.rs` | Small popup: "New Folder", text input, Enter to create / Esc to cancel |
| 11 | Render delete confirmation | `crates/clickup-tui/src/ui/mod.rs` | Bottom bar: `"Delete folder '{name}'? (y/n)"` |
| 12 | Update key hints | `crates/clickup-tui/src/ui/mod.rs` | Add `N New folder` and `D Delete` to SpaceContent key hints |

#### Acceptance Criteria

- **GIVEN** the user is on SpaceContent, **WHEN** they press `N`, **THEN** a "New Folder" dialog appears with a text input.
- **GIVEN** the dialog is open and the user types "Sprint 42" and presses Enter, **WHEN** the API succeeds, **THEN** the folder appears in the SpaceContent list and a flash message confirms creation.
- **GIVEN** the user selects a folder and presses `D`, **WHEN** the confirmation prompt appears, **THEN** pressing `y` deletes the folder and pressing `n` cancels.
- **GIVEN** the selected item is a list (not a folder), **WHEN** `D` is pressed, **THEN** nothing happens (D only applies to folders on this screen; lists use `d`).
- **GIVEN** the user is in filter/search mode, **WHEN** `N` or `D` is pressed, **THEN** the keys are treated as filter text input, not folder actions.

#### Agent Hints

- **Class:** builder
- **Skill:** `tui-workspaces`
- **Context:** `crates/clickup-tui/src/input.rs` (SpaceContent key handling lines 267–326), `crates/clickup-tui/src/app.rs` (SpaceContentItem, space_content field), `crates/clickup-tui/src/data.rs`
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new public items

---

## Execution Sequence

```
S-1 (API endpoints + models) ──┬──→ S-2 (CLI commands)
                                └──→ S-3 (TUI create/delete)
```

S-2 and S-3 are independent of each other, both depend on S-1.

## Assumptions

1. `POST /space/{space_id}/folder` only requires `name` in the body. **Risk if wrong:** API docs confirm; low risk.
2. `DELETE /folder/{folder_id}` cascades to all contained lists and tasks. CLI/TUI must warn the user. **Risk if wrong:** This is documented ClickUp behavior.
3. The `Folder` model returned from create/update operations matches the existing `Folder` struct shape. **Risk if wrong:** Defensive deserialization handles unknown fields.
4. SpaceContent's `space_content: Vec<SpaceContentItem>` can be rebuilt after folder create/delete by re-merging `folders` and `lists`. **Risk if wrong:** Need to understand the `SpaceContentItem` enum to rebuild correctly.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Three crates (API, CLI, TUI) |
| Ambiguity | 1 | Clear CRUD pattern, API well-documented |
| Dependencies | 1 | No cross-feature dependencies |
| Risk | 1 | Standard CRUD; TUI dialog is new but simple pattern |

**Total: 5/12** → Standard processing
