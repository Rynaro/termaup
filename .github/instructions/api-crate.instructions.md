---
applyTo: "crates/clickup-api/**/*.rs"
---

# clickup-api — Coding Rules

## Context

The `clickup-api` library crate is the shared foundation used by both `clickup-cli` and `clickup-tui`.

## Constraints

- This crate must NEVER depend on `clap`, `ratatui`, `crossterm`, or any CLI/TUI-specific crate
- All public functions must be `async` and return `crate::error::Result<T>`
- Use `thiserror` for all error definitions — never `anyhow` in this crate
- Every endpoint method must include `tracing::debug!` logging for the HTTP call
- All serde model structs must derive `Debug, Clone, Serialize, Deserialize`
- Use `reqwest::Client` connection pooling — create the client once and reuse
- Rate limiting must be transparent to callers — ClickUpClient handles it internally
- Pagination must be automatic — endpoint methods return the full collected data

## Patterns

Error mapping from HTTP status codes to `ClickUpError` variants:

| HTTP status | `ClickUpError` variant | Notes |
|-------------|----------------------|-------|
| 401 | `AuthError` | Invalid or missing token |
| 404 | `NotFound` | Resource does not exist |
| 429 | `RateLimited` | Parse `X-RateLimit-Reset` header |
| Other 4xx/5xx | `ApiError` | Parse `err` and `ECODE` from response body |
| Network failure | `NetworkError` | Connection, DNS, timeout errors |
| JSON parse failure | `DeserializationError` | Invalid response body |
