# ClickUp API v2 — Known Quirks & Defensive Patterns

This document catalogs known inconsistencies in the ClickUp API v2 responses.
All model structs in `clickup-api` must be designed to survive these quirks.

## Golden Rule

> Only `id` and `name` fields should be required (non-`Option`, non-`#[serde(default)]`) on model structs. Every other field must tolerate absence, `null`, or unexpected types.

## Quirk Catalog

### 1. Null vs Absent vs Empty Arrays

ClickUp uses three different representations for "no items":

| Representation | Example | `#[serde(default)]` handles it? |
|---|---|---|
| Absent field | `{}` (no `assignees` key) | ✅ Yes |
| Explicit null | `{"assignees": null}` | ❌ No — panics |
| Empty array | `{"assignees": []}` | ✅ Yes |

**Fix**: All `Vec` fields must use:
```rust
#[serde(default, deserialize_with = "crate::serde_helpers::deserialize_null_as_default")]
```

### 2. Type-Unstable IDs

IDs arrive as strings in some endpoints and integers in others:

```json
// From list endpoint:
{"id": "abc123"}
// From search/webhook:
{"id": 12345}
```

**Fix**: Use `deserialize_string_or_number` on all ID fields.

### 3. Boolean-Integer Confusion

Some boolean fields arrive as `0`/`1` integers instead of `true`/`false`:

```json
// Expected:
{"resolved": true}
// Actual (checklist items):
{"resolved": 1}
```

**Fix**: Use `deserialize_bool_or_int` or `deserialize_option_bool_or_int`.

### 4. Priority as `false`

When a task has no priority, ClickUp returns `"priority": false` instead of `"priority": null`:

```json
{"priority": false}
// instead of:
{"priority": null}
```

**Fix**: Use `deserialize_maybe_false` on the priority field.

### 5. Time Values as Objects

The `time_spent` field can be a raw number OR a wrapper object:

```json
// Raw:
{"time_spent": 3600000}
// Wrapped:
{"time_spent": {"time": 3600000}}
// String:
{"time_spent": "3600000"}
```

**Fix**: Use `deserialize_time_value`.

### 6. TIML Tasks — Missing Structural Fields

Tasks In Multiple Lists (TIML) are tasks that appear in a list but "belong" to a different home list. When fetched with `include_timl=true`, these tasks may be **missing** the `list`, `folder`, and `space` fields entirely, or they may reference their home list instead of the queried list.

```json
// Normal task:
{"id": "abc", "list": {"id": "l1", "name": "Sprint 1"}, "folder": {"id": "f1"}, "space": {"id": "s1"}}

// TIML task — fields may be absent:
{"id": "xyz", "name": "TIML task"}
```

**Fix**: `list`, `folder`, `space` must be `Option<T>` with `#[serde(default)]`.

### 7. Deleted / System Users

The `creator` field may reference a deleted user (with null username/email) or a system user (with `id: -1`):

```json
{"creator": {"id": -1, "username": null, "email": null}}
// Or absent entirely on some task variants
```

**Fix**: `creator` should be `Option<User>` with `#[serde(default)]`. User fields (`username`, `email`) must default to `""` when null/absent.

### 8. String-or-Null on User Fields

User `username` and `email` fields may be `null` (for deactivated users) instead of strings:

```json
{"id": 42, "username": null, "email": null}
```

**Fix**: Use `deserialize_string_or_null` with `#[serde(default)]`.

### 9. Endpoint Response Shape Variance

The same entity type (e.g., Task) has different shapes depending on the endpoint:

| Endpoint | Fields included |
|---|---|
| `GET /task/{id}` | Full task with all fields |
| `GET /list/{id}/task` | Task list — may omit `markdown_description`, `subtasks` |
| TIML tasks in list | May omit `list`, `folder`, `space` |
| Subtask objects | Nested — may omit `creator`, `custom_fields` |
| Search results | Slimmer — fewer fields than direct fetch |

**Fix**: All non-ID fields must be optional or defaulted.

### 10. Extra Fields Without Warning

ClickUp adds new fields to responses without API versioning. Fields like `permission_level`, `sharing`, `team_id`, `archived` may appear without notice.

**Fix**: Serde's default behavior ignores unknown fields. Do NOT use `#[serde(deny_unknown_fields)]`.

## Serde Helper Reference

| Helper | Purpose | Use on |
|--------|---------|--------|
| `deserialize_string_or_number` | ID that may be string or int | All `id` fields |
| `deserialize_default_string_or_number` | ID that may be string, int, or null, defaults to `""` | ID fields on `Default`-able sub-structs |
| `deserialize_option_string_or_number` | Optional ID | Optional ID fields |
| `deserialize_string_or_null` | String that may be null → defaults to `""` | User `username`, `email` |
| `deserialize_null_as_default` | Value that may be null → `Default::default()` | All `Vec<T>` fields |
| `deserialize_bool_or_int` | Bool that may be 0/1 | Checklist `resolved` |
| `deserialize_option_bool_or_int` | Optional bool that may be 0/1 | Optional checklist `resolved` |
| `deserialize_maybe_false` | Object or false or null → `Option<T>` | `priority` field |
| `deserialize_time_value` | Number, `{"time": ms}`, string, or null | `time_spent`, `time_estimate` |
| `deserialize_i32_or_string` | Integer that may be string | `orderindex` |

## Testing Requirements

Every model struct must have these test cases:

1. **Full response** — All fields populated with typical values
2. **Minimal response** — Only `id` (and `name` where applicable)
3. **Null-heavy response** — Every nullable field set to `null` explicitly
4. **Type-mixed response** — IDs as integers, booleans as integers, times as objects
