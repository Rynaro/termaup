# clickup-rs — Coding Conventions

## Context

clickup-rs is a Rust workspace with 3 crates: `clickup-api` (library), `clickup-cli` (binary), `clickup-tui` (binary). These conventions apply globally to all code in the workspace.

## Language & Style

- Rust edition: 2024
- All public items must have `///` doc comments with a summary line
- Use `snake_case` for functions/variables, `CamelCase` for types, `SCREAMING_SNAKE` for constants
- Max line width: 100 characters (enforced by rustfmt)
- Prefer early returns over deep nesting
- Use `Self` where possible inside impl blocks
- Group imports: std → external crates → crate-internal, separated by blank lines

## Error Handling

- Library code (`clickup-api`): use `thiserror` and the crate's own `Result<T>` alias
- Binary code (`clickup-cli`, `clickup-tui`): use `anyhow::Result` and `?` propagation
- NEVER use `.unwrap()` outside of test code
- NEVER use `panic!()` in library code
- Use `.context("descriptive message")` from anyhow when propagating errors in binaries

## Async

- All I/O operations are async using tokio
- Prefer `tokio::spawn` for background tasks
- Use `tokio::sync::mpsc` for cross-task communication (TUI data loading)
- Always set timeouts on HTTP requests (30s default)
- Never block the main thread in the TUI — spawn data fetches on `tokio::spawn`

## Serialization

- All API models derive `Debug, Clone, Serialize, Deserialize`
- Use `#[serde(default)]` on Vec fields that may be absent from the API response
- Use `#[serde(rename = "type")]` for the reserved keyword field in Status types
- Use `#[serde(rename_all = "snake_case")]` as default, with per-field `#[serde(rename = "...")]` or `#[serde(alias = "...")]` for ClickUp's mixed-case API responses
- Prefer `Option<T>` over default values for nullable API fields

## Testing

- Write tests in `#[cfg(test)] mod tests {}` blocks at the bottom of each file
- Use `wiremock` for API integration tests — never call the real ClickUp API
- Use `tokio::test` for async tests
- Use `insta` for snapshot testing in CLI output tests
- Use ratatui's `TestBackend` for TUI widget rendering tests
- Fixture JSON files go in `tests/fixtures/` relative to each crate
- Test names should be descriptive: `test_get_task_returns_correct_fields`

## Dependencies

- Use workspace dependencies (defined in root `Cargo.toml`) whenever possible
- Reference them in member crates as `dependency.workspace = true`
- Pin major versions, allow minor/patch flexibility

## Git

- Commit messages: conventional commits format (feat:, fix:, refactor:, test:, docs:, chore:)
- One logical change per commit
- PR titles match the conventional commit format
