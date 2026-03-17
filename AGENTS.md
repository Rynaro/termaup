# AGENTS.md — clickup-rs

You are building **clickup-rs**, a fast CLI and TUI client for the ClickUp project management platform, written entirely in Rust.

## Identity & Mission

- This is a greenfield Rust project — there is NO existing ClickUp CLI/TUI in the Rust ecosystem.
- The goal is to replace the bloated ClickUp web app for power users who prefer the terminal.
- Quality, performance, and developer experience are paramount.

## Architecture

The project is a **Cargo workspace** with three member crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `crates/clickup-api` | Library | Pure async API client for ClickUp API v2 |
| `crates/clickup-cli` | Binary | clap-based CLI application |
| `crates/clickup-tui` | Binary | ratatui-based TUI application |

Dependency flow: `clickup-cli` and `clickup-tui` both depend on `clickup-api`. The two binaries NEVER depend on each other.

## ClickUp API v2 Reference

Base URL: `https://api.clickup.com/api/v2`

### Authentication
- **Personal Token**: Sent as `Authorization: {token}` header (token starts with `pk_`)
- **OAuth2**: Authorization code flow → `POST /oauth/token` to exchange code for access token, then `Authorization: Bearer {token}`
- Token validation: `GET /user` returns the authenticated user

### Resource Hierarchy
```
Workspace (Team) → Space → Folder → List → Task
                          └→ List (folderless) → Task
```

### Key Endpoints (MVP scope)
| Method | Endpoint | Returns |
|--------|----------|---------|
| GET | `/user` | Authenticated user info |
| GET | `/team` | All workspaces (teams) |
| GET | `/team/{team_id}/space` | Spaces in a workspace |
| GET | `/space/{space_id}` | Space details |
| GET | `/space/{space_id}/folder` | Folders in a space |
| GET | `/folder/{folder_id}/list` | Lists in a folder |
| GET | `/space/{space_id}/list` | Folderless lists in a space |
| GET | `/list/{list_id}` | List details |
| GET | `/list/{list_id}/task` | Tasks in a list (paginated, `page` param) |
| GET | `/task/{task_id}` | Task details (supports `?include_subtasks=true&include_markdown_description=true`) |
| GET | `/task/{task_id}/comment` | Task comments |

### Rate Limits
- 100 req/min (Free/Unlimited/Business), 1000 (Business+), 10000 (Enterprise)
- Response headers: `X-RateLimit-Remaining`, `X-RateLimit-Reset` (Unix timestamp)
- HTTP 429 on rate limit exceeded — must implement backoff

### Pagination
- Task lists use `page` parameter (0-indexed)
- Response includes a `last_page` boolean field in `TasksResponse`
- Always paginate until `last_page == true`

### Error Responses
```json
{ "err": "error message", "ECODE": "ERROR_CODE" }
```

## Rust Conventions

### General
- Rust edition: 2024
- All public items MUST have doc comments (`///`)
- Use `thiserror` for library error types, `anyhow` for binary error handling
- Use `tokio` as the async runtime everywhere
- Prefer `reqwest` with `rustls-tls` (no OpenSSL dependency)
- All structs that cross serialization boundaries derive `Debug, Clone, Serialize, Deserialize`
- Use `#[serde(rename_all = "snake_case")]` as default, with per-field `#[serde(rename = "...")]` or `#[serde(alias = "...")]` for ClickUp's mixed-case API responses
- Never use `unwrap()` in library or binary code — use `?` operator or `unwrap_or_default()`
- `unwrap()` is ONLY acceptable in test code

### Error Handling
- `clickup-api` defines `ClickUpError` enum with `thiserror`
- `clickup-api` exports `pub type Result<T> = std::result::Result<T, ClickUpError>`
- Binary crates use `anyhow::Result` at the top level and convert from `ClickUpError`

### Async Patterns
- All API calls are async
- TUI uses `tokio::sync::mpsc` channels for async data loading → UI event communication
- Never block the main thread in the TUI — spawn data fetches on `tokio::spawn`

### Testing
- Use `wiremock` for HTTP mocking in API client tests
- Use `insta` for snapshot testing in CLI output tests
- Use ratatui's `TestBackend` for TUI widget rendering tests
- All test fixtures go in `tests/fixtures/` directories

## Dependency Stack

| Purpose | Crate | Notes |
|---------|-------|-------|
| HTTP | `reqwest` 0.12 | features: json, rustls-tls |
| Async | `tokio` 1.x | features: full |
| Serialization | `serde` 1 + `serde_json` 1 | features: derive |
| CLI | `clap` 4 | features: derive, env |
| TUI | `ratatui` 0.29 | — |
| Terminal | `crossterm` 0.28 | — |
| Errors (lib) | `thiserror` 2 | — |
| Errors (bin) | `anyhow` 1 | — |
| Logging | `tracing` 0.1 + `tracing-subscriber` 0.3 | features: env-filter |
| Secrets | `keyring` 3 | OS-native credential storage |
| Config | `toml` 0.8 + `dirs` 6 | Config at ~/.config/clickup-rs/ |
| Markdown (CLI) | `termimad` 0.30 | Terminal markdown rendering |
| Markdown (TUI) | `pulldown-cmark` 0.12 | Parse MD → ratatui spans |
| Tables (CLI) | `comfy-table` 7 | — |
| Colors | `owo-colors` 4 | — |
| Dates | `chrono` 0.4 | — |
| Prompts | `dialoguer` 0.11 | Interactive selection |
| Password | `rpassword` 7 | Hidden token input |
| URLs | `url` 2 | — |
| Secrets wrapping | `secrecy` 0.10 | — |
| Testing HTTP | `wiremock` 0.6 | dev-dependency |
| Snapshot testing | `insta` | dev-dependency |

## File Organization Patterns

### API Client Crate (`crates/clickup-api/src/`)
```
lib.rs          → pub mod declarations
error.rs        → ClickUpError enum + Result type alias
config.rs       → Config struct, load/save, paths
auth.rs         → TokenStorage (keyring + file fallback)
client.rs       → ClickUpClient struct, HTTP methods
rate_limiter.rs → Rate limit tracking and auto-wait
pagination.rs   → Generic pagination helpers
models/         → One file per entity (user.rs, workspace.rs, space.rs, etc.)
endpoints/      → One file per resource (users.rs, workspaces.rs, spaces.rs, etc.)
```

### CLI Crate (`crates/clickup-cli/src/`)
```
main.rs          → Clap app setup, tracing init, command routing
commands/        → One file per command group (auth.rs, spaces.rs, lists.rs, tasks.rs)
output.rs        → Formatters (table, json, markdown, colors)
client_factory.rs → Helper to create authenticated ClickUpClient
```

### TUI Crate (`crates/clickup-tui/src/`)
```
main.rs     → Terminal setup/teardown, main loop
app.rs      → App state machine, Screen enum
event.rs    → Event system (crossterm events + async data events)
data.rs     → Async data loading functions
input.rs    → Key event routing per screen
theme.rs    → Color theme definitions
ui/         → One file per screen (workspace_select.rs, space_list.rs, list_view.rs, task_detail.rs)
  layout.rs → Standard layout (top bar, main, bottom bar)
  search.rs → Search/filter overlay
  help.rs   → Help modal overlay
  markdown.rs → MD→ratatui spans converter
widgets/    → Custom ratatui widgets
```

## Configuration & Storage

- Config file: `~/.config/clickup-rs/config.toml`
- Token: OS keyring (service: "clickup-rs", username: "api_token"), fallback to `~/.config/clickup-rs/.token`
- TUI log: `~/.config/clickup-rs/tui.log`
- Environment variables: `CLICKUP_LOG` (log level), `CLICKUP_TOKEN` (token override)

## Code Quality Gates

Every PR must pass:
1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace -- -D warnings`
3. `cargo test --workspace`
4. `cargo build --workspace --release`
