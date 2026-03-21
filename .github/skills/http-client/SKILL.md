---
name: http-client
description: Build the core HTTP client with rate limiting, retry, and pagination in clickup-api. Use this when creating client.rs, rate_limiter.rs, or pagination.rs.
---

# HTTP Client

## Context

This skill implements the core `ClickUpClient` struct that all endpoint methods use. It includes transparent rate limiting, automatic retry on 429 responses, and generic pagination helpers.

## Crates

- `clickup-api`

## Prerequisites

- Skill `error-config` completed — `ClickUpError` and `Config` types exist

## Deliverables

- `crates/clickup-api/src/client.rs` — `ClickUpClient` struct with HTTP methods
- `crates/clickup-api/src/rate_limiter.rs` — Rate limit tracker with auto-wait
- `crates/clickup-api/src/pagination.rs` — Generic page-crawling helper

## Implementation

### client.rs

```rust
pub struct ClickUpClient {
    http: reqwest::Client,
    base_url: String,        // "https://api.clickup.com/api/v2"
    token: SecretString,
    rate_limiter: RateLimiter,
}
```

- Constructor: `ClickUpClient::new(token: SecretString) -> Result<Self>`
- Build `reqwest::Client` once with:
  - `Authorization: {token}` default header
  - `Content-Type: application/json` default header
  - `User-Agent: clickup-rs/0.1.0`
  - Timeout: 30 seconds
  - TLS: `rustls-tls` (no system OpenSSL)
- Methods:
  - `async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T>`
  - `async fn get_with_params<T: DeserializeOwned>(&self, path: &str, params: &[(&str, &str)]) -> Result<T>`
- Both methods must:
  1. Call `self.rate_limiter.wait_if_needed().await` before the request
  2. Execute the request
  3. Call `self.rate_limiter.update_from_response(&response)` after
  4. Map HTTP status codes to `ClickUpError` variants (see api-crate instruction §Error Mapping)
  5. Log the request with `tracing::debug!`

### rate_limiter.rs

```rust
pub struct RateLimiter {
    remaining: AtomicU32,
    reset_at: AtomicU64,  // Unix timestamp
}
```

- `update_from_response(response: &Response)` — parse `X-RateLimit-Remaining` and `X-RateLimit-Reset` headers
- `wait_if_needed()` — if `remaining == 0`, sleep until `reset_at` timestamp
- Thread-safe: use `std::sync::atomic` types

### pagination.rs

- Generic helper for paginated endpoints (task lists use `page` parameter, 0-indexed)
- Takes a closure that fetches one page and returns `(Vec<T>, bool)` where `bool` is `last_page`
- Accumulates all items across pages
- Returns `Vec<T>` with all collected items

### Response diagnostics

- Log full response bodies at `tracing::trace!` level for debugging (gated behind `CLICKUP_LOG=trace`)
- On deserialization failure, log the endpoint path and first 500 chars of the body at `tracing::error!` level
- The `DeserializationError` variant must include `endpoint`, `message`, and `body_preview` fields
- Never let a raw `serde_json::Error` propagate without endpoint context — use `ClickUpError::deserialization()` helper

## Acceptance

- ClickUpClient handles HTTP 401 → `ClickUpError::AuthError`
- ClickUpClient handles HTTP 404 → `ClickUpError::NotFound`
- ClickUpClient handles HTTP 429 → waits and retries (transparent to caller)
- ClickUpClient handles other 4xx/5xx → `ClickUpError::ApiError` with parsed `err` and `ECODE`
- Rate limiter correctly parses `X-RateLimit-*` headers
- Rate limiter sleeps until reset timestamp when remaining hits 0
- Pagination collects all pages until `last_page == true`
- All operations tested with `wiremock::MockServer`
