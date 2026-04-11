# F09 — Custom Field Editing

> **SPECTRA v4.2.0** — Feature plan
> **Date:** 2026-04-11
> **Phase:** 3 — Task Enrichment
> **Complexity:** 8/12 | **Confidence:** 75%

---

## Problem Statement

ClickUp custom fields are the primary way teams extend task metadata beyond the built-in fields. termaup can display custom field values (via `CustomField::display_value()`) but cannot create, list definitions, or set values. The ClickUp API provides `GET /list/{list_id}/field` to fetch available field definitions and `POST /task/{task_id}/field/{field_id}` to set a field's value — but each field type (text, number, checkbox, date, dropdown, labels, emoji, url, email, phone, currency, location, users, tasks, manual_progress) requires a different JSON value format. This heterogeneity is the core challenge.

## Approach

Add two new API endpoints (`get_custom_fields()` and `set_custom_field_value()`), a `CustomFieldDefinition` model for list-level field schemas, and a `SetCustomFieldRequest` model with a generic `serde_json::Value` payload. The CLI provides `clickup field list --list LIST_ID` and `clickup field set --task TASK_ID --field FIELD_ID --value "..."`. The TUI presents a custom field editor accessible via `F` on TaskDetail — listing the task's fields with type-specific input handlers.

### Rejected Alternatives

1. **Typed `FieldValue` enum** — `enum FieldValue { Text(String), Number(f64), Checkbox(bool), ... }` with custom serialization per variant. Strongly typed but brittle: ClickUp adds new field types without notice (e.g., `ai_summary`, `relationships`), breaking the enum. The `serde_json::Value` approach is more resilient. Rejected for fragility.

2. **Single generic CLI flag** — `clickup field set --task T --field F --value V` for all types, parsing the value based on field type at the CLI layer. This is actually the approach we take, but we also considered requiring type-specific flags (`--text-value`, `--number-value`, etc.) which would be verbose and hard to discover. Rejected the verbose flag approach.

3. **Inline editing on TaskDetail** — Edit fields directly in the TaskDetail view. This conflicts with F06's edit mode pattern and would create UX confusion about which fields are editable inline vs via picker. Rejected; dedicated `F` key with a focused field editor is cleaner.

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| `GET /list/{list_id}/field` endpoint | Creating new custom field definitions | Custom field bulk editing |
| `POST /task/{task_id}/field/{field_id}` endpoint | Deleting custom field definitions | Custom field formula evaluation |
| `CustomFieldDefinition` model | `automatic_progress` field type (read-only) | Custom field filtering in task list |
| `SetCustomFieldRequest` model | Field-level permissions | Custom field validation rules |
| CLI `field list` and `field set` commands | Relationship field type (API v3) | |
| TUI field editor with type-specific inputs | | |
| Value format handling for: text, number, checkbox, date, dropdown, labels, email, url, phone, currency, emoji, users, tasks, manual_progress, location | | |

## Stories

### S-1: API Endpoints and Models

**As a** developer using clickup-api, **I want** `get_custom_fields()` and `set_custom_field_value()` methods on `ClickUpClient`, **so that** CLI and TUI can list available fields and set their values.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** —

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/src/models/custom_field.rs` | Add `CustomFieldDefinition` struct: `id: String`, `name: String`, `#[serde(rename = "type")] field_type: String`, `type_config: Option<serde_json::Value>`, `required: Option<bool>`, `date_created: Option<String>`. All with defensive serde (`#[serde(default)]` on optional fields). |
| 2 | Create | `crates/clickup-api/src/models/custom_field.rs` | Add `CustomFieldsResponse` wrapper: `fields: Vec<CustomFieldDefinition>` with `#[serde(default, deserialize_with = "deserialize_null_as_default")]`. |
| 3 | Create | `crates/clickup-api/src/models/custom_field.rs` | Add `SetCustomFieldRequest` struct: `value: serde_json::Value` (the raw JSON value). Different field types require different shapes — text/url/email/phone → string, number/currency → number, checkbox → boolean, date → string (Unix ms), dropdown → UUID string, labels → array of UUID strings, emoji → integer, users → array of user objects, manual_progress → object `{current: N}`, location → object `{location: {lat, lng}, formatted_address: "..."}`. |
| 4 | Modify | `crates/clickup-api/src/models/mod.rs` | Add `pub mod custom_field;` and re-export key types. |
| 5 | Create | `crates/clickup-api/src/endpoints/custom_fields.rs` | Add `get_custom_fields(&self, list_id: &str) -> Result<Vec<CustomFieldDefinition>>`: `GET /list/{list_id}/field`, log with `tracing::debug!`, deserialize `CustomFieldsResponse`. |
| 6 | Create | `crates/clickup-api/src/endpoints/custom_fields.rs` | Add `set_custom_field_value(&self, task_id: &str, field_id: &str, req: &SetCustomFieldRequest) -> Result<()>`: `POST /task/{task_id}/field/{field_id}`, log, use `put_no_body` pattern (POST returns empty body on success — may need a `post_no_response` helper or handle empty response). |
| 7 | Modify | `crates/clickup-api/src/endpoints/mod.rs` | Add `pub mod custom_fields;` declaration. |
| 8 | Test | `crates/clickup-api/tests/fixtures/custom_fields_list.json` | Create fixture: JSON response from `GET /list/{id}/field` with fields of types: text, number, dropdown (with options), labels (with options), checkbox, date, currency, emoji. |
| 9 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock test: mock `GET /list/123/field` → verify `get_custom_fields()` returns correct `Vec<CustomFieldDefinition>`. |
| 10 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock test: mock `POST /task/abc/field/cf1` → verify `set_custom_field_value()` sends correct body and handles empty success response. |

#### Acceptance Criteria

- [ ] GIVEN a list with custom fields, WHEN `client.get_custom_fields("list123")` is called, THEN it returns a `Vec<CustomFieldDefinition>` with id, name, field_type, and type_config populated.
- [ ] GIVEN a `CustomFieldDefinition` with `type_config` containing dropdown options, WHEN deserialized, THEN `type_config` is `Some(...)` with parseable options array.
- [ ] GIVEN a `SetCustomFieldRequest` with `value: json!("hello")`, WHEN serialized, THEN the body is `{"value":"hello"}`.
- [ ] GIVEN a successful `POST /task/{id}/field/{field_id}`, WHEN `set_custom_field_value()` completes, THEN it returns `Ok(())`.
- [ ] GIVEN a `CustomFieldDefinition` response with unknown field types, WHEN deserialized, THEN unknown types are preserved as strings without error.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/models/task.rs` (existing CustomField struct for reference), `crates/clickup-api/src/endpoints/comments.rs` (CRUD endpoint exemplar), `crates/clickup-api/src/client.rs` (HTTP helper methods)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests; no `panic!()` in lib
  - [ ] P0: Crate boundaries respected
  - [ ] P1: `cargo test -p clickup-api` passes
  - [ ] P1: Wiremock tests for both endpoints
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean
  - [ ] P2: `///` doc comments on all public types and methods

---

### S-2: CLI Field Commands

**As a** CLI user, **I want** to run `clickup field list --list LIST_ID` to see available custom fields and `clickup field set --task TASK_ID --field FIELD_ID --value "..."` to set a value, **so that** I can manage custom field data from the command line.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-cli/src/commands/fields.rs` | Add `FieldCommands` enum (clap Subcommand): `List { #[arg(long)] list: String }`, `Set { #[arg(long)] task: String, #[arg(long)] field: String, #[arg(long)] value: String }`. |
| 2 | Create | `crates/clickup-cli/src/commands/fields.rs` | Add `list_fields()` handler: call `client.get_custom_fields(&list_id)` → display table with columns: ID, Name, Type, Required. Support `--format` (table/json). |
| 3 | Create | `crates/clickup-cli/src/commands/fields.rs` | Add `set_field()` handler: parse value string to appropriate `serde_json::Value` (try number, then boolean, then string) → build `SetCustomFieldRequest` → call `client.set_custom_field_value()` → print `✓ Field updated`. |
| 4 | Modify | `crates/clickup-cli/src/commands/mod.rs` | Add `pub mod fields;` declaration. |
| 5 | Modify | `crates/clickup-cli/src/main.rs` | Add `Field(FieldCommands)` variant to root `Commands` enum and wire dispatch. |
| 6 | Test | `crates/clickup-cli/src/commands/fields.rs` | Unit test: value parser correctly converts "42" to `json!(42)`, "true" to `json!(true)`, "hello" to `json!("hello")`. |

#### Acceptance Criteria

- [ ] GIVEN `clickup field list --list 123`, WHEN executed, THEN a table is printed showing each field's ID, name, type, and required status.
- [ ] GIVEN `clickup field list --list 123 --format json`, WHEN executed, THEN the raw JSON array of field definitions is printed.
- [ ] GIVEN `clickup field set --task abc --field cf1 --value "hello"`, WHEN executed, THEN `POST /task/abc/field/cf1` is sent with body `{"value":"hello"}`.
- [ ] GIVEN `clickup field set --task abc --field cf1 --value "42"`, WHEN executed, THEN the value is sent as a JSON number `{"value":42}`.
- [ ] GIVEN `clickup field set --task abc --field cf1 --value "true"`, WHEN executed, THEN the value is sent as a JSON boolean `{"value":true}`.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs` (command pattern exemplar), `crates/clickup-cli/src/main.rs` (command dispatch), `crates/clickup-cli/src/output.rs` (table/json formatting)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-3: TUI Custom Field List and Editor

**As a** TUI user on the TaskDetail screen, **I want** to press `F` to open a custom field editor that lists the task's fields and lets me select one to edit, **so that** I can modify custom field values without leaving the TUI.

**Timebox:** ≤2d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-tui/src/app.rs` | Add state: `field_editor_open: bool`, `field_editor_selected: usize`, `field_editor_fields: Vec<CustomField>` (populated from current task), `field_edit_value: Option<String>` (input buffer for the selected field). |
| 2 | Create | `crates/clickup-tui/src/ui/field_editor.rs` | Render floating panel: list task's custom fields with current values. Selected field highlighted. When editing: show type-appropriate input below the list. Bottom bar: `Enter: edit | Esc: close`. |
| 3 | Modify | `crates/clickup-tui/src/ui/mod.rs` | Add `pub mod field_editor;` declaration. |
| 4 | Extend | `crates/clickup-tui/src/input.rs` | In `handle_task_detail()`: `F` key opens field editor. Route keys when open: `j`/`↓`/`k`/`↑` navigate, `Enter` begins editing selected field (show input widget), `Esc` closes editor or cancels current edit. |
| 5 | Extend | `crates/clickup-tui/src/data.rs` | Add `spawn_set_custom_field()`: takes client, tx, task_id, field_id, `SetCustomFieldRequest` → calls `client.set_custom_field_value()` → sends `DataPayload::CustomFieldUpdated` on success. |
| 6 | Extend | `crates/clickup-tui/src/event.rs` | Add `DataPayload::CustomFieldUpdated` variant. Handle in main loop: refresh task detail to reflect the new value, close field editor. |

#### Acceptance Criteria

- [ ] GIVEN the user is on TaskDetail, WHEN they press `F`, THEN a floating panel shows the task's custom fields with their current display values.
- [ ] GIVEN the field editor is open, WHEN the user selects a text field and presses Enter, THEN a text input appears with the current value.
- [ ] GIVEN the user has typed a new value, WHEN they press Enter, THEN a `POST /task/{id}/field/{field_id}` is sent with the new value.
- [ ] GIVEN the field editor is open, WHEN Esc is pressed, THEN the editor closes without changes.
- [ ] GIVEN a successful field update, WHEN the response arrives, THEN the task detail refreshes and the field shows the new value.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/task_detail.rs` (custom field rendering patterns), `crates/clickup-tui/src/ui/comment_sidebar.rs` (floating panel pattern), `crates/clickup-tui/src/app.rs` (modal state pattern)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-4: Type-Specific Input Handlers

**As a** TUI user editing a custom field, **I want** the input control to match the field type (text input, number input, dropdown picker, checkbox toggle, date picker, multi-select for labels), **so that** each field type has an appropriate editing experience.

**Timebox:** ≤3d | **Risk:** P2 | **Depends on:** S-3

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Extend | `crates/clickup-tui/src/ui/field_editor.rs` | Add type dispatch: based on `field_type` string, render the appropriate input widget. For `text`/`url`/`email`/`phone`/`short_text`: text input. For `number`/`currency`: number input (validate numeric). For `checkbox`: toggle on Enter (no input widget needed). For `date`: text input with YYYY-MM-DD format. For `drop_down`: picker showing options from `type_config`. For `labels`: multi-select picker from `type_config` options. For `emoji`: number picker (1-5 or 1-N based on `type_config.count`). |
| 2 | Extend | `crates/clickup-tui/src/ui/field_editor.rs` | Add dropdown picker sub-widget: list `type_config.options` by name, highlight selected, Enter to confirm. Build `SetCustomFieldRequest` with the option's UUID. |
| 3 | Extend | `crates/clickup-tui/src/ui/field_editor.rs` | Add labels multi-select sub-widget: list `type_config.options` with checkboxes, Space to toggle, Enter to confirm. Build `SetCustomFieldRequest` with array of selected UUIDs. |
| 4 | Extend | `crates/clickup-tui/src/input.rs` | Route field-editor input to type-specific handlers based on which sub-widget is active. |
| 5 | Create | `crates/clickup-tui/src/ui/field_editor.rs` | Add value converter: convert user input to the correct `serde_json::Value` format per field type. Text→string, number→f64, checkbox→bool, date→Unix ms string, dropdown→UUID string, labels→UUID array, emoji→integer, manual_progress→`{current: N}`. |
| 6 | Test | `crates/clickup-tui/src/ui/field_editor.rs` | Render tests: dropdown picker shows option labels, labels picker shows checkboxes, checkbox field toggles immediately. |

#### Acceptance Criteria

- [ ] GIVEN a `text` field in the editor, WHEN the user enters editing mode, THEN a text input widget appears.
- [ ] GIVEN a `drop_down` field with options ["High", "Medium", "Low"], WHEN the user enters editing mode, THEN a picker shows the three options.
- [ ] GIVEN a `labels` field with options, WHEN the user enters editing mode, THEN a multi-select picker shows all labels with checkboxes.
- [ ] GIVEN a `checkbox` field, WHEN the user presses Enter, THEN the value toggles between true/false immediately and the API call is made.
- [ ] GIVEN a `date` field, WHEN the user types "2026-05-01" and confirms, THEN the value is sent as a Unix ms timestamp string.
- [ ] GIVEN a `drop_down` field and the user selects "High", WHEN confirmed, THEN the API receives `{"value":"uuid-for-high"}`.
- [ ] GIVEN an unknown field type, WHEN the user attempts to edit, THEN a text input is shown as fallback.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/models/task.rs` (CustomField.resolved_options(), type_config structure), `crates/clickup-tui/src/ui/field_editor.rs` (from S-3), `crates/clickup-tui/src/ui/edit_widgets.rs` (from F06/S-4 — reuse input patterns)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside tests
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P1: Render tests for dropdown and labels pickers
  - [ ] P2: `cargo fmt --all -- --check` clean
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` clean
  - [ ] P2: All public items have `///` doc comments

---

### S-5: Integration Tests and Fixtures

**As a** maintainer, **I want** wiremock tests covering custom field listing and value setting, **so that** regressions are caught automatically.

**Timebox:** ≤1d | **Risk:** P1 | **Depends on:** S-1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create | `crates/clickup-api/tests/fixtures/custom_fields_list.json` | Fixture with 6+ field definitions of different types, including type_config with options for dropdown/labels. |
| 2 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock test: `test_get_custom_fields_returns_definitions` — verify deserialization of all field types. |
| 3 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock test: `test_set_custom_field_text_value` — verify POST body for text field. |
| 4 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock test: `test_set_custom_field_dropdown_value` — verify POST body contains UUID. |
| 5 | Test | `crates/clickup-api/tests/fixture_tests.rs` | Wiremock test: `test_get_custom_fields_empty_list` — verify empty list returns empty vec. |

#### Acceptance Criteria

- [ ] GIVEN a wiremock server returning the custom fields fixture, WHEN `get_custom_fields()` is called, THEN all field definitions deserialize correctly with type_config preserved.
- [ ] GIVEN a wiremock server, WHEN `set_custom_field_value()` is called with a text value, THEN the request body is `{"value":"hello"}`.
- [ ] GIVEN a wiremock server returning an empty fields array, WHEN `get_custom_fields()` is called, THEN it returns an empty `Vec`.

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/tests/fixture_tests.rs` (existing test patterns), `crates/clickup-api/tests/fixtures/` (fixture directory)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P1: `cargo test --workspace` passes
  - [ ] P1: All new endpoints have wiremock tests
  - [ ] P2: `cargo clippy --workspace -- -D warnings` clean

---

## Execution Sequence

```
S-1 (API) ──┬──→ S-2 (CLI)
            ├──→ S-3 (TUI list/editor) ──→ S-4 (type handlers)
            └──→ S-5 (tests)
```

- **Phase 1:** S-1 — API endpoints and models
- **Phase 2 (parallel):** S-2 (CLI) + S-3 (TUI field list) + S-5 (tests)
- **Phase 3:** S-4 (type-specific input handlers, after S-3)

## Assumptions

1. `POST /task/{task_id}/field/{field_id}` returns an empty body or a simple success indicator on success, not the updated task. **Risk if wrong:** May need to parse a response body — adjust `set_custom_field_value()` return type.
2. The `type_config.options` structure is consistent between `GET /list/{id}/field` (field definitions) and `GET /task/{id}` (field values on tasks). **Risk if wrong:** May need separate option resolution per endpoint.
3. Setting a dropdown field requires the option's UUID, not the display name. **Risk if wrong:** Need to resolve name → UUID in the CLI value parser.
4. The `manual_progress` field type requires `{"value": {"current": N}}` format. **Risk if wrong:** Adjust serialization — minimal impact.

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | 3 crates, 2 new endpoints, 1 new model file, CLI commands, TUI editor |
| Ambiguity | 2 | 15+ field types each with different value formats; some may surprise |
| Dependencies | 2 | Depends on F06 patterns (TUI edit widgets, spawn_update_task); new endpoints are independent |
| Risk | 2 | Type heterogeneity is the main risk — unknown field types or undocumented value formats |

**Total: 8/12** → Careful processing; type-specific handlers may reveal edge cases
