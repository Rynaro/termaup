---
name: domain-models
description: Create serde model structs for all ClickUp API entities in clickup-api. Use this when creating or modifying files in clickup-api/src/models/.
---

# Domain Models

## Context

This skill creates all the Rust structs that represent ClickUp API entities. Each struct must correctly deserialize from the ClickUp API's JSON responses. Use the fixture files in this skill's directory as the source of truth for field names and types.

## Crates

- `clickup-api`

## Prerequisites

- Skill `foundation` completed — workspace compiles

## Deliverables

- `crates/clickup-api/src/models/mod.rs` — Re-exports all model types
- `crates/clickup-api/src/models/user.rs` — `User`, `AuthenticatedUser`, `AuthenticatedUserResponse`
- `crates/clickup-api/src/models/workspace.rs` — `Workspace`, `WorkspaceMember`, `WorkspacesResponse`
- `crates/clickup-api/src/models/space.rs` — `Space`, `SpacesResponse`
- `crates/clickup-api/src/models/status.rs` — `Status` (with `#[serde(rename = "type")]` for the reserved keyword)
- `crates/clickup-api/src/models/folder.rs` — `Folder`, `FoldersResponse`
- `crates/clickup-api/src/models/list.rs` — `List`, `ListsResponse`
- `crates/clickup-api/src/models/task.rs` — `Task`, `TasksResponse`, `TaskPriority`, `Tag`, `CustomField`
- `crates/clickup-api/src/models/comment.rs` — `Comment`, `CommentsResponse`

## Implementation

### Derive rules for ALL model structs

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
```

### Field conventions

- Use `Option<T>` for nullable API fields (e.g., `due_date`, `date_closed`, `profilePicture`)
- Use `#[serde(default)]` on `Vec` fields that may be absent (e.g., `assignees`, `tags`, `subtasks`, `custom_fields`)
- Use `#[serde(rename = "...")]` for camelCase fields (e.g., `profilePicture` → `profile_picture`)
- Use `#[serde(alias = "...")]` when the API uses inconsistent casing
- ClickUp timestamps are millisecond Unix timestamps as strings (e.g., `"1679012345000"`) — store as `Option<String>` and provide conversion methods

### Defensive Deserialization Rules

The ClickUp API is type-unstable and endpoint-inconsistent. See `docs/clickup-api-quirks.md` for the full catalog. Key rules:

1. **Only `id` and `name` may be required fields** — all others must be `Option<T>` or `#[serde(default)]`
2. **All `Vec` fields**: `#[serde(default, deserialize_with = "crate::serde_helpers::deserialize_null_as_default")]` — ClickUp returns `null` for empty arrays
3. **All ID fields**: use `deserialize_string_or_number` (or `deserialize_default_string_or_number` for defaultable structs) — IDs can be strings or integers
4. **Boolean fields on non-core structs**: consider `deserialize_bool_or_int` — ClickUp sends `0`/`1` instead of `true`/`false`
5. **Sub-object fields** (`list`, `folder`, `space`, `creator`): MUST be `Option<T>` with `#[serde(default)]` — absent for TIML tasks, search results
6. **Priority-like fields**: use `deserialize_maybe_false` — API sends `false` instead of `null`
7. **Time fields**: use `deserialize_time_value` — number, `{"time": ms}`, or string

### Testing requirements

Every model struct must have these test cases:
1. **Full response** — All fields populated with typical values
2. **Minimal response** — Only `id` (and `name` where applicable)  
3. **Null-heavy response** — Every nullable field set to explicit `null`
4. **Type-mixed response** — IDs as integers, booleans as integers, times as objects

### Key type mappings

| API field | Rust type | Notes |
|-----------|-----------|-------|
| `id` (user) | `u64` | Numeric in API |
| `id` (workspace/space/folder/list/task) | `String` | String in API |
| `status.type` | `String` with `#[serde(rename = "type")]` | Reserved keyword |
| `date_created`, `date_updated` | `Option<String>` | Millisecond timestamp as string |
| `priority` | `Option<TaskPriority>` | Can be null |
| `custom_fields` | `Vec<CustomField>` with `#[serde(default)]` | May be absent |

### Fixture files

Use these fixtures for testing deserialization:
- [user-response.json](./fixtures/user-response.json) — `GET /user` response
- [workspaces-response.json](./fixtures/workspaces-response.json) — `GET /team` response
- [tasks-response.json](./fixtures/tasks-response.json) — `GET /list/{list_id}/task` response

### Testing pattern

Each model file should include a `#[cfg(test)] mod tests` block that:
1. Loads the corresponding fixture JSON
2. Deserializes it into the response wrapper type
3. Asserts key fields are correctly populated

## Acceptance

- All model structs deserialize from the provided JSON fixtures without error
- `serde_json::from_str` round-trips for all response types
- Reserved keyword `type` is handled via `#[serde(rename = "type")]` on `Status`
- Optional fields correctly handle both `null` and absent cases
- Vec fields with `#[serde(default)]` handle both absent and empty array cases
