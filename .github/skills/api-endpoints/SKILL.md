---
name: api-endpoints
description: Implement typed async endpoint methods on ClickUpClient in clickup-api. Use this when creating or modifying files in clickup-api/src/endpoints/.
---

# API Endpoints

## Context

This skill implements the typed async methods that call each ClickUp API endpoint. Each method lives in its own module under `endpoints/`, uses `ClickUpClient` for HTTP, and returns strongly-typed model structs.

## Crates

- `clickup-api`

## Prerequisites

- Skill `http-client` completed — `ClickUpClient` with `get()` and `get_with_params()` methods exist
- Skill `domain-models` completed — all model structs exist

## Deliverables

- `crates/clickup-api/src/endpoints/mod.rs` — Re-exports all endpoint modules
- `crates/clickup-api/src/endpoints/users.rs` — `get_authenticated_user()`
- `crates/clickup-api/src/endpoints/workspaces.rs` — `get_workspaces()`
- `crates/clickup-api/src/endpoints/spaces.rs` — `get_spaces()`, `get_space()`
- `crates/clickup-api/src/endpoints/folders.rs` — `get_folders()`
- `crates/clickup-api/src/endpoints/lists.rs` — `get_lists_in_folder()`, `get_folderless_lists()`, `get_list()`
- `crates/clickup-api/src/endpoints/tasks.rs` — `get_tasks()`, `get_task()`
- `crates/clickup-api/src/endpoints/comments.rs` — `get_task_comments()`

## Implementation

Implement each endpoint as an `impl ClickUpClient` method. Reference the [endpoint table](./endpoint-reference.md) for exact URL patterns and parameters.

### Method pattern

```rust
impl ClickUpClient {
    /// Get the authenticated user.
    pub async fn get_authenticated_user(&self) -> Result<User> {
        let response: AuthenticatedUserResponse = self.get("/user").await?;
        Ok(response.user)
    }
}
```

### Key considerations

- Unwrap response wrappers (e.g., `AuthenticatedUserResponse` → `User`, `WorkspacesResponse` → `Vec<Workspace>`)
- `get_tasks()` must use the pagination helper — accumulate all pages until `last_page == true`
- `get_task()` should pass `include_subtasks=true` and `include_markdown_description=true` as query params
- Every method must include `tracing::debug!("GET {path}")` before the HTTP call
- Return `crate::error::Result<T>` — never `anyhow`

### Testing pattern

Each endpoint module should include `#[cfg(test)] mod tests` that:
1. Start a `wiremock::MockServer`
2. Mount a mock for the endpoint URL returning the corresponding fixture JSON
3. Create a `ClickUpClient` pointing at the mock server
4. Call the endpoint method and assert the returned struct fields
- Include "sparse response" tests: mock the endpoint returning minimal JSON (only `id` and `name`)
- Include tests for TIML task shape variance (missing `list`, `folder`, `space` fields)
- Include tests for null/absent/false handling on response fields

## Acceptance

- All endpoint methods compile and return the correct model types
- Each endpoint has at least one `wiremock`-based integration test
- Pagination correctly collects all pages for `get_tasks()`
- `get_task()` passes `include_subtasks=true&include_markdown_description=true`
- All methods log the HTTP call via `tracing::debug!`
