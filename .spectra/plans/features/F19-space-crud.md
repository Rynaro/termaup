# F19 — Space CRUD

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 5 — Workspace Management
> **Complexity:** 5/12 | **Confidence:** 90%

---

## Problem Statement

termaup currently supports read-only space browsing (`get_spaces()`, `get_space()`). Users who need to create, rename, or delete spaces must switch to the ClickUp web app. While space management is an infrequent operation (typically done during project setup), CLI users managing multiple workspaces or automating workspace configuration need these capabilities.

The ClickUp API v2 provides full CRUD for spaces: `POST /team/{team_id}/space`, `PUT /space/{space_id}`, and `DELETE /space/{space_id}`.

## Approach

1. **API layer:** Add three new endpoint methods (`create_space()`, `update_space()`, `delete_space()`) plus corresponding request model structs. Keep the models minimal — ClickUp's `POST /team/{id}/space` accepts `name`, `multiple_assignees`, `features` (with sub-objects for due dates, time tracking, tags, etc.), but only `name` is required for MVP.

2. **CLI layer:** Add `create`, `update`, and `delete` subcommands to the existing `clickup space` command group. Delete includes `--yes` flag to skip confirmation prompt.

3. **TUI:** Not included. Space creation/deletion is a rare administrative operation best done from CLI. TUI remains read-only at the workspace management level.

### Rejected Alternatives

1. **Full features object in CreateSpaceRequest** — ClickUp's space features config is complex (due_dates, time_tracking, tags, custom_fields, etc.). Including all options in the create request adds significant CLI flag complexity for rarely-used options. MVP uses name only; features can be added later. Rejected for scope.

2. **TUI space creation dialog** — Adding a creation dialog at the SpaceList level is possible but space creation is rare (1-2 per project). CLI is sufficient. Rejected for ROI.

3. **Batch space operations** — Creating multiple spaces at once from a YAML/JSON config file. Over-engineered for the use case. Rejected for YAGNI.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `create_space()` API endpoint | TUI space creation/editing | Space features configuration (due dates, time tracking, etc.) |
| `update_space()` API endpoint | Space member management | Space archival |
| `delete_space()` API endpoint | Space template application | Batch space operations |
| `CreateSpaceRequest` and `UpdateSpaceRequest` models | Space cloning | |
| CLI `clickup space create/update/delete` commands | | |
| Confirmation prompt for delete (unless `--yes`) | | |
| wiremock tests for all endpoints | | |

## Stories

### S-1: API Endpoints and Models

**As a** developer using `clickup-api`, **I want** `create_space()`, `update_space()`, and `delete_space()` methods on `ClickUpClient`, **so that** CLI can manage spaces programmatically.

**Timebox:** ≤2d | **Risk:** Low — standard CRUD pattern | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create `CreateSpaceRequest` struct | `crates/clickup-api/src/models/space.rs` | `pub struct CreateSpaceRequest { pub name: String, #[serde(skip_serializing_if = "Option::is_none")] pub multiple_assignees: Option<bool> }` — derive `Debug, Clone, Serialize, Deserialize` |
| 2 | Create `UpdateSpaceRequest` struct | `crates/clickup-api/src/models/space.rs` | `pub struct UpdateSpaceRequest { pub name: String }` |
| 3 | Add `create_space()` | `crates/clickup-api/src/endpoints/spaces.rs` | `pub async fn create_space(&self, team_id: &str, request: &CreateSpaceRequest) -> Result<Space>` — `POST /team/{team_id}/space` |
| 4 | Add `update_space()` | `crates/clickup-api/src/endpoints/spaces.rs` | `pub async fn update_space(&self, space_id: &str, request: &UpdateSpaceRequest) -> Result<Space>` — `PUT /space/{space_id}` |
| 5 | Add `delete_space()` | `crates/clickup-api/src/endpoints/spaces.rs` | `pub async fn delete_space(&self, space_id: &str) -> Result<()>` — `DELETE /space/{space_id}` |
| 6 | Add wiremock tests | `crates/clickup-api/tests/space_crud_tests.rs` | Test create (verify JSON body), update (verify path + body), delete (verify path), 404 error, 401 error |
| 7 | Add fixtures | `crates/clickup-api/tests/fixtures/create_space_response.json` | Realistic Space response after creation |

#### Acceptance Criteria

- **GIVEN** a valid team ID and `CreateSpaceRequest { name: "New Space" }`, **WHEN** `create_space()` is called, **THEN** a `POST /team/{team_id}/space` is sent with `{"name": "New Space"}` body, and the returned `Space` has the created name.
- **GIVEN** a valid space ID and `UpdateSpaceRequest { name: "Renamed" }`, **WHEN** `update_space()` is called, **THEN** a `PUT /space/{space_id}` is sent with `{"name": "Renamed"}` body.
- **GIVEN** a valid space ID, **WHEN** `delete_space()` is called, **THEN** a `DELETE /space/{space_id}` is sent and `Result<()>` is returned.
- **GIVEN** a non-existent space ID, **WHEN** `delete_space()` is called, **THEN** `ClickUpError::NotFound` is returned.
- **GIVEN** an invalid token, **WHEN** any CRUD method is called, **THEN** `ClickUpError::AuthError` is returned.

#### Agent Hints

- **Class:** builder
- **Skill:** `api-endpoints`
- **Context:** `crates/clickup-api/src/endpoints/spaces.rs` (existing `get_spaces` line 7, `get_space` line 14), `crates/clickup-api/src/models/space.rs` (existing `Space` struct line 8), `crates/clickup-api/src/client.rs` (`post` line 87, `put` line 103, `delete` line 132)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: wiremock tests for create, update, delete + error cases
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on all new public items

---

### S-2: CLI Space Commands

**As a** CLI user, **I want** `clickup space create`, `clickup space update`, and `clickup space delete` commands, **so that** I can manage spaces from the terminal.

**Timebox:** ≤2d | **Risk:** Low — follows existing CLI patterns | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add `Create` variant to `SpaceCommands` | `crates/clickup-cli/src/commands/spaces.rs` | `Create { #[arg(long)] name: String, #[arg(long)] workspace: Option<String> }` |
| 2 | Add `Update` variant to `SpaceCommands` | `crates/clickup-cli/src/commands/spaces.rs` | `Update { space_id: String, #[arg(long)] name: String }` |
| 3 | Add `Delete` variant to `SpaceCommands` | `crates/clickup-cli/src/commands/spaces.rs` | `Delete { space_id: String, #[arg(long)] yes: bool }` |
| 4 | Implement `create` handler | `crates/clickup-cli/src/commands/spaces.rs` | Build `CreateSpaceRequest`, auto-detect workspace (or use `--workspace`), call `create_space()`, print `"✓ Created space: {name} ({id})"` |
| 5 | Implement `update` handler | `crates/clickup-cli/src/commands/spaces.rs` | Build `UpdateSpaceRequest`, call `update_space()`, print `"✓ Updated space: {name}"` |
| 6 | Implement `delete` handler | `crates/clickup-cli/src/commands/spaces.rs` | If `--yes` not set, prompt with `dialoguer::Confirm`: `"Delete space '{name}'? This cannot be undone."`. Call `delete_space()`, print `"✓ Deleted space: {id}"` |
| 7 | Auto-detect workspace for create | `crates/clickup-cli/src/commands/spaces.rs` | If `--workspace` not provided, call `get_workspaces()`. If single workspace, use it. If multiple, prompt with `dialoguer::Select`. |

#### Acceptance Criteria

- **GIVEN** a workspace with ID `team123`, **WHEN** running `clickup space create --name "New Space" --workspace team123`, **THEN** the space is created and `"✓ Created space: New Space ({id})"` is printed.
- **GIVEN** a space with ID `space456`, **WHEN** running `clickup space update space456 --name "Renamed Space"`, **THEN** the space is renamed and `"✓ Updated space: Renamed Space"` is printed.
- **GIVEN** a space with ID `space456`, **WHEN** running `clickup space delete space456`, **THEN** a confirmation prompt appears. If confirmed, the space is deleted and `"✓ Deleted space: space456"` is printed.
- **GIVEN** `--yes` flag, **WHEN** running `clickup space delete space456 --yes`, **THEN** no confirmation prompt appears and the space is deleted immediately.
- **GIVEN** no `--workspace` flag and only one workspace exists, **WHEN** running `clickup space create --name "Test"`, **THEN** the single workspace is auto-selected.

#### Agent Hints

- **Class:** builder
- **Skill:** `cli-data`
- **Context:** `crates/clickup-cli/src/commands/spaces.rs` (existing SpaceCommands), `crates/clickup-cli/src/commands/tasks.rs` (handler pattern reference)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo fmt` + `cargo clippy` clean
  - [ ] P2: `///` doc comments on new public items

---

### S-3: Space CRUD Tests

**As a** maintainer, **I want** comprehensive tests for space CRUD operations, **so that** regressions are caught.

**Timebox:** ≤1d | **Risk:** Low | **Depends on:** S-1, S-2

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | wiremock endpoint tests | `crates/clickup-api/tests/space_crud_tests.rs` | Full create/update/delete cycle against mock server |
| 2 | Model serialization tests | `crates/clickup-api/src/models/space.rs` | Verify `CreateSpaceRequest` serializes correctly, `skip_serializing_if` works for `None` fields |
| 3 | Model deserialization test | `crates/clickup-api/src/models/space.rs` | Verify `Space` deserializes from create/update response fixtures |
| 4 | Error handling tests | `crates/clickup-api/tests/space_crud_tests.rs` | Test 401, 404, 429 responses map to correct `ClickUpError` variants |

#### Acceptance Criteria

- **GIVEN** wiremock server, **WHEN** `create_space()` is called, **THEN** the mock receives `POST /api/v2/team/{id}/space` with correct JSON body.
- **GIVEN** `CreateSpaceRequest { name: "Test", multiple_assignees: None }`, **WHEN** serialized, **THEN** the JSON does not contain `multiple_assignees` key.
- **GIVEN** API returns 429, **WHEN** any CRUD method is called, **THEN** `ClickUpError::RateLimited` is returned.

#### Agent Hints

- **Class:** builder
- **Skill:** `docs-release`
- **Context:** `crates/clickup-api/tests/` (existing test patterns)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P2: `cargo clippy` clean

---

## Execution Sequence

```
S-1 (API endpoints + models) ──→ S-2 (CLI commands) ──→ S-3 (tests)
```

Linear dependency chain. S-2 depends on S-1 for the API methods. S-3 depends on both for comprehensive coverage.

## Assumptions

1. The ClickUp API `POST /team/{team_id}/space` only requires `name` in the request body; all other fields (features, multiple_assignees) are optional. **Risk if wrong:** API docs confirm `name` is the only required field; low risk.
2. `DELETE /space/{space_id}` returns an empty response body on success (HTTP 200 with no content). **Risk if wrong:** Client's `delete()` method already handles empty responses via `handle_response_no_body()`.
3. The `PUT /space/{space_id}` response returns the updated `Space` object. **Risk if wrong:** If it returns empty, change return type to `Result<()>`.
4. Space deletion is irreversible and deletes all contained folders, lists, and tasks. CLI confirmation prompt is essential. **Risk if wrong:** None — this is a ClickUp API behavior, not our concern; we just need to warn the user.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Two crates (API + CLI), new models + endpoints + commands |
| Ambiguity | 1 | Clear API documentation, standard CRUD pattern |
| Dependencies | 1 | Builds on existing client infrastructure, no cross-feature deps |
| Risk | 1 | Standard REST CRUD, well-understood pattern |

**Total: 5/12** → Standard processing
