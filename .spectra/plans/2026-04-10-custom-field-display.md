# Custom Field Display Values

> **SPECTRA v4.2.0** — Plan artifact
> **Date:** 2026-04-10
> **Feature:** Fix custom field display to show human-readable values instead of internal IDs

---

## Problem Statement

Custom fields on the Task Detail screen (TUI and CLI) display raw internal identifiers instead of human-readable values. For field types like `drop_down` and `labels`, the ClickUp API returns the selected option's UUID as the `value`, while the mapping from UUIDs to display names lives in the `type_config.options` array — a field the `CustomField` model currently does not capture.

Additionally, several custom field types are not rendered at all: `emoji` (rating), `users` (people), `tasks` (linked), `manual_progress`, `automatic_progress`, `location`, and `currency` (missing currency symbol).

## Root Cause

The `CustomField` struct in `crates/clickup-api/src/models/task.rs` only stores four fields:

```rust
pub struct CustomField {
    pub id: String,
    pub name: String,
    pub field_type: String,           // "drop_down", "labels", etc.
    pub value: Option<serde_json::Value>,  // e.g. "uuid-option-a" — just the ID!
}
```

The `type_config` field from the API response — which contains the `options` array mapping IDs to names — is silently discarded during deserialization. Without it, the format functions in both TUI and CLI cannot resolve option IDs to display names.

### ClickUp API Response Structure (what we receive but don't capture)

```json
{
  "id": "cf_123",
  "name": "Priority Level",
  "type": "drop_down",
  "type_config": {
    "options": [
      { "id": "uuid-a", "name": "High", "color": "#FF0000", "orderindex": 0 },
      { "id": "uuid-b", "name": "Medium", "color": "#FFFF00", "orderindex": 1 },
      { "id": "uuid-c", "name": "Low", "color": "#00FF00", "orderindex": 2 }
    ]
  },
  "value": "uuid-a"
}
```

Current display: `Priority Level: uuid-a`
Expected display: `Priority Level: High`

For `labels`, the value is an array of UUIDs: `["label-id-1", "label-id-2"]`, and `type_config.options` uses `label` instead of `name`.

---

## Approach

**Selected hypothesis:** Add `type_config` to the model + consolidate formatting into a `display_value()` method on `CustomField` in the API crate.

### Rationale

- Resolving an option UUID to its display name is a **data transformation**, not a UI concern — it belongs in the API crate
- Both TUI and CLI have nearly-identical 60-line format functions (duplicated logic, duplicated bug)
- A `display_value()` method on the model eliminates duplication and ensures consistent behavior
- Date formatting uses identical `chrono` format string (`%b %d, %Y`) in both crates — no divergence to manage
- The API crate already depends on `chrono` (used in `rate_limiter.rs` and `client.rs`)

### Rejected Alternatives

1. **Keep duplication, just pass type_config** — Lower risk but perpetuates maintenance burden. Both format functions would need identical updates for every new field type. Rejected for long-term cost.

2. **Typed `CustomFieldValue` enum** — Type-safe with exhaustive matching, but ClickUp adds field types without notice. A closed enum violates open-closed principle and would break deserialization on unknown types. Rejected for fragility.

3. **Shared formatting module (not on struct)** — Free function `format_custom_field(...)` in a utility module. Less discoverable than a method. No material advantage over the method approach. Rejected for ergonomics.

---

## Scope

| In Scope | Out of Scope | Deferred |
|----------|--------------|----------|
| Add `type_config` to `CustomField` model | Custom field editing/mutation | Color-coded option display in TUI |
| `display_value()` method on `CustomField` | New API endpoints for custom fields | Custom field filtering by resolved value |
| Fix dropdown + labels value resolution | Voting custom field support | `automatic_progress` display (read-only, no user-settable value) |
| Handle emoji, users, location, progress, currency types | | |
| Update TUI `format_custom_field_value()` → use `display_value()` | | |
| Update CLI `format_custom_field()` → use `display_value()` | | |
| Update test fixtures with `type_config` data | | |
| Document custom field quirks in `docs/clickup-api-quirks.md` | | |

---

## Stories

### Story 1: Extend `CustomField` model with `type_config`

**Title:** Add `type_config` field and option resolution helpers to `CustomField`

**As a** developer using clickup-api,
**I want** the `CustomField` struct to capture `type_config` from the API response,
**so that** dropdown and label option IDs can be resolved to human-readable names.

**Timebox:** ≤2d

#### Acceptance Criteria

1. **GIVEN** a task API response with `type_config` containing `options`, **WHEN** deserialized into `CustomField`, **THEN** `type_config` is preserved as `Option<serde_json::Value>`.
2. **GIVEN** a `CustomField` with `type_config` absent or `null`, **WHEN** deserialized, **THEN** `type_config` is `None` (no panic).
3. **GIVEN** additional API fields like `required`, `date_created`, `hide_from_guests`, **WHEN** present in the response, **THEN** deserialization succeeds (unknown fields are silently ignored — `serde` default behavior).

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify `CustomField` struct | `crates/clickup-api/src/models/task.rs` | Add `type_config: Option<serde_json::Value>` with `#[serde(default)]` |
| 2 | Test full fixture | `crates/clickup-api/src/models/task.rs` | Update `sample_task_json()` to include `type_config` with options for the dropdown field, add a labels field |
| 3 | Test minimal fixture | `crates/clickup-api/src/models/task.rs` | Test that `type_config` absent/null deserializes to `None` |
| 4 | Test type-mixed fixture | `crates/clickup-api/src/models/task.rs` | Test with various field types: dropdown, labels, emoji, currency, users, location, progress |
| 5 | Update integration fixture | `crates/clickup-api/tests/fixtures/task_detail.json` | Add realistic `type_config` to the dropdown custom field, add more field type examples |

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/src/models/task.rs` (existing `CustomField` struct)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside test code
  - [ ] P1: `cargo test -p clickup-api` passes with new fixtures
  - [ ] P1: Deserialization tests cover full, minimal, null-heavy, type-mixed
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean

---

### Story 2: Add `display_value()` method on `CustomField`

**Title:** Implement `CustomField::display_value()` that resolves all field types to display strings

**As a** CLI/TUI developer,
**I want** a `display_value()` method on `CustomField` that returns a human-readable string,
**so that** I don't need to duplicate field-type-specific formatting logic in each binary crate.

**Timebox:** ≤3d

**Depends on:** Story 1

#### Acceptance Criteria

1. **GIVEN** a `drop_down` field with `value: "uuid-a"` and `type_config.options` containing `{id: "uuid-a", name: "High"}`, **WHEN** `display_value()` is called, **THEN** it returns `"High"`.
2. **GIVEN** a `labels` field with `value: ["id-1", "id-2"]` and `type_config.options` containing matching entries with `label` keys, **WHEN** `display_value()` is called, **THEN** it returns `"Label 1, Label 2"`.
3. **GIVEN** a `drop_down` field with `value: "unknown-uuid"` and no matching option in `type_config`, **WHEN** `display_value()` is called, **THEN** it returns the raw UUID string (graceful fallback).
4. **GIVEN** a `drop_down` field with `type_config` absent (`None`), **WHEN** `display_value()` is called, **THEN** it falls back to displaying the raw value (no panic).
5. **GIVEN** a `number` field with `value: 5`, **WHEN** `display_value()` is called, **THEN** it returns `"5"`.
6. **GIVEN** a `currency` field with `value: 99.99` and `type_config: {currency_type: "USD"}`, **WHEN** `display_value()` is called, **THEN** it returns `"USD 99.99"`.
7. **GIVEN** a `checkbox` field with `value: true`, **WHEN** `display_value()` is called, **THEN** it returns `"✅"`.
8. **GIVEN** a `date` field with `value: "1710000000000"`, **WHEN** `display_value()` is called, **THEN** it returns a formatted date like `"Mar 09, 2024"`.
9. **GIVEN** an `emoji` field with `value: 3` and `type_config: {code_point: "2b50", count: 5}`, **WHEN** `display_value()` is called, **THEN** it returns `"⭐⭐⭐"` (3 emoji repetitions).
10. **GIVEN** a `users` field with `value: [{id: 123, username: "alice"}, {id: 456, username: "bob"}]`, **WHEN** `display_value()` is called, **THEN** it returns `"alice, bob"`.
11. **GIVEN** a `manual_progress` field with `value: {current: 50}` and `type_config: {start: 0, end: 100}`, **WHEN** `display_value()` is called, **THEN** it returns `"50%"`.
12. **GIVEN** a `location` field with `value: {location: {lat: -28.0, lng: 153.4}, formatted_address: "Gold Coast QLD, Australia"}`, **WHEN** `display_value()` is called, **THEN** it returns `"Gold Coast QLD, Australia"`.
13. **GIVEN** a field with `value: None`, **WHEN** `display_value()` is called, **THEN** it returns `"—"`.
14. **GIVEN** an unknown field type with any value, **WHEN** `display_value()` is called, **THEN** it returns a reasonable string representation (no panic).

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Create `display_value()` method | `crates/clickup-api/src/models/task.rs` | `impl CustomField { pub fn display_value(&self) -> String }` handling all known field types |
| 2 | Handle `drop_down` resolution | same | Look up `value` string in `type_config.options[].id` → return `name` |
| 3 | Handle `labels` resolution | same | Look up each string in `value` array in `type_config.options[].id` → return `label` or `name`, join with `", "` |
| 4 | Handle `emoji` rendering | same | Parse `type_config.code_point` as Unicode, repeat `value` times |
| 5 | Handle `currency` formatting | same | Prepend `type_config.currency_type` to the formatted number |
| 6 | Handle `users` formatting | same | Extract `username` from each user object in value array |
| 7 | Handle `manual_progress` | same | Calculate percentage from `value.current` / `type_config.end` |
| 8 | Handle `location` | same | Extract `formatted_address` from value |
| 9 | Handle `tasks` field type | same | Extract task IDs from value, display as comma-separated list |
| 10 | Retain existing handlers | same | `number`, `checkbox`, `date`, `text`, `url`, `email`, `phone`, `short_text` — port logic from current format functions |
| 11 | Unit tests | same | One test per field type + edge cases (missing type_config, unknown option ID, null value, unknown type) |

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/task_detail.rs:395-457` (existing TUI format function to port), `crates/clickup-cli/src/commands/tasks.rs:309-370` (existing CLI format function to port)
- **Gates:**
  - [ ] P0: `cargo build --workspace` compiles
  - [ ] P0: No `.unwrap()` outside test code; no `panic!()` in library code
  - [ ] P1: Unit tests for every field type listed in acceptance criteria
  - [ ] P1: Edge case tests for missing `type_config`, unknown option IDs, unknown field types
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean
  - [ ] P2: `/// doc comment` on `display_value()` method

---

### Story 3: Update TUI task detail to use `display_value()`

**Title:** Replace TUI custom field formatting with `CustomField::display_value()`

**As a** TUI user viewing task details,
**I want** custom fields to show human-readable values (e.g., "High" instead of "uuid-a"),
**so that** the terminal experience matches the ClickUp web UI.

**Timebox:** ≤1d

**Depends on:** Story 2

#### Acceptance Criteria

1. **GIVEN** a task with a dropdown custom field, **WHEN** viewing task detail in TUI, **THEN** the dropdown shows the option name, not the UUID.
2. **GIVEN** a task with labels custom field, **WHEN** viewing task detail in TUI, **THEN** labels show as comma-separated display names.
3. **GIVEN** existing formatting for number/checkbox/date/text fields, **WHEN** viewing task detail, **THEN** behavior is unchanged (backward-compatible).

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify rendering loop | `crates/clickup-tui/src/ui/task_detail.rs` | Replace `format_custom_field_value(&field.field_type, field.value.as_ref())` with `field.display_value()` |
| 2 | Remove `format_custom_field_value()` | same | Delete the now-unused private function |
| 3 | Verify compilation | — | `cargo build -p clickup-tui` |

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-tui/src/ui/task_detail.rs:155-179` (rendering loop), `crates/clickup-tui/src/ui/task_detail.rs:395-457` (function to remove)
- **Gates:**
  - [ ] P0: `cargo build -p clickup-tui` compiles
  - [ ] P0: No dead code warnings from removed function
  - [ ] P1: `cargo test -p clickup-tui` passes
  - [ ] P2: `cargo clippy -p clickup-tui -- -D warnings` (note: pre-existing dead code warnings may exist)

---

### Story 4: Update CLI task view to use `display_value()`

**Title:** Replace CLI custom field formatting with `CustomField::display_value()`

**As a** CLI user viewing task details,
**I want** custom fields to show human-readable values,
**so that** the CLI output is meaningful and consistent with the TUI.

**Timebox:** ≤1d

**Depends on:** Story 2

#### Acceptance Criteria

1. **GIVEN** a task with a dropdown custom field, **WHEN** running `clickup task get <id>`, **THEN** the dropdown shows the option name, not the UUID.
2. **GIVEN** existing formatting for number/checkbox/date/text fields, **WHEN** viewing task detail, **THEN** behavior is unchanged.

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Modify rendering loop | `crates/clickup-cli/src/commands/tasks.rs` | Replace `format_custom_field(&field.field_type, field.value.as_ref())` with `field.display_value()` |
| 2 | Remove `format_custom_field()` | same | Delete the now-unused private function |
| 3 | Verify compilation | — | `cargo build -p clickup-cli` |

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-cli/src/commands/tasks.rs:219-228` (rendering loop), `crates/clickup-cli/src/commands/tasks.rs:309-370` (function to remove)
- **Gates:**
  - [ ] P0: `cargo build -p clickup-cli` compiles
  - [ ] P0: No dead code warnings from removed function
  - [ ] P1: `cargo test -p clickup-cli` passes
  - [ ] P2: `cargo clippy -p clickup-cli -- -D warnings` clean

---

### Story 5: Update test fixtures and add integration tests

**Title:** Add comprehensive custom field fixtures with `type_config` data

**As a** maintainer,
**I want** test fixtures that reflect real ClickUp API responses including `type_config`,
**so that** custom field value resolution is covered by automated tests.

**Timebox:** ≤2d

**Depends on:** Story 1

#### Acceptance Criteria

1. **GIVEN** `task_detail.json` fixture, **WHEN** inspected, **THEN** it includes custom fields with `type_config.options` for dropdown and labels types.
2. **GIVEN** integration test, **WHEN** deserializing fixture with `type_config`, **THEN** `display_value()` returns the correct display name.
3. **GIVEN** fixtures, **WHEN** they include emoji, currency, users, location, progress custom fields, **THEN** all deserialize correctly and `display_value()` produces expected output.

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Update fixture | `crates/clickup-api/tests/fixtures/task_detail.json` | Add `type_config` to existing dropdown field, add labels/emoji/currency/users/location/progress custom fields |
| 2 | Update fixture test | `crates/clickup-api/tests/fixture_tests.rs` | Add assertions for `display_value()` on each custom field type in the fixture |
| 3 | Add edge-case fixture | `crates/clickup-api/tests/fixtures/task_custom_fields_edge.json` | Task with: missing type_config, unknown option ID, null value, unknown field type, empty options array |
| 4 | Test edge cases | `crates/clickup-api/tests/fixture_tests.rs` | Verify graceful fallback for all edge cases |

#### Agent Hints

- **Class:** builder
- **Context:** `crates/clickup-api/tests/fixtures/task_detail.json` (existing fixture), `crates/clickup-api/tests/fixture_tests.rs` (existing test file)
- **Gates:**
  - [ ] P0: `cargo test -p clickup-api` passes
  - [ ] P1: Every custom field type has at least one fixture + assertion
  - [ ] P1: Edge cases covered (missing type_config, unknown option, null value)
  - [ ] P2: `cargo clippy -p clickup-api -- -D warnings` clean

---

### Story 6: Document custom field quirks

**Title:** Add custom field value resolution quirks to API quirks catalog

**As a** contributor,
**I want** the custom field value format documented in `docs/clickup-api-quirks.md`,
**so that** future developers understand the ID-to-name resolution pattern.

**Timebox:** ≤1d

**Depends on:** Story 1

#### Action Plan

| # | Action | File | Details |
|---|--------|------|---------|
| 1 | Add section | `docs/clickup-api-quirks.md` | Document: dropdown value is option UUID (not name), labels value is array of UUIDs, type_config.options contains the mapping. Note that dropdown options use `name` while labels use `label`. |
| 2 | Document field types | same | Table of all field types with value format and type_config format |

#### Agent Hints

- **Class:** builder
- **Context:** `docs/clickup-api-quirks.md` (existing quirks catalog)
- **Gates:**
  - [ ] P2: Documentation is accurate and follows existing format

---

## Execution Sequence

```
Story 1 (model) ──┬──→ Story 2 (display_value) ──┬──→ Story 3 (TUI)
                  │                               └──→ Story 4 (CLI)
                  ├──→ Story 5 (fixtures)
                  └──→ Story 6 (docs)
```

Stories 3 and 4 can execute in parallel after Story 2.
Stories 5 and 6 can begin after Story 1 (independent of Stories 2-4).

---

## Confidence Report

**Overall: 88%** → AUTO_PROCEED

| Factor | Score | Notes |
|--------|-------|-------|
| Pattern match | 90% | Similar to existing model extensions (comment, checklist patterns) |
| Requirement clarity | 90% | ClickUp API docs confirm the type_config structure; root cause is clear |
| Decomposition stability | 85% | Stories are independent value units; dependency chain is linear |
| Constraint compliance | 85% | No architectural boundary violations; `display_value()` is data transformation |

---

## Complexity Score

| Dimension | Score | Rationale |
|-----------|-------|-----------|
| Scope | 2 | Multi-feature: model + TUI + CLI + tests + docs |
| Ambiguity | 1 | Clear requirements, confirmed API format |
| Dependencies | 2 | 3 crates affected, well-defined interfaces |
| Risk | 1 | Additive changes, backward-compatible, no breaking API |

**Total: 6/12** → Standard processing

---

## Assumptions

1. The ClickUp API always includes `type_config` when custom fields are present in `GET /task/{id}` responses. **Risk if wrong:** `display_value()` gracefully falls back to raw value display — no panic, just degraded UX.
2. Both TUI and CLI date formatting should use the same format (`%b %d, %Y`). **Risk if wrong:** Minor — can be overridden per-crate if needed.
3. The `emoji` field type uses Unicode code points that can be decoded with `char::from_u32(u32::from_str_radix(code_point, 16))`. **Risk if wrong:** Fallback to numeric display (e.g., "3/5").
