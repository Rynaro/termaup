---
name: cli-auth
description: Implement authentication commands for clickup-cli. Use this when creating auth.rs, output.rs, or client_factory.rs in the CLI crate.
---

# CLI Auth

## Context

This skill implements the CLI authentication workflow — login, logout, status check, and workspace switching. It also creates the output helpers and client factory that all other CLI commands depend on.

## Crates

- `clickup-cli`

## Prerequisites

- Skill `api-endpoints` completed — `ClickUpClient` with endpoint methods exists

## Deliverables

- `crates/clickup-cli/src/commands/auth.rs` — `login`, `logout`, `status`, `switch` subcommands
- `crates/clickup-cli/src/output.rs` — Success/error/info helpers with colored prefixes
- `crates/clickup-cli/src/client_factory.rs` — Create authenticated `ClickUpClient` from stored token

## Implementation

### commands/auth.rs

Use `clap` derive macros:

```rust
#[derive(Subcommand)]
pub enum AuthCommand {
    /// Log in with a ClickUp API token
    Login,
    /// Log out and remove stored credentials
    Logout,
    /// Show current authentication status
    Status,
    /// Switch default workspace
    Switch,
}
```

- `login`: Prompt for token with `rpassword::read_password()`, validate via `get_authenticated_user()`, store with `TokenStorage`, list workspaces and prompt for default with `dialoguer::Select`
- `logout`: Delete token from `TokenStorage`, confirm with success message
- `status`: Load token, call `get_authenticated_user()`, display username/email/workspace
- `switch`: List workspaces with `dialoguer::Select`, update `Config.default_workspace_id`

### output.rs

```rust
pub fn success(msg: &str)  // "✓ {msg}" in green
pub fn error(msg: &str)    // "✗ {msg}" in red
pub fn info(msg: &str)     // "ℹ {msg}" in blue
```

Use `owo-colors` for coloring — never raw ANSI escape codes.

### client_factory.rs

```rust
pub async fn create_client() -> anyhow::Result<ClickUpClient>
```

1. Try `CLICKUP_TOKEN` env var first
2. Fall back to `TokenStorage::get_token()`
3. Return descriptive error if no token found

## Acceptance

- `clickup auth login` → prompts for token → validates → stores → shows user info
- `clickup auth status` → shows authenticated user and default workspace
- `clickup auth switch` → lists workspaces → updates default
- `clickup auth logout` → removes stored credentials
- All output uses colored prefixes (✓, ✗, ℹ)
- Errors propagate with `.context()` messages
