---
applyTo: "crates/clickup-api/**/*.rs"
---

# API Crate Instructions

You are working in the `clickup-api` library crate. This is the shared foundation used by both the CLI and TUI.

## Rules
- This crate must NEVER depend on `clap`, `ratatui`, `crossterm`, or any CLI/TUI-specific crate
- All public functions must be `async` and return `crate::error::Result<T>`
- Use `thiserror` for all error definitions — never `anyhow` in this crate
- Every endpoint method must include `tracing::debug!` logging for the HTTP call
- All serde model structs must derive `Debug, Clone, Serialize, Deserialize`
- Use `reqwest::Client` connection pooling — create the client once and reuse
- Rate limiting must be transparent to callers — the client handles it internally
- Pagination must be automatic — endpoint methods return the full collected data

## API Base URL
`https://api.clickup.com/api/v2`

## HTTP Client Defaults
- `Authorization: {token}` header on every request
- `Content-Type: application/json`
- `User-Agent: clickup-rs/0.1.0`
- Timeout: 30 seconds
- TLS: rustls (no system OpenSSL)

## Error Mapping
- HTTP 401 → `ClickUpError::AuthError`
- HTTP 404 → `ClickUpError::NotFound`
- HTTP 429 → `ClickUpError::RateLimited` (parse `X-RateLimit-Reset` header)
- Other 4xx/5xx → `ClickUpError::ApiError` (parse `err` and `ECODE` from response body)
- Network failures → `ClickUpError::NetworkError`
- JSON parse failures → `ClickUpError::DeserializationError`
