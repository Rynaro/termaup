---
name: error-config
description: Implement shared error types, config system, and token storage in clickup-api. Use this when creating error.rs, config.rs, or auth.rs in the API crate.
---

# Error Handling & Configuration

## Context

This skill creates the foundational error handling, configuration management, and credential storage layer in `clickup-api`. All other API crate code depends on these modules.

## Crates

- `clickup-api`

## Prerequisites

- Skill `foundation` completed — workspace compiles

## Deliverables

- `crates/clickup-api/src/error.rs` — `ClickUpError` enum with `thiserror`, `Result<T>` type alias
- `crates/clickup-api/src/config.rs` — `Config` struct with TOML load/save
- `crates/clickup-api/src/auth.rs` — `TokenStorage` with OS keyring + file fallback

## Implementation

### error.rs

Define `ClickUpError` enum with these variants (all using `#[error("...")]`):

| Variant | Source | When |
|---------|--------|------|
| `AuthError(String)` | — | HTTP 401, invalid/missing token |
| `NotFound(String)` | — | HTTP 404 |
| `RateLimited { reset_at: u64 }` | — | HTTP 429 |
| `ApiError { message: String, ecode: Option<String> }` | — | Other 4xx/5xx |
| `NetworkError(String)` | `reqwest::Error` | Connection failures |
| `DeserializationError(String)` | `serde_json::Error` | JSON parse failures |
| `ConfigError(String)` | `std::io::Error`, `toml::de::Error` | Config read/write |
| `KeyringError(String)` | `keyring::Error` | Credential storage |

Export `pub type Result<T> = std::result::Result<T, ClickUpError>;`

### config.rs

- Config file path: `~/.config/clickup-rs/config.toml`
- Use `dirs::config_dir()` to resolve platform-specific config directory
- `Config` struct fields: `default_workspace_id: Option<String>`, `log_level: Option<String>`
- Implement `Config::load() -> Result<Config>` and `Config::save(&self) -> Result<()>`
- Create parent directories if they don't exist
- See [config-sample.toml](./config-sample.toml) for the expected format

### auth.rs

- `TokenStorage` struct with methods: `store_token()`, `get_token()`, `delete_token()`
- Primary storage: OS keyring via `keyring` crate (service: `"clickup-rs"`, username: `"api_token"`)
- Fallback: encrypted file at `~/.config/clickup-rs/.token`
- Environment override: `CLICKUP_TOKEN` env var takes precedence over stored token
- Wrap token values in `secrecy::SecretString`

## Acceptance

- `Config::load()` → `Config::save()` → `Config::load()` round-trips without data loss
- Token stored in keyring can be retrieved in a subsequent call
- File fallback works when keyring is unavailable
- `CLICKUP_TOKEN` env var overrides stored token
- All `ClickUpError` variants produce human-readable `Display` output
- All error types implement `std::error::Error`
